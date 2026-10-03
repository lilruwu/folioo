// sync::library — the on-disk layout of a sync folder, and nothing else.
//
// A library is a plain directory of JSON files that an external tool copies
// between machines:
//
//   Folioo/
//     library.json        marker + format version
//     notes/<id>.json     one file per note (active or trashed)
//     deleted/<id>.json   one tombstone per permanently deleted note
//     tags.json           the whole tag list
//
// This module knows how to find, create, read and write that layout. It makes
// no sync decisions: what to write, read or delete is the engine's job.

use std::collections::BTreeMap;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

use serde::{Deserialize, Serialize};

use crate::{Folder, Note};

/// Subdirectory created inside the folder the user chooses.
pub const LIBRARY_DIR: &str = "Folioo";
const MARKER: &str = "library.json";
const NOTES_DIR: &str = "notes";
const DELETED_DIR: &str = "deleted";
const TAGS_FILE: &str = "tags.json";

/// The library format this build reads and writes. A library declaring a newer
/// one is refused rather than half-understood.
pub const FORMAT: u32 = 1;

// ── Errors ──────────────────────────────────────────────────────────────────

#[derive(Debug)]
pub enum LibraryError {
    /// The marker is missing: the folder is unmounted, unplugged, or was
    /// emptied. Never a reason to delete anything — only to not sync.
    Unavailable,
    /// Written by a newer Folioo than this one.
    NewerFormat(u32),
    /// A library file exists but isn't what it should be.
    Corrupt(String),
    /// An id that can't safely name a file.
    InvalidId(String),
    Io(io::Error),
}

impl std::fmt::Display for LibraryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unavailable => write!(f, "The sync folder is not available"),
            Self::NewerFormat(v) => write!(
                f,
                "The sync folder was written by a newer version of Folioo (format {v}); \
                 update Folioo to sync with it"
            ),
            Self::Corrupt(what) => write!(f, "The sync folder is damaged: {what}"),
            Self::InvalidId(id) => write!(f, "Cannot store a note with the id {id:?}"),
            Self::Io(e) => write!(f, "Could not access the sync folder: {e}"),
        }
    }
}

impl From<io::Error> for LibraryError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, LibraryError>;

// ── File formats ────────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct Marker {
    format: u32,
}

/// A note as stored in `notes/<id>.json`: every field the app has, plus the
/// millisecond modification time sync compares on.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NoteFile {
    #[serde(flatten)]
    pub note: Note,
    #[serde(rename = "updatedMs")]
    pub updated_ms: i64,
}

/// `deleted/<id>.json`: a note was permanently deleted at `deleted_ms`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tombstone {
    pub id: String,
    #[serde(rename = "deletedMs")]
    pub deleted_ms: i64,
}

/// `tags.json`: the complete tag list, in display order, and when it last
/// changed. It syncs as a whole — the newer list wins.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TagsFile {
    #[serde(rename = "updatedMs")]
    pub updated_ms: i64,
    pub tags: Vec<Folder>,
}

// ── Ids ─────────────────────────────────────────────────────────────────────

/// Whether `id` can name a canonical note or tombstone file.
///
/// Deliberately permissive about the *shape* of an id — any id the database
/// might hold must pass, or its file would be ignored by every scan and the
/// note would never sync — and strict about characters: no dots, spaces,
/// slashes or parentheses. That keeps a crafted id such as `../x` from ever
/// becoming a path outside the library, and means any other file that lands in
/// the folder (a sync tool's conflict copy, a stray document) is simply not a
/// note.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 128
        && id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

// ── Atomic writes ───────────────────────────────────────────────────────────

/// Write `bytes` to `path` so that anything watching the directory sees either
/// the old file or the complete new one, never a partial write.
///
/// The data goes to a hidden temporary file in the *same* directory — rename
/// is only atomic within one filesystem — is flushed to disk, and is then
/// renamed over the target. The temporary name doesn't end in `.json`, so a
/// scan never mistakes a leftover one for a note.
pub fn write_atomic(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let dir = path
        .parent()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no directory"))?;
    let name = path
        .file_name()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "path has no file name"))?
        .to_string_lossy();
    let tmp = dir.join(format!(
        ".{name}.tmp-{}-{}",
        std::process::id(),
        crate::now_ms()
    ));

    let result = (|| {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        fs::rename(&tmp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}

fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec_pretty(value)
        .map_err(|e| LibraryError::Corrupt(format!("could not encode {}: {e}", path.display())))?;
    write_atomic(path, &bytes)?;
    Ok(())
}

fn read_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let bytes = fs::read(path)?;
    serde_json::from_slice(&bytes)
        .map_err(|e| LibraryError::Corrupt(format!("{} is not valid: {e}", path.display())))
}

