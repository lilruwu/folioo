// sync::engine — one pass between the local database and a library folder.
//
// For every note id known to either side, the pass decides — from the local
// row, the library's files, and what both sides agreed on at the last sync —
// whether to write, read, merge, or propagate a deletion. Three rules shape it:
//
// * Absence never deletes. A note whose file is missing is written back. Only
//   an explicit tombstone file deletes anything, so an unmounted or emptied
//   folder can never look like "the user deleted everything".
// * No edit is lost to a conflict. When both sides changed a note, the later
//   version stays and the other becomes a "(conflicted copy)" note.
// * The database is never held for the whole pass. Each change is applied
//   only if the local row is still the version the pass looked at; an edit the
//   user makes mid-pass is left for the next pass instead of being overwritten.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Mutex;

use rusqlite::Connection;
use serde::Serialize;

use super::library::{self, FileStamp, Library, LibraryError, NoteFile, TagsFile, Tombstone};
use crate::db::{self, SyncState};
use crate::{Folder, Note};

/// How long a permanent deletion is remembered. A machine offline for longer
/// comes back without learning about it, and re-uploads the note.
pub const TOMBSTONE_RETENTION_MS: i64 = 90 * 24 * 60 * 60 * 1000;

const CONFLICT_MARKER: &str = "(conflicted copy)";

/// Colour for a tag recreated because a note still referred to it.
const RECREATED_TAG_COLOR: &str = "#8A7A66";

// ── Errors and report ───────────────────────────────────────────────────────

#[derive(Debug)]
pub enum SyncError {
    Library(LibraryError),
    Db(rusqlite::Error),
    /// Another thread panicked while holding the database.
    Poisoned,
}

impl std::fmt::Display for SyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Library(e) => write!(f, "{e}"),
            Self::Db(e) => write!(f, "Database error: {e}"),
            Self::Poisoned => write!(f, "The database is unavailable"),
        }
    }
}

impl From<LibraryError> for SyncError {
    fn from(e: LibraryError) -> Self {
        Self::Library(e)
    }
}

impl From<rusqlite::Error> for SyncError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Db(e)
    }
}

/// What a pass did. `errors` holds per-note problems that were skipped — a
/// note that couldn't be read or written is retried on the next pass, and
/// never deleted because of the error.
#[derive(Debug, Default, Clone, Serialize)]
pub struct SyncReport {
    /// Note files written to the library.
    pub written: usize,
    /// Notes created or updated locally from the library.
    pub read: usize,
    /// Conflicts resolved by keeping both versions.
    pub conflicts: usize,
    /// Local notes purged because the library holds their tombstone.
    #[serde(rename = "deletedLocally")]
    pub deleted_locally: usize,
    /// Note files removed from the library because of a local deletion.
    #[serde(rename = "deletedInFolder")]
    pub deleted_in_folder: usize,
    /// The local tag list changed.
    #[serde(rename = "tagsChanged")]
    pub tags_changed: bool,
    /// Library files parsed — zero for an unchanged library.
    #[serde(skip)]
    pub files_parsed: usize,
    pub errors: Vec<String>,
}

impl SyncReport {
    /// Whether anything the UI shows changed, so it knows to reload.
    pub fn changed_locally(&self) -> bool {
        self.read > 0 || self.conflicts > 0 || self.deleted_locally > 0 || self.tags_changed
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

/// Run one sync pass against the library at `library_dir`.
///
/// Fails with `LibraryError::Unavailable` — having touched nothing on either
/// side — when the library marker is missing.
pub fn run(db: &Mutex<Connection>, library_dir: &Path, now_ms: i64) -> Result<SyncReport, SyncError> {
    let lib = library::open_library(library_dir)?;
    let mut pass = Pass {
        db,
        lib,
        now: now_ms,
        report: SyncReport::default(),
    };

    pass.prune_tombstones()?;

    let scan = pass.lib.scan()?;
    let (versions, local_tombstones, states) = pass.with_db(|c| {
        Ok((db::note_versions(c)?, db::tombstones(c)?, db::sync_states(c)?))
    })?;

    let ids: BTreeSet<&str> = versions
        .keys()
        .chain(local_tombstones.keys())
        .chain(scan.notes.keys())
        .chain(scan.tombstones.keys())
        .map(String::as_str)
        .collect();

    for id in ids {
        let input = Input {
            id,
            local_ms: versions.get(id).copied(),
            local_deleted_ms: local_tombstones.get(id).copied(),
            state: states.get(id).copied(),
            file: scan.notes.get(id).copied(),
            tombstone_file: scan.tombstones.contains_key(id),
        };
        match pass.sync_note(&input) {
            Ok(()) => {}
            // The folder vanished mid-pass, or the database is gone: stop
            // rather than keep failing note by note.
            Err(e @ SyncError::Library(LibraryError::Unavailable)) | Err(e @ SyncError::Poisoned) => {
                return Err(e)
            }
            Err(e) => pass.report.errors.push(format!("{id}: {e}")),
        }
    }

    if let Err(e) = pass.sync_tags() {
        match e {
            SyncError::Poisoned => return Err(e),
            e => pass.report.errors.push(format!("tags: {e}")),
        }
    }

    Ok(pass.report)
}

// ── One id ──────────────────────────────────────────────────────────────────

/// Everything known about one id before anything is parsed.
struct Input<'a> {
    id: &'a str,
    /// The local note's `updated_ms`, if the note exists here.
    local_ms: Option<i64>,
    /// When the note was permanently deleted here, if it was.
    local_deleted_ms: Option<i64>,
    /// What both sides agreed on at the last sync.
    state: Option<SyncState>,
    /// The note file, if the library has one.
    file: Option<FileStamp>,
    /// Whether the library has a tombstone file for this id.
    tombstone_file: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Local {
    Note(i64),
    Deleted(i64),
    Absent,
}

/// The library's side of one id, after reading what was needed.
enum Remote {
    /// `content` is `None` when the file is unchanged since the last sync and
    /// so wasn't parsed; `ms` then comes from the sync record.
    Note {
        ms: i64,
        stamp: FileStamp,
        content: Option<NoteFile>,
    },
    Deleted {
        ms: i64,
    },
    Absent,
}

#[derive(Debug, PartialEq)]
enum Action {
    Nothing,
    /// Same version, but the file was touched: remember its new stamp so the
    /// next pass needn't parse it again.
    RefreshStamp,
    /// Local → library.
    WriteNote,
    /// Library → local.
    ReadNote,
    /// Both sides changed: keep both unless they happen to agree.
    Merge,
    /// The library's tombstone wins over the local note.
    PurgeLocal { deleted_ms: i64 },
    /// The local deletion wins over the library's note file.
    WriteTombstone,
    /// Learn a deletion from the library for a note this machine never had.
    RecordTombstone { deleted_ms: i64 },
    /// A tombstone past retention, for a note this machine never had.
    DropExpiredTombstoneFile,
}

/// The whole sync policy for one id, as a pure function of its inputs.
fn decide(local: Local, remote: &Remote, state: Option<SyncState>, cutoff_ms: i64) -> Action {
    // "Changed" means "not the version both sides agreed on last time". With
    // no record at all, both sides count as changed.
    let changed_since_sync = |ms: i64| state.map_or(true, |s| s.synced_ms != ms);

    match (local, remote) {
        (Local::Note(lu), Remote::Note { ms: fm, content, .. }) => {
            match (changed_since_sync(lu), changed_since_sync(*fm)) {
                (false, false) if content.is_some() => Action::RefreshStamp,
                (false, false) => Action::Nothing,
                (true, false) => Action::WriteNote,
                (false, true) => Action::ReadNote,
                (true, true) => Action::Merge,
            }
        }
        (Local::Note(lu), Remote::Deleted { ms: fd }) => {
            // Unchanged here since the last sync: the deletion happened after,
            // and stands. Changed on both sides: the later action wins.
            if !changed_since_sync(lu) || *fd > lu {
                Action::PurgeLocal { deleted_ms: *fd }
            } else {
                Action::WriteNote
            }
        }
        // Absence never deletes.
        (Local::Note(_), Remote::Absent) => Action::WriteNote,

        (Local::Deleted(ld), Remote::Note { ms: fm, .. }) => {
            // The file is still the version this machine deleted, or older
            // than the deletion: the deletion stands. Edited elsewhere after
            // the deletion: the edit wins.
            if !changed_since_sync(*fm) || *fm <= ld {
                Action::WriteTombstone
            } else {
                Action::ReadNote
            }
        }
        (Local::Deleted(_), Remote::Deleted { .. }) => Action::Nothing,
        (Local::Deleted(_), Remote::Absent) => Action::WriteTombstone,

        (Local::Absent, Remote::Note { .. }) => Action::ReadNote,
        (Local::Absent, Remote::Deleted { ms }) if *ms < cutoff_ms => {
            Action::DropExpiredTombstoneFile
        }
        (Local::Absent, Remote::Deleted { ms }) => Action::RecordTombstone { deleted_ms: *ms },
        (Local::Absent, Remote::Absent) => Action::Nothing,
    }
}

/// Whether two versions of a note say the same thing. Dates are left out:
/// they only record *when*, and sync already compares `updated_ms`.
fn same_content(a: &Note, b: &Note) -> bool {
    a.title == b.title
        && a.content == b.content
        && a.folder == b.folder
        && a.favorite == b.favorite
        && a.deleted_at == b.deleted_at
}

fn conflict_title(title: &str) -> String {
    let base = if title.trim().is_empty() {
        "Untitled"
    } else {
        title
    };
    format!("{base} {CONFLICT_MARKER}")
}

fn state_for(synced_ms: i64, stamp: FileStamp) -> SyncState {
    SyncState {
        synced_ms,
        file_size: stamp.size,
        file_mtime_ms: stamp.mtime_ms,
    }
}

fn stamp_of(state: SyncState) -> FileStamp {
    FileStamp {
        size: state.file_size,
        mtime_ms: state.file_mtime_ms,
    }
}

// ── The pass ────────────────────────────────────────────────────────────────

struct Pass<'a> {
    db: &'a Mutex<Connection>,
    lib: Library,
    now: i64,
    report: SyncReport,
}

impl Pass<'_> {
    /// Run `f` with the database locked — briefly, never across file I/O
    /// larger than a single note.
    fn with_db<T>(&self, f: impl FnOnce(&Connection) -> rusqlite::Result<T>) -> Result<T, SyncError> {
        let conn = self.db.lock().map_err(|_| SyncError::Poisoned)?;
        Ok(f(&conn)?)
    }