// ── Locating and creating a library ─────────────────────────────────────────

/// The library directory for a folder the user chose: the folder itself when
/// it already is a library (a second machine picking the synced folder
/// directly), otherwise a `Folioo/` subdirectory, so choosing the root of a
/// Dropbox never scatters note files through it.
pub fn resolve_library(chosen: &Path) -> PathBuf {
    if chosen.join(MARKER).is_file() {
        chosen.to_path_buf()
    } else {
        chosen.join(LIBRARY_DIR)
    }
}

fn check_marker(dir: &Path) -> Result<()> {
    let marker: Marker = read_json(&dir.join(MARKER))?;
    if marker.format > FORMAT {
        return Err(LibraryError::NewerFormat(marker.format));
    }
    Ok(())
}

/// Create a library at `dir`, or adopt the one already there.
///
/// Only ever called when the user chooses a folder in Settings. A background
/// sync never initialises anything: a marker that has gone missing there means
/// the folder is unavailable, not that it should be recreated.
///
/// An existing marker is validated and left untouched, so adopting another
/// machine's library never rewrites it — and one written by a newer Folioo is
/// refused before anything is created.
pub fn init_library(dir: &Path) -> Result<Library> {
    let marker = dir.join(MARKER);
    if marker.is_file() {
        check_marker(dir)?;
    } else {
        fs::create_dir_all(dir)?;
        write_json(&marker, &Marker { format: FORMAT })?;
    }
    open_library(dir)
}

/// Open an existing library for a sync pass.
///
/// A missing marker is `Unavailable` — the common shape of an rclone mount that
/// didn't come up, which is an ordinary empty directory. The note and tombstone
/// directories are recreated if a sync tool dropped them (some don't copy empty
/// directories); that is safe because the marker proves this is a library.
pub fn open_library(dir: &Path) -> Result<Library> {
    if !dir.join(MARKER).is_file() {
        return Err(LibraryError::Unavailable);
    }
    check_marker(dir)?;
    fs::create_dir_all(dir.join(NOTES_DIR))?;
    fs::create_dir_all(dir.join(DELETED_DIR))?;
    Ok(Library {
        root: dir.to_path_buf(),
    })
}

// ── Scanning ────────────────────────────────────────────────────────────────

/// What a file looked like when last seen. Comparing stamps lets a pass skip
/// parsing every file that hasn't changed — decisive for notes carrying images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileStamp {
    pub size: u64,
    pub mtime_ms: i64,
}

impl FileStamp {
    fn of(meta: &fs::Metadata) -> Self {
        let mtime_ms = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(UNIX_EPOCH).ok())
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0);
        Self {
            size: meta.len(),
            mtime_ms,
        }
    }
}

/// One directory listing of a library.
#[derive(Debug, Default)]
pub struct Scan {
    /// Canonical `notes/<id>.json` files, by id.
    pub notes: BTreeMap<String, FileStamp>,
    /// Canonical `deleted/<id>.json` files, by id.
    pub tombstones: BTreeMap<String, FileStamp>,
}

/// Split a directory entry into its id, if it is a canonical `<id>.json` file.
fn canonical_id(file_name: &str) -> Option<&str> {
    file_name
        .strip_suffix(".json")
        .filter(|stem| is_valid_id(stem))
}

pub struct Library {
    root: PathBuf,
}

impl Library {
    fn note_path(&self, id: &str) -> Result<PathBuf> {
        if !is_valid_id(id) {
            return Err(LibraryError::InvalidId(id.to_string()));
        }
        Ok(self.root.join(NOTES_DIR).join(format!("{id}.json")))
    }