    fn cutoff(&self) -> i64 {
        self.now - TOMBSTONE_RETENTION_MS
    }

    fn prune_tombstones(&mut self) -> Result<(), SyncError> {
        let cutoff = self.cutoff();
        let expired = self.with_db(|c| db::prune_tombstones(c, cutoff))?;
        for id in expired {
            if let Err(e) = self.lib.remove_tombstone(&id) {
                self.report.errors.push(format!("{id}: {e}"));
            }
        }
        Ok(())
    }

    fn read_remote(&mut self, input: &Input) -> Result<(Remote, bool), SyncError> {
        let id = input.id;

        let note = match input.file {
            None => None,
            Some(stamp) => match input.state {
                // Unchanged since last seen: trust the record, skip the parse.
                Some(state) if stamp_of(state) == stamp => Some((state.synced_ms, stamp, None)),
                _ => {
                    self.report.files_parsed += 1;
                    let file = self.lib.read_note(id)?;
                    if file.note.id != id {
                        return Err(LibraryError::Corrupt(format!(
                            "notes/{id}.json holds the note {:?}",
                            file.note.id
                        ))
                        .into());
                    }
                    Some((file.updated_ms, stamp, Some(file)))
                }
            },
        };

        let tombstone = if !input.tombstone_file {
            None
        } else if input.local_ms.is_none() && input.file.is_none() && input.local_deleted_ms.is_some() {
            // Already known here and there's nothing to compare it with.
            input.local_deleted_ms
        } else {
            self.report.files_parsed += 1;
            let stone = self.lib.read_tombstone(id)?;
            if stone.id != id {
                return Err(LibraryError::Corrupt(format!(
                    "deleted/{id}.json holds the id {:?}",
                    stone.id
                ))
                .into());
            }
            Some(stone.deleted_ms)
        };

        // When a note file and a tombstone file coexist — the sync tool
        // delivered one before the other — the later of the two describes the
        // library; the other is stale and gets cleaned up.
        Ok(match (note, tombstone) {
            (Some((ms, _, _)), Some(deleted)) if deleted > ms => (Remote::Deleted { ms: deleted }, true),
            (Some((ms, stamp, content)), _) => (Remote::Note { ms, stamp, content }, false),
            (None, Some(deleted)) => (Remote::Deleted { ms: deleted }, false),
            (None, None) => (Remote::Absent, false),
        })
    }