    fn tombstone_path(&self, id: &str) -> Result<PathBuf> {
        if !is_valid_id(id) {
            return Err(LibraryError::InvalidId(id.to_string()));
        }
        Ok(self.root.join(DELETED_DIR).join(format!("{id}.json")))
    }

    /// List the note and tombstone files. Only canonical `<id>.json` files
    /// count; everything else — our own hidden temporaries, a sync tool's
    /// conflict copies, anything a user drops in — is ignored and left alone.
    pub fn scan(&self) -> Result<Scan> {
        Ok(Scan {
            notes: canonical_files(&self.root.join(NOTES_DIR))?,
            tombstones: canonical_files(&self.root.join(DELETED_DIR))?,
        })
    }

    /// The stamp of one note file, without listing the directory — what a push
    /// of a few changed notes needs instead of a full scan.
    pub fn stat_note(&self, id: &str) -> Result<Option<FileStamp>> {
        match fs::metadata(self.note_path(id)?) {
            Ok(meta) if meta.is_file() => Ok(Some(FileStamp::of(&meta))),
            Ok(_) => Ok(None),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    pub fn has_tombstone(&self, id: &str) -> Result<bool> {
        Ok(self.tombstone_path(id)?.is_file())
    }

    // ── Notes ───────────────────────────────────────────────────────────────

    /// Write a note and return the stamp of the file as written.
    pub fn write_note(&self, file: &NoteFile) -> Result<FileStamp> {
        let path = self.note_path(&file.note.id)?;
        write_json(&path, file)?;
        Ok(FileStamp::of(&fs::metadata(&path)?))
    }

    pub fn read_note(&self, id: &str) -> Result<NoteFile> {
        read_json(&self.note_path(id)?)
    }

    pub fn remove_note(&self, id: &str) -> Result<()> {
        remove_if_present(&self.note_path(id)?)
    }

    // ── Tombstones ──────────────────────────────────────────────────────────

    pub fn write_tombstone(&self, tombstone: &Tombstone) -> Result<()> {
        write_json(&self.tombstone_path(&tombstone.id)?, tombstone)
    }

    pub fn read_tombstone(&self, id: &str) -> Result<Tombstone> {
        read_json(&self.tombstone_path(id)?)
    }

    pub fn remove_tombstone(&self, id: &str) -> Result<()> {
        remove_if_present(&self.tombstone_path(id)?)
    }

    // ── Tags ────────────────────────────────────────────────────────────────

    /// The tag list, or `None` if no machine has written one yet.
    pub fn read_tags(&self) -> Result<Option<TagsFile>> {
        let path = self.root.join(TAGS_FILE);
        if !path.is_file() {
            return Ok(None);
        }
        read_json(&path).map(Some)
    }

    pub fn write_tags(&self, tags: &TagsFile) -> Result<()> {
        write_json(&self.root.join(TAGS_FILE), tags)
    }
}

/// The canonical `<id>.json` files directly inside `dir`, with their stamps.
fn canonical_files(dir: &Path) -> Result<BTreeMap<String, FileStamp>> {
    let mut found = BTreeMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let meta = entry.metadata()?;
        if !meta.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if let Some(id) = canonical_id(&name) {
            found.insert(id.to_string(), FileStamp::of(&meta));
        }
    }
    Ok(found)
}

fn remove_if_present(path: &Path) -> Result<()> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    /// A scratch directory removed when dropped.
    struct TempDir(PathBuf);