    fn sync_note(&mut self, input: &Input) -> Result<(), SyncError> {
        let (remote, stale_note_file) = self.read_remote(input)?;
        let local = match (input.local_ms, input.local_deleted_ms) {
            (Some(ms), _) => Local::Note(ms),
            (None, Some(ms)) => Local::Deleted(ms),
            (None, None) => Local::Absent,
        };
        let id = input.id;

        match decide(local, &remote, input.state, self.cutoff()) {
            Action::Nothing => {
                if stale_note_file {
                    self.lib.remove_note(id)?;
                }
            }

            Action::RefreshStamp => {
                if let (Some(state), Remote::Note { stamp, .. }) = (input.state, &remote) {
                    self.with_db(|c| db::set_sync_state(c, id, state_for(state.synced_ms, *stamp)))?;
                }
            }

            Action::WriteNote => {
                self.write_local_note(id)?;
                if input.tombstone_file {
                    self.lib.remove_tombstone(id)?;
                }
            }

            Action::ReadNote => {
                let Remote::Note { ms, stamp, content } = remote else {
                    unreachable!("ReadNote is only decided for a note file")
                };
                let file = match content {
                    Some(file) => file,
                    None => {
                        self.report.files_parsed += 1;
                        self.lib.read_note(id)?
                    }
                };
                let expected = input.local_ms;
                let applied = self.with_db(|c| db::apply_synced_note(c, &file.note, ms, expected))?;
                if applied {
                    self.with_db(|c| db::set_sync_state(c, id, state_for(ms, stamp)))?;
                    self.report.read += 1;
                    if input.tombstone_file {
                        self.lib.remove_tombstone(id)?;
                    }
                }
            }

            Action::Merge => {
                let Remote::Note { ms, stamp, content: Some(file) } = remote else {
                    unreachable!("Merge implies the file changed, so it was parsed")
                };
                self.merge(id, input.local_ms, file, ms, stamp)?;
            }

            Action::PurgeLocal { deleted_ms } => {
                let expected = input.local_ms.expect("PurgeLocal needs a local note");
                let purged = self.with_db(|c| db::purge_synced_note(c, id, expected, deleted_ms))?;
                if purged {
                    self.with_db(|c| db::delete_sync_state(c, id))?;
                    self.report.deleted_locally += 1;
                    if stale_note_file || input.file.is_some() {
                        self.lib.remove_note(id)?;
                    }
                }
            }

            Action::WriteTombstone => {
                let deleted_ms = input
                    .local_deleted_ms
                    .expect("WriteTombstone needs a local deletion");
                self.lib.write_tombstone(&Tombstone {
                    id: id.to_string(),
                    deleted_ms,
                })?;
                if input.file.is_some() {
                    self.lib.remove_note(id)?;
                    self.report.deleted_in_folder += 1;
                }
                self.with_db(|c| db::delete_sync_state(c, id))?;
            }

            Action::RecordTombstone { deleted_ms } => {
                self.with_db(|c| db::record_tombstone(c, id, deleted_ms))?;
                if stale_note_file {
                    self.lib.remove_note(id)?;
                }
            }

            Action::DropExpiredTombstoneFile => {
                self.lib.remove_tombstone(id)?;
                if stale_note_file {
                    self.lib.remove_note(id)?;
                }
            }
        }
        Ok(())
    }

    /// Write the local note as it is right now, and record the agreement.
    fn write_local_note(&mut self, id: &str) -> Result<(), SyncError> {
        let Some((note, ms)) = self.with_db(|c| db::get_note_versioned(c, id))? else {
            // Purged while the pass ran; the next pass handles its tombstone.
            return Ok(());
        };
        let stamp = self.lib.write_note(&NoteFile {
            note,
            updated_ms: ms,
        })?;
        self.with_db(|c| db::set_sync_state(c, id, state_for(ms, stamp)))?;
        self.report.written += 1;
        Ok(())
    }

    fn merge(
        &mut self,
        id: &str,
        seen_local_ms: Option<i64>,
        file: NoteFile,
        file_ms: i64,
        stamp: FileStamp,
    ) -> Result<(), SyncError> {
        let Some((local, local_ms)) = self.with_db(|c| db::get_note_versioned(c, id))? else {
            return Ok(());
        };
        if Some(local_ms) != seen_local_ms {
            // Edited while the pass ran: decide next time, with all the facts.
            return Ok(());
        }
        let local_wins = local_ms >= file_ms;

        if !same_content(&local, &file.note) {
            // Preserve the losing version as its own note *before* the winner
            // overwrites anything, so a crash in between loses nothing.
            let (loser, loser_ms) = if local_wins {
                (file.note.clone(), file_ms)
            } else {
                (local.clone(), local_ms)
            };
            let copy = Note {
                id: crate::generate_id(),
                title: conflict_title(&loser.title),
                ..loser
            };
            self.with_db(|c| db::insert_note(c, &copy, loser_ms))?;
            let copy_stamp = self.lib.write_note(&NoteFile {
                note: copy.clone(),
                updated_ms: loser_ms,
            })?;
            self.with_db(|c| db::set_sync_state(c, &copy.id, state_for(loser_ms, copy_stamp)))?;
            self.report.conflicts += 1;
            self.report.written += 1;
        }

        if local_wins {
            self.write_local_note(id)?;
        } else {
            let applied =
                self.with_db(|c| db::apply_synced_note(c, &file.note, file_ms, Some(local_ms)))?;
            if applied {
                self.with_db(|c| db::set_sync_state(c, id, state_for(file_ms, stamp)))?;
                self.report.read += 1;
            }
        }
        Ok(())
    }

    // ── Tags ────────────────────────────────────────────────────────────────