    impl TempDir {
        fn new() -> Self {
            static N: AtomicU32 = AtomicU32::new(0);
            let dir = std::env::temp_dir().join(format!(
                "folioo-library-{}-{}",
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

    fn note(id: &str) -> NoteFile {
        NoteFile {
            note: Note {
                id: id.into(),
                title: "Groceries".into(),
                content: "<div>milk</div><img src=\"data:image/png;base64,AAAA\">".into(),
                folder: "Personal".into(),
                favorite: true,
                created: "2026-09-30".into(),
                updated: "2026-10-01".into(),
                deleted_at: Some("2026-10-01".into()),
            },
            updated_ms: 1_790_000_000_123,
        }
    }

    fn names_in(dir: &Path) -> Vec<String> {
        let mut names: Vec<String> = fs::read_dir(dir)
            .unwrap()
            .map(|e| e.unwrap().file_name().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    // ── Layout ──────────────────────────────────────────────────────────────

    #[test]
    fn a_plain_folder_gets_a_folioo_subdirectory() {
        let t = TempDir::new();
        assert_eq!(resolve_library(&t.0), t.0.join("Folioo"));
    }

    #[test]
    fn a_folder_that_already_is_a_library_is_used_directly() {
        let t = TempDir::new();
        init_library(&t.0.join("Folioo")).unwrap();
        // A second machine picks the synced Folioo folder itself.
        let chosen = t.0.join("Folioo");
        assert_eq!(resolve_library(&chosen), chosen);
    }

    #[test]
    fn init_creates_the_marker_and_directories() {
        let t = TempDir::new();
        let dir = resolve_library(&t.0);
        init_library(&dir).unwrap();
        assert_eq!(names_in(&dir), ["deleted", "library.json", "notes"]);
    }

    #[test]
    fn init_adopts_an_existing_library_without_rewriting_it() {
        let t = TempDir::new();
        let dir = t.0.join("Folioo");
        let lib = init_library(&dir).unwrap();
        lib.write_note(&note("n1")).unwrap();
        let marker_before = fs::read(dir.join(MARKER)).unwrap();

        init_library(&dir).unwrap();

        assert_eq!(fs::read(dir.join(MARKER)).unwrap(), marker_before);
        assert!(lib.read_note("n1").is_ok(), "adopting must not clear notes");
    }

    #[test]
    fn a_newer_format_is_refused_and_nothing_is_created() {
        let t = TempDir::new();
        fs::write(t.0.join(MARKER), r#"{"format": 99}"#).unwrap();

        let err = init_library(&t.0).err().unwrap();

        assert!(matches!(err, LibraryError::NewerFormat(99)), "got {err:?}");
        assert_eq!(names_in(&t.0), ["library.json"]);
    }

    // ── Availability ────────────────────────────────────────────────────────

    #[test]
    fn an_empty_folder_is_unavailable_not_initialised() {
        // What an rclone mount point looks like when it isn't mounted.
        let t = TempDir::new();
        let err = open_library(&t.0).err().unwrap();
        assert!(matches!(err, LibraryError::Unavailable), "got {err:?}");
        assert!(names_in(&t.0).is_empty(), "opening must never create anything");
    }

    #[test]
    fn a_library_that_lost_its_marker_is_unavailable() {
        let t = TempDir::new();
        let lib = init_library(&t.0).unwrap();
        lib.write_note(&note("n1")).unwrap();
        fs::remove_file(t.0.join(MARKER)).unwrap();

        assert!(matches!(open_library(&t.0), Err(LibraryError::Unavailable)));
    }

    #[test]
    fn a_malformed_marker_is_reported_as_corrupt() {
        let t = TempDir::new();
        fs::write(t.0.join(MARKER), "not json").unwrap();
        assert!(matches!(open_library(&t.0), Err(LibraryError::Corrupt(_))));
    }

    #[test]
    fn open_recreates_directories_a_sync_tool_dropped() {
        let t = TempDir::new();
        init_library(&t.0).unwrap();
        fs::remove_dir(t.0.join(NOTES_DIR)).unwrap();
        fs::remove_dir(t.0.join(DELETED_DIR)).unwrap();

        open_library(&t.0).unwrap();

        assert!(t.0.join(NOTES_DIR).is_dir());
        assert!(t.0.join(DELETED_DIR).is_dir());
    }

    // ── Atomic writes ───────────────────────────────────────────────────────

    #[test]
    fn an_atomic_write_leaves_only_the_complete_file() {
        let t = TempDir::new();
        let path = t.0.join("a.json");
        write_atomic(&path, b"first").unwrap();
        write_atomic(&path, b"second, longer").unwrap();

        assert_eq!(fs::read(&path).unwrap(), b"second, longer");
        assert_eq!(names_in(&t.0), ["a.json"], "no temporary file may be left behind");
    }

    #[test]
    fn a_failed_atomic_write_leaves_nothing() {
        let t = TempDir::new();
        let path = t.0.join("missing-dir").join("a.json");
        assert!(write_atomic(&path, b"data").is_err());
        assert!(names_in(&t.0).is_empty());
    }

    // ── Ids and scanning ────────────────────────────────────────────────────

    #[test]
    fn ids_that_could_escape_the_library_are_rejected() {
        for bad in ["", "../x", "a/b", "a\\b", "n1.json", "n1 (copy)", ".hidden"] {
            assert!(!is_valid_id(bad), "{bad:?} must be rejected");
        }
        for good in ["n18b643947bdbda3c", "N1", "legacy_id-2"] {
            assert!(is_valid_id(good), "{good:?} must be accepted");
        }

        let t = TempDir::new();
        let lib = init_library(&t.0).unwrap();
        let err = lib.write_note(&note("../escape")).unwrap_err();
        assert!(matches!(err, LibraryError::InvalidId(_)), "got {err:?}");
        assert!(!t.0.join("escape.json").exists());
    }

    #[test]
    fn scan_counts_only_canonical_files_and_leaves_the_rest_alone() {
        let t = TempDir::new();
        let lib = init_library(&t.0).unwrap();
        lib.write_note(&note("n1")).unwrap();
        lib.write_note(&note("n2")).unwrap();
        lib.write_tombstone(&Tombstone { id: "n3".into(), deleted_ms: 5 }).unwrap();

        let notes = t.0.join(NOTES_DIR);
        let syncthing = notes.join("n1.sync-conflict-20261001-120000-ABCDEFG.json");
        let dropbox = notes.join("n2 (conflicted copy 2026-10-01).json");
        fs::write(&syncthing, "{}").unwrap();
        fs::write(&dropbox, "{}").unwrap();
        fs::write(notes.join("README.txt"), "not a note").unwrap();
        fs::write(notes.join(".n1.json.tmp-1-2"), "half written").unwrap();
        fs::write(t.0.join(DELETED_DIR).join("n3 (conflicted copy).json"), "{}").unwrap();

        let scan = lib.scan().unwrap();

        assert_eq!(scan.notes.keys().collect::<Vec<_>>(), ["n1", "n2"]);
        assert_eq!(scan.tombstones.keys().collect::<Vec<_>>(), ["n3"]);
        // Ignored, not touched: a scan never deletes what it doesn't own.
        assert!(syncthing.exists() && dropbox.exists());
    }

    #[test]
    fn scan_stamps_track_what_was_written() {
        let t = TempDir::new();
        let lib = init_library(&t.0).unwrap();
        let written = lib.write_note(&note("n1")).unwrap();

        let scan = lib.scan().unwrap();

        assert_eq!(scan.notes["n1"], written);
        assert_eq!(
            written.size,
            fs::metadata(t.0.join(NOTES_DIR).join("n1.json")).unwrap().len()
        );
    }

    // ── Round trips ─────────────────────────────────────────────────────────

    #[test]
    fn notes_round_trip_with_trash_state_and_timestamp() {
        let t = TempDir::new();
        let lib = init_library(&t.0).unwrap();
        let original = note("n1");
        lib.write_note(&original).unwrap();

        assert_eq!(lib.read_note("n1").unwrap(), original);

        let raw = fs::read_to_string(t.0.join(NOTES_DIR).join("n1.json")).unwrap();
        assert!(raw.contains("\"deletedAt\"") && raw.contains("\"updatedMs\""), "{raw}");
    }



    #[test]
    fn tombstones_and_tags_round_trip() {
        let t = TempDir::new();
        let lib = init_library(&t.0).unwrap();
        assert_eq!(lib.read_tags().unwrap(), None);

        let stone = Tombstone { id: "n9".into(), deleted_ms: 42 };
        lib.write_tombstone(&stone).unwrap();
        assert_eq!(lib.read_tombstone("n9").unwrap(), stone);

        let tags = TagsFile {
            updated_ms: 7,
            tags: vec![
                Folder { name: "Work".into(), color: "#4B85E8".into() },
                Folder { name: "Ideas".into(), color: "#E85252".into() },
            ],
        };
        lib.write_tags(&tags).unwrap();
        assert_eq!(lib.read_tags().unwrap(), Some(tags));
    }

    #[test]
    fn removing_a_missing_file_is_not_an_error() {
        let t = TempDir::new();
        let lib = init_library(&t.0).unwrap();
        lib.remove_note("n1").unwrap();
        lib.remove_tombstone("n1").unwrap();
    }
}