    fn sync_tags(&mut self) -> Result<(), SyncError> {
        let (local, local_ms, base) = self.with_db(|c| {
            Ok((
                db::list_folders(c)?,
                db::folders_updated_ms(c)?,
                db::tags_synced_ms(c)?,
            ))
        })?;
        let remote = self.lib.read_tags()?;

        match (remote, base) {
            // No machine has written a list yet: ours becomes the list.
            (None, _) => {
                self.write_tags(local_ms, local)?;
            }

            // First time this machine meets this library: merge both lists, so
            // joining an existing library never drops a tag from either side.
            (Some(remote), None) => {
                let mut merged = remote.tags.clone();
                for tag in &local {
                    if !merged.iter().any(|t| t.name == tag.name) {
                        merged.push(tag.clone());
                    }
                }
                if merged == remote.tags {
                    self.adopt_tags(&remote, local_ms)?;
                } else {
                    let now = self.now;
                    if merged != local {
                        self.replace_local_tags(&merged, now, local_ms)?;
                    }
                    self.write_tags(now, merged)?;
                }
            }

            // Whole list, last writer wins.
            (Some(remote), Some(base)) => {
                let local_changed = local_ms != base;
                let remote_changed = remote.updated_ms != base;
                match (local_changed, remote_changed) {
                    (false, false) => {}
                    (true, false) => self.write_tags(local_ms, local)?,
                    (false, true) => self.adopt_tags(&remote, local_ms)?,
                    (true, true) if local_ms >= remote.updated_ms => self.write_tags(local_ms, local)?,
                    (true, true) => self.adopt_tags(&remote, local_ms)?,
                }
            }
        }

        // Whatever the list became, no note may point at a tag that doesn't
        // exist. Recreating one changes the list, so it's written back.
        let now = self.now;
        let created = self.with_db(|c| db::create_missing_folders(c, RECREATED_TAG_COLOR, now))?;
        if created > 0 {
            let (tags, ms) = self.with_db(|c| Ok((db::list_folders(c)?, db::folders_updated_ms(c)?)))?;
            self.report.tags_changed = true;
            self.write_tags(ms, tags)?;
        }
        Ok(())
    }

    fn write_tags(&mut self, updated_ms: i64, tags: Vec<Folder>) -> Result<(), SyncError> {
        self.lib.write_tags(&TagsFile { updated_ms, tags })?;
        self.with_db(|c| db::set_tags_synced_ms(c, updated_ms))?;
        Ok(())
    }

    /// Take the library's list as ours.
    fn adopt_tags(&mut self, remote: &TagsFile, seen_local_ms: i64) -> Result<(), SyncError> {
        // An empty list can't be right — the app never lets the last tag go —
        // so it is treated as damage rather than as "delete every tag".
        if remote.tags.is_empty() {
            return Err(LibraryError::Corrupt("tags.json lists no tags".into()).into());
        }
        if self.replace_local_tags(&remote.tags, remote.updated_ms, seen_local_ms)? {
            self.with_db(|c| db::set_tags_synced_ms(c, remote.updated_ms))?;
        }
        Ok(())
    }

    fn replace_local_tags(&mut self, tags: &[Folder], updated_ms: i64, seen_local_ms: i64) -> Result<bool, SyncError> {
        let replaced = self.with_db(|c| db::replace_folders(c, tags, updated_ms, seen_local_ms))?;
        if replaced {
            self.report.tags_changed = true;
        }
        Ok(replaced)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sync::library::init_library;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};

    const DAY_MS: i64 = 24 * 60 * 60 * 1000;

    // ── Fixtures ────────────────────────────────────────────────────────────

    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static N: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "folioo-engine-{}-{}",
                std::process::id(),
                N.fetch_add(1, Ordering::Relaxed)
            ));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).unwrap();
            Self(dir)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    /// A shared library folder, initialised the way Settings would.
    fn library() -> TempDir {
        let t = TempDir::new();
        init_library(&t.0).unwrap();
        t
    }

    /// One Folioo installation: its own database.
    struct Machine(Mutex<Connection>);

    impl Machine {
        fn new(tags: &[&str]) -> Self {
            let conn = Connection::open_in_memory().unwrap();
            db::init(&conn).unwrap();
            for tag in tags {
                db::insert_folder(&conn, tag, "#000000", 1).unwrap();
            }
            Self(Mutex::new(conn))
        }

        fn conn(&self) -> std::sync::MutexGuard<'_, Connection> {
            self.0.lock().unwrap()
        }

        fn create(&self, id: &str, title: &str, folder: &str, ms: i64) {
            let note = Note {
                id: id.into(),
                title: title.into(),
                content: format!("<div>{title}</div>"),
                folder: folder.into(),
                favorite: false,
                created: "2026-10-01".into(),
                updated: "2026-10-01".into(),
                deleted_at: None,
            };
            db::insert_note(&self.conn(), &note, ms).unwrap();
        }

        fn edit(&self, id: &str, title: &str, ms: i64) {
            db::update_note(&self.conn(), id, title, &format!("<div>{title}</div>"), "2026-10-01", ms)
                .unwrap();
        }

        fn note(&self, id: &str) -> Option<Note> {
            db::get_note(&self.conn(), id).unwrap()
        }

        /// Every note title, sorted — active and trashed alike.
        fn titles(&self) -> Vec<String> {
            let mut titles: Vec<String> = db::list_all_notes(&self.conn())
                .unwrap()
                .into_iter()
                .map(|n| n.title)
                .collect();
            titles.sort();
            titles
        }

        fn tags(&self) -> Vec<String> {
            db::list_folders(&self.conn()).unwrap().into_iter().map(|f| f.name).collect()
        }

        fn sync(&self, lib: &TempDir, now: i64) -> SyncReport {
            let report = run(&self.0, &lib.0, now).unwrap();
            assert!(report.errors.is_empty(), "sync errors: {:?}", report.errors);
            report
        }
    }

    fn note_file(lib: &TempDir, id: &str) -> PathBuf {
        lib.0.join("notes").join(format!("{id}.json"))
    }

    fn tombstone_file(lib: &TempDir, id: &str) -> PathBuf {
        lib.0.join("deleted").join(format!("{id}.json"))
    }

    // ── Notes travel ────────────────────────────────────────────────────────

    #[test]
    fn notes_created_on_each_machine_reach_the_other() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "From A", "Personal", 10);
        b.create("n2", "From B", "Personal", 20);

        a.sync(&lib, 100);
        b.sync(&lib, 200);
        a.sync(&lib, 300);

        assert_eq!(a.titles(), ["From A", "From B"]);
        assert_eq!(b.titles(), ["From A", "From B"]);
    }

    #[test]
    fn an_edit_on_one_machine_reaches_the_other() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Draft", "Personal", 10);
        a.sync(&lib, 100);
        b.sync(&lib, 200);

        a.edit("n1", "Final", 300);
        a.sync(&lib, 400);
        b.sync(&lib, 500);

        assert_eq!(b.note("n1").unwrap().title, "Final");
    }

    #[test]
    fn trashed_notes_round_trip_with_their_deletion_date() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Old", "Personal", 10);
        db::trash_note(&a.conn(), "n1", "2026-10-01", 20).unwrap();

        a.sync(&lib, 100);
        b.sync(&lib, 200);

        let on_b = b.note("n1").expect("the trashed note travels");
        assert_eq!(on_b.deleted_at.as_deref(), Some("2026-10-01"));
        assert!(db::list_notes(&b.conn()).unwrap().is_empty(), "it must stay in the trash");
    }

    // ── Conflicts ───────────────────────────────────────────────────────────

    #[test]
    fn a_note_edited_on_both_machines_keeps_both_versions() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Shared", "Personal", 10);
        a.sync(&lib, 100);
        b.sync(&lib, 200);

        a.edit("n1", "A's edit", 300);
        b.edit("n1", "B's edit", 400); // later
        a.sync(&lib, 500);
        let report = b.sync(&lib, 600);
        a.sync(&lib, 700);

        assert_eq!(report.conflicts, 1);
        assert_eq!(b.note("n1").unwrap().title, "B's edit", "the later edit stays as the note");
        let expected = ["A's edit (conflicted copy)", "B's edit"];
        assert_eq!(b.titles(), expected);
        assert_eq!(a.titles(), expected, "both machines converge");
    }

    #[test]
    fn identical_copies_with_no_shared_history_are_not_a_conflict() {
        // Both machines imported the same backup before ever syncing.
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Same", "Personal", 10);
        b.create("n1", "Same", "Personal", 20);

        a.sync(&lib, 100);
        let report = b.sync(&lib, 200);

        assert_eq!(report.conflicts, 0);
        assert_eq!(b.titles(), ["Same"]);
    }

    #[test]
    fn an_edit_made_during_the_pass_is_never_overwritten() {
        let m = Machine::new(&["Personal"]);
        m.create("n1", "Before", "Personal", 10);
        let incoming = Note {
            title: "From the folder".into(),
            ..m.note("n1").unwrap()
        };
        // The pass saw version 10; the user has since saved version 20.
        m.edit("n1", "Typed just now", 20);

        let applied = db::apply_synced_note(&m.conn(), &incoming, 15, Some(10)).unwrap();

        assert!(!applied);
        assert_eq!(m.note("n1").unwrap().title, "Typed just now");
    }

    // ── Absence never deletes ───────────────────────────────────────────────

    #[test]
    fn an_unmounted_folder_deletes_and_writes_nothing() {
        let lib = library();
        let a = Machine::new(&["Personal"]);
        a.create("n1", "Precious", "Personal", 10);
        a.sync(&lib, 100);

        // An rclone mount point that didn't come up: an ordinary empty directory.
        let unmounted = TempDir::new();
        let err = run(&a.0, &unmounted.0, 200).unwrap_err();

        assert!(matches!(err, SyncError::Library(LibraryError::Unavailable)), "got {err:?}");
        assert_eq!(a.titles(), ["Precious"]);
        assert_eq!(fs::read_dir(&unmounted.0).unwrap().count(), 0);
    }

    #[test]
    fn a_library_that_lost_its_marker_is_not_synced() {
        let lib = library();
        let a = Machine::new(&["Personal"]);
        a.create("n1", "Precious", "Personal", 10);
        a.sync(&lib, 100);
        fs::remove_file(lib.0.join("library.json")).unwrap();
        fs::remove_file(note_file(&lib, "n1")).unwrap();

        assert!(run(&a.0, &lib.0, 200).is_err());
        assert_eq!(a.titles(), ["Precious"]);
    }

    #[test]
    fn a_note_file_deleted_by_hand_is_written_back() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Keep me", "Personal", 10);
        a.sync(&lib, 100);
        b.sync(&lib, 200);
        fs::remove_file(note_file(&lib, "n1")).unwrap();

        // B had synced it before: a missing file is still not a deletion.
        let report = b.sync(&lib, 300);

        assert_eq!(report.written, 1);
        assert!(note_file(&lib, "n1").exists());
        assert_eq!(a.titles(), ["Keep me"]);
        assert_eq!(b.titles(), ["Keep me"]);
    }

    // ── Tombstones ──────────────────────────────────────────────────────────

    #[test]
    fn a_purge_on_one_machine_removes_the_note_on_the_other() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Doomed", "Personal", 10);
        a.create("n2", "Kept", "Personal", 10);
        a.sync(&lib, 100);
        b.sync(&lib, 200);

        db::purge_note(&a.conn(), "n1", 300).unwrap();
        a.sync(&lib, 400);
        assert!(!note_file(&lib, "n1").exists());
        assert!(tombstone_file(&lib, "n1").exists());

        let report = b.sync(&lib, 500);
        assert_eq!(report.deleted_locally, 1);
        assert_eq!(b.titles(), ["Kept"]);

        // And it stays gone on every later pass, on both sides.
        a.sync(&lib, 600);
        b.sync(&lib, 700);
        assert_eq!(a.titles(), ["Kept"]);
        assert_eq!(b.titles(), ["Kept"]);
    }

    #[test]
    fn an_edit_made_after_a_purge_elsewhere_wins() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Draft", "Personal", 10);
        a.sync(&lib, 100);
        b.sync(&lib, 200);

        db::purge_note(&a.conn(), "n1", 300).unwrap();
        a.sync(&lib, 350);
        b.edit("n1", "Still needed", 400); // after the purge
        b.sync(&lib, 500);

        assert_eq!(b.titles(), ["Still needed"]);
        assert!(note_file(&lib, "n1").exists());
        assert!(!tombstone_file(&lib, "n1").exists());

        a.sync(&lib, 600);
        assert_eq!(a.titles(), ["Still needed"], "the edit brings it back on A");
    }

    #[test]
    fn a_purge_made_after_an_unsynced_edit_still_wins() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        a.create("n1", "Draft", "Personal", 10);
        a.sync(&lib, 100);
        b.sync(&lib, 200);

        b.edit("n1", "Edited offline", 300);
        db::purge_note(&a.conn(), "n1", 400).unwrap(); // later than B's edit
        a.sync(&lib, 450);
        b.sync(&lib, 500);

        assert!(b.titles().is_empty());
    }

    #[test]
    fn tombstones_expire_after_ninety_days() {
        let lib = library();
        let a = Machine::new(&["Personal"]);
        a.create("n1", "Doomed", "Personal", 10);
        a.sync(&lib, 100);
        db::purge_note(&a.conn(), "n1", 1_000).unwrap();
        a.sync(&lib, 2_000);
        assert!(tombstone_file(&lib, "n1").exists());

        a.sync(&lib, 1_000 + 91 * DAY_MS);

        assert!(!tombstone_file(&lib, "n1").exists());
        assert!(db::tombstones(&a.conn()).unwrap().is_empty());
    }

    // ── Files Folioo didn't write ───────────────────────────────────────────

    #[test]
    fn a_sync_tool_conflict_copy_is_ignored_and_left_in_place() {
        let lib = library();
        let a = Machine::new(&["Personal"]);
        a.create("n1", "Real", "Personal", 10);
        a.sync(&lib, 100);
        let copy = lib.0.join("notes").join("n1.sync-conflict-20261001-120000-ABCDEFG.json");
        fs::copy(note_file(&lib, "n1"), &copy).unwrap();

        a.sync(&lib, 200);

        assert_eq!(a.titles(), ["Real"]);
        assert!(copy.exists());
    }

    #[test]
    fn an_unreadable_note_file_is_skipped_not_deleted() {
        let lib = library();
        let a = Machine::new(&["Personal"]);
        a.create("n1", "Fine", "Personal", 10);
        fs::write(note_file(&lib, "n9"), "{ half a note").unwrap();

        let report = run(&a.0, &lib.0, 100).unwrap();

        assert_eq!(report.errors.len(), 1, "{:?}", report.errors);
        assert!(note_file(&lib, "n9").exists(), "never deleted because of an error");
        assert_eq!(a.titles(), ["Fine"], "the rest of the pass still runs");
        assert!(note_file(&lib, "n1").exists());
    }

    // ── Tags ────────────────────────────────────────────────────────────────

    #[test]
    fn joining_a_library_merges_both_tag_lists_and_all_notes() {
        let lib = library();
        let a = Machine::new(&["Work", "Personal"]);
        let b = Machine::new(&["Personal", "Recipes"]);
        a.create("n1", "Report", "Work", 10);
        b.create("n2", "Paella", "Recipes", 20);

        a.sync(&lib, 100);
        b.sync(&lib, 200);
        a.sync(&lib, 300);

        for m in [&a, &b] {
            assert_eq!(m.tags(), ["Work", "Personal", "Recipes"]);
            assert_eq!(m.titles(), ["Paella", "Report"]);
        }
    }

    #[test]
    fn deleting_a_tag_propagates() {
        let lib = library();
        let (a, b) = (Machine::new(&["Work", "Personal"]), Machine::new(&["Work", "Personal"]));
        a.create("n1", "Report", "Work", 10);
        a.sync(&lib, 100);
        b.sync(&lib, 200);
        a.sync(&lib, 250);

        db::delete_folder(&a.conn(), "Work", "Personal", 300).unwrap();
        a.sync(&lib, 400);
        b.sync(&lib, 500);

        assert_eq!(b.tags(), ["Personal"]);
        assert_eq!(b.note("n1").unwrap().folder, "Personal");
    }

    #[test]
    fn a_tag_still_used_by_a_note_is_recreated() {
        let lib = library();
        let (a, b) = (Machine::new(&["Work", "Personal"]), Machine::new(&["Work", "Personal"]));
        a.sync(&lib, 100);
        b.sync(&lib, 200);

        // A deletes Work while B, not yet synced, files a new note under it.
        db::delete_folder(&a.conn(), "Work", "Personal", 300).unwrap();
        b.create("n1", "New task", "Work", 350);
        a.sync(&lib, 400);
        b.sync(&lib, 500);
        a.sync(&lib, 600);

        for m in [&a, &b] {
            assert!(m.tags().contains(&"Work".to_string()), "{:?}", m.tags());
            assert_eq!(m.note("n1").unwrap().folder, "Work");
        }
    }

    // ── Cost ────────────────────────────────────────────────────────────────

    #[test]
    fn a_pass_over_an_unchanged_library_parses_no_files() {
        let lib = library();
        let (a, b) = (Machine::new(&["Personal"]), Machine::new(&["Personal"]));
        for i in 0..5 {
            a.create(&format!("n{i}"), &format!("Note {i}"), "Personal", 10);
        }
        a.sync(&lib, 100);
        b.sync(&lib, 200);

        let again_a = a.sync(&lib, 300);
        let again_b = b.sync(&lib, 400);

        for report in [again_a, again_b] {
            assert_eq!(report.files_parsed, 0, "{report:?}");
            assert_eq!(report.written, 0);
            assert!(!report.changed_locally());
        }
    }

    // ── Policy table ────────────────────────────────────────────────────────

    #[test]
    fn decide_covers_the_deletion_rules() {
        let synced = |ms| Some(SyncState { synced_ms: ms, file_size: 1, file_mtime_ms: 1 });
        let note = |ms| Remote::Note {
            ms,
            stamp: FileStamp { size: 1, mtime_ms: 1 },
            content: None,
        };

        // Absence never deletes, with or without history.
        assert_eq!(decide(Local::Note(5), &Remote::Absent, synced(5), 0), Action::WriteNote);
        assert_eq!(decide(Local::Note(5), &Remote::Absent, None, 0), Action::WriteNote);
        // Unchanged here: a tombstone wins even with an out-of-order clock.
        assert_eq!(
            decide(Local::Note(5), &Remote::Deleted { ms: 1 }, synced(5), 0),
            Action::PurgeLocal { deleted_ms: 1 }
        );
        // Changed here after the deletion: the edit wins.
        assert_eq!(decide(Local::Note(9), &Remote::Deleted { ms: 7 }, synced(5), 0), Action::WriteNote);
        // The library still holds the version this machine deleted.
        assert_eq!(decide(Local::Deleted(3), &note(5), synced(5), 0), Action::WriteTombstone);
        // Edited elsewhere after this machine deleted it.
        assert_eq!(decide(Local::Deleted(3), &note(8), synced(5), 0), Action::ReadNote);
        // An expired tombstone for a note never seen here.
        assert_eq!(
            decide(Local::Absent, &Remote::Deleted { ms: 1 }, None, 10),
            Action::DropExpiredTombstoneFile
        );
    }
}
