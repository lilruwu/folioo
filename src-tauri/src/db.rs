// db.rs — SQLite schema, queries, and first-run seeding.

use std::collections::HashMap;

use rusqlite::{params, Connection, OptionalExtension, Result};

use crate::{Folder, Note, NoteSummary};

/// Create tables if they don't exist and apply lightweight migrations.
pub fn init(conn: &Connection) -> Result<()> {
    // WAL avoids a full journal fsync on every autosave (one UPDATE every few
    // hundred ms while typing); NORMAL is the recommended durability pairing.
    conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA synchronous=NORMAL;")?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS notes (
            id         TEXT PRIMARY KEY,
            title      TEXT NOT NULL DEFAULT '',
            content    TEXT NOT NULL DEFAULT '',
            folder     TEXT NOT NULL DEFAULT 'Personal',
            favorite   INTEGER NOT NULL DEFAULT 0,
            created    TEXT NOT NULL,
            updated    TEXT NOT NULL,
            deleted_at TEXT
        )",
        [],
    )?;
    conn.execute(
        "CREATE TABLE IF NOT EXISTS folders (
            name     TEXT PRIMARY KEY,
            color    TEXT NOT NULL,
            position INTEGER NOT NULL DEFAULT 0
        )",
        [],
    )?;

    // Migration: add deleted_at to databases created before the trash existed.
    let has_deleted_at: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('notes') WHERE name = 'deleted_at'")?
        .exists([])?;
    if !has_deleted_at {
        conn.execute("ALTER TABLE notes ADD COLUMN deleted_at TEXT", [])?;
    }

    // Migration: plain-text shadow of `content`, kept in sync on every write.
    // Lets note listings and search skip the full HTML (which may embed
    // base64 images) entirely.
    let has_content_text: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('notes') WHERE name = 'content_text'")?
        .exists([])?;
    if !has_content_text {
        conn.execute(
            "ALTER TABLE notes ADD COLUMN content_text TEXT NOT NULL DEFAULT ''",
            [],
        )?;
        backfill_content_text(conn)?;
    }

    // Migration: millisecond modification timestamp. The `updated` date is
    // day-granular, so two edits on the same day are indistinguishable — which
    // is unusable for the last-writer-wins comparison sync needs.
    let has_updated_ms: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('notes') WHERE name = 'updated_ms'")?
        .exists([])?;
    if !has_updated_ms {
        conn.execute(
            "ALTER TABLE notes ADD COLUMN updated_ms INTEGER NOT NULL DEFAULT 0",
            [],
        )?;
        backfill_updated_ms(conn)?;
    }

    // Tombstones for permanently deleted notes. Without them a sync cannot tell
    // "deleted here" from "never seen here", and would re-download every note
    // the user purged, forever.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS deletions (
            id         TEXT PRIMARY KEY,
            deleted_ms INTEGER NOT NULL
        )",
        [],
    )?;

    // Small key/value store for state that belongs to no single row — currently
    // the tag list's modification time.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS meta (
            key   TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // Folder sync: per note, the `updated_ms` both sides agreed on at the last
    // successful sync, and the note file's size and mtime as last seen — so a
    // pass can skip parsing files that haven't changed.
    conn.execute(
        "CREATE TABLE IF NOT EXISTS sync_state (
            id            TEXT PRIMARY KEY,
            synced_ms     INTEGER NOT NULL,
            file_size     INTEGER NOT NULL,
            file_mtime_ms INTEGER NOT NULL
        )",
        [],
    )?;

    // Covers list_notes, list_trash and purge_expired.
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_deleted_updated
         ON notes (deleted_at, updated DESC)",
        [],
    )?;
    Ok(())
}

/// Seed `updated_ms` for rows that predate the column, from each row's
/// `updated` date (midnight UTC of that day).
///
/// The result is coarse — every note edited on the same day lands on the same
/// millisecond — but it is monotonic, and the first sync after upgrading
/// establishes real timestamps from then on. A row whose `updated` doesn't
/// parse falls back to 0, so sync treats it as old rather than as newer than a
/// real remote edit.
fn backfill_updated_ms(conn: &Connection) -> Result<()> {
    conn.execute(
        "UPDATE notes
            SET updated_ms = COALESCE(CAST(strftime('%s', updated) AS INTEGER), 0) * 1000",
        [],
    )?;
    Ok(())
}

/// Populate `content_text` for rows that predate the column.
fn backfill_content_text(conn: &Connection) -> Result<()> {
    let rows: Vec<(String, String)> = conn
        .prepare("SELECT id, content FROM notes")?
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
        .collect::<Result<_>>()?;
    let mut stmt = conn.prepare("UPDATE notes SET content_text = ?2 WHERE id = ?1")?;
    for (id, content) in rows {
        stmt.execute(params![id, strip_html(&content)])?;
    }
    Ok(())
}

/// Reduce note HTML to searchable plain text: tags (and everything inside
/// them, e.g. base64 `src` attributes) are dropped, common entities decoded,
/// and whitespace collapsed.
pub fn strip_html(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut chars = html.char_indices();
    let mut last_space = true;
    let push = |out: &mut String, c: char, last_space: &mut bool| {
        if c.is_whitespace() {
            if !*last_space {
                out.push(' ');
                *last_space = true;
            }
        } else {
            out.push(c);
            *last_space = false;
        }
    };
    while let Some((i, c)) = chars.next() {
        match c {
            '<' => {
                in_tag = true;
                // A tag boundary separates words ("<div>a</div><div>b</div>").
                push(&mut out, ' ', &mut last_space);
            }
            '>' if in_tag => in_tag = false,
            _ if in_tag => {}
            '&' => {
                // Decode the entities the editor actually produces.
                let rest = &html[i..];
                let known = [
                    ("&nbsp;", ' '),
                    ("&amp;", '&'),
                    ("&lt;", '<'),
                    ("&gt;", '>'),
                    ("&quot;", '"'),
                    ("&#39;", '\''),
                ];
                if let Some((ent, decoded)) = known.iter().find(|(e, _)| rest.starts_with(e)) {
                    push(&mut out, *decoded, &mut last_space);
                    for _ in 0..ent.chars().count() - 1 {
                        chars.next();
                    }
                } else {
                    push(&mut out, '&', &mut last_space);
                }
            }
            c => push(&mut out, c, &mut last_space),
        }
    }
    out.trim().to_string()
}

// ── Notes ───────────────────────────────────────────────────────────────────

fn row_to_note(row: &rusqlite::Row) -> Result<Note> {
    Ok(Note {
        id: row.get(0)?,
        title: row.get(1)?,
        content: row.get(2)?,
        folder: row.get(3)?,
        favorite: row.get::<_, i64>(4)? != 0,
        created: row.get(5)?,
        updated: row.get(6)?,
        deleted_at: row.get(7)?,
    })
}

const SELECT_COLS: &str = "id, title, content, folder, favorite, created, updated, deleted_at";

fn row_to_summary(row: &rusqlite::Row) -> Result<NoteSummary> {
    Ok(NoteSummary {
        id: row.get(0)?,
        title: row.get(1)?,
        folder: row.get(2)?,
        favorite: row.get::<_, i64>(3)? != 0,
        created: row.get(4)?,
        updated: row.get(5)?,
        deleted_at: row.get(6)?,
        search_text: row.get(7)?,
    })
}

const SUMMARY_COLS: &str =
    "id, title, folder, favorite, created, updated, deleted_at, content_text";

/// Active (non-trashed) notes, without their (potentially huge) content.
pub fn list_notes(conn: &Connection) -> Result<Vec<NoteSummary>> {
    let sql = format!(
        "SELECT {SUMMARY_COLS} FROM notes WHERE deleted_at IS NULL ORDER BY updated DESC, created DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_summary)?;
    rows.collect()
}

/// Notes currently in the trash, most recently deleted first.
pub fn list_trash(conn: &Connection) -> Result<Vec<NoteSummary>> {
    let sql = format!(
        "SELECT {SUMMARY_COLS} FROM notes WHERE deleted_at IS NOT NULL ORDER BY deleted_at DESC, updated DESC"
    );
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_summary)?;
    rows.collect()
}

pub fn get_note(conn: &Connection, id: &str) -> Result<Option<Note>> {
    let sql = format!("SELECT {SELECT_COLS} FROM notes WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_note).optional()
}

pub fn get_summary(conn: &Connection, id: &str) -> Result<Option<NoteSummary>> {
    let sql = format!("SELECT {SUMMARY_COLS} FROM notes WHERE id = ?1");
    conn.query_row(&sql, params![id], row_to_summary).optional()
}

/// Every note (active and trashed) — used for full export/backup.
pub fn list_all_notes(conn: &Connection) -> Result<Vec<Note>> {
    let sql = format!("SELECT {SELECT_COLS} FROM notes ORDER BY updated DESC, created DESC");
    let mut stmt = conn.prepare(&sql)?;
    let rows = stmt.query_map([], row_to_note)?;
    rows.collect()
}

/// Insert-or-replace a whole note (used when importing a backup).
pub fn upsert_note(conn: &Connection, note: &Note, now_ms: i64) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    write_note_row(&tx, note, now_ms)?;
    tx.commit()
}

/// Insert-or-replace a note row stamped `updated_ms`, dropping any tombstone
/// for its id: a note arriving by import or sync is a deliberate resurrection.
fn write_note_row(conn: &Connection, note: &Note, updated_ms: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO notes (id, title, content, content_text, folder, favorite, created, updated, deleted_at, updated_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
         ON CONFLICT(id) DO UPDATE SET
            title=?2, content=?3, content_text=?4, folder=?5, favorite=?6, created=?7, updated=?8, deleted_at=?9, updated_ms=?10",
        params![
            note.id,
            note.title,
            note.content,
            strip_html(&note.content),
            note.folder,
            note.favorite as i64,
            note.created,
            note.updated,
            note.deleted_at,
            updated_ms
        ],
    )?;
    conn.execute("DELETE FROM deletions WHERE id = ?1", params![note.id])?;
    Ok(())
}

pub fn insert_note(conn: &Connection, note: &Note, now_ms: i64) -> Result<()> {
    conn.execute(
        "INSERT INTO notes (id, title, content, content_text, folder, favorite, created, updated, updated_ms)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            note.id,
            note.title,
            note.content,
            strip_html(&note.content),
            note.folder,
            note.favorite as i64,
            note.created,
            note.updated,
            now_ms
        ],
    )?;
    Ok(())
}

pub fn update_note(
    conn: &Connection,
    id: &str,
    title: &str,
    content: &str,
    updated: &str,
    now_ms: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE notes SET title = ?2, content = ?3, content_text = ?4, updated = ?5, updated_ms = ?6
         WHERE id = ?1",
        params![id, title, content, strip_html(content), updated, now_ms],
    )?;
    Ok(())
}

/// Reassign a note to a different tag/folder.
pub fn update_note_folder(
    conn: &Connection,
    id: &str,
    folder: &str,
    updated: &str,
    now_ms: i64,
) -> Result<()> {
    conn.execute(
        "UPDATE notes SET folder = ?2, updated = ?3, updated_ms = ?4 WHERE id = ?1",
        params![id, folder, updated, now_ms],
    )?;
    Ok(())
}

/// Move a note to the trash (soft delete).
pub fn trash_note(conn: &Connection, id: &str, when: &str, now_ms: i64) -> Result<()> {
    conn.execute(
        "UPDATE notes SET deleted_at = ?2, updated_ms = ?3 WHERE id = ?1",
        params![id, when, now_ms],
    )?;
    Ok(())
}

/// Restore a note from the trash.
pub fn restore_note(conn: &Connection, id: &str, now_ms: i64) -> Result<()> {
    conn.execute(
        "UPDATE notes SET deleted_at = NULL, updated_ms = ?2 WHERE id = ?1",
        params![id, now_ms],
    )?;
    Ok(())
}

/// Permanently remove a single note.
pub fn purge_note(conn: &Connection, id: &str, now_ms: i64) -> Result<()> {
    // Tombstone and row removal share a transaction: a purge that lost its
    // marker would be undone by the next sync re-downloading the note.
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT OR REPLACE INTO deletions (id, deleted_ms) VALUES (?1, ?2)",
        params![id, now_ms],
    )?;
    tx.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    tx.commit()
}

/// Permanently remove every trashed note.
pub fn empty_trash(conn: &Connection, now_ms: i64) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT OR REPLACE INTO deletions (id, deleted_ms)
         SELECT id, ?1 FROM notes WHERE deleted_at IS NOT NULL",
        params![now_ms],
    )?;
    tx.execute("DELETE FROM notes WHERE deleted_at IS NOT NULL", [])?;
    tx.commit()
}

/// Drop trashed notes whose deletion date is strictly before `cutoff` (YYYY-MM-DD).
pub fn purge_expired(conn: &Connection, cutoff: &str, now_ms: i64) -> Result<usize> {
    let tx = conn.unchecked_transaction()?;
    tx.execute(
        "INSERT OR REPLACE INTO deletions (id, deleted_ms)
         SELECT id, ?1 FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?2",
        params![now_ms, cutoff],
    )?;
    let n = tx.execute(
        "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
        params![cutoff],
    )?;
    tx.commit()?;
    Ok(n)
}

/// Flip a note's favorite flag and return the new value.
pub fn toggle_favorite(conn: &Connection, id: &str, now_ms: i64) -> Result<bool> {
    let fav: i64 = conn.query_row(
        "UPDATE notes SET favorite = 1 - favorite, updated_ms = ?2 WHERE id = ?1
         RETURNING favorite",
        params![id, now_ms],
        |r| r.get(0),
    )?;
    Ok(fav != 0)
}

// ── Folders / tags ──────────────────────────────────────────────────────────

pub fn list_folders(conn: &Connection) -> Result<Vec<Folder>> {
    let mut stmt =
        conn.prepare("SELECT name, color FROM folders ORDER BY position ASC, name ASC")?;
    let rows = stmt.query_map([], |row| {
        Ok(Folder {
            name: row.get(0)?,
            color: row.get(1)?,
        })
    })?;
    rows.collect()
}

/// Case-insensitive existence check, used to reject duplicate tags.
pub fn folder_exists(conn: &Connection, name: &str) -> Result<bool> {
    let n: i64 = conn.query_row(
        "SELECT COUNT(*) FROM folders WHERE name = ?1 COLLATE NOCASE",
        params![name],
        |r| r.get(0),
    )?;
    Ok(n > 0)
}

pub fn insert_folder(conn: &Connection, name: &str, color: &str, now_ms: i64) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    let next_pos: i64 =
        tx.query_row("SELECT COALESCE(MAX(position), 0) + 1 FROM folders", [], |r| {
            r.get(0)
        })?;
    tx.execute(
        "INSERT INTO folders (name, color, position) VALUES (?1, ?2, ?3)",
        params![name, color, next_pos],
    )?;
    touch_folders(&tx, now_ms)?;
    tx.commit()
}

/// Insert a folder only if its name doesn't already exist (used when importing).
pub fn upsert_folder_ignore(
    conn: &Connection,
    name: &str,
    color: &str,
    now_ms: i64,
) -> Result<()> {
    let next_pos: i64 =
        conn.query_row("SELECT COALESCE(MAX(position), 0) + 1 FROM folders", [], |r| r.get(0))?;
    let inserted = conn.execute(
        "INSERT OR IGNORE INTO folders (name, color, position) VALUES (?1, ?2, ?3)",
        params![name, color, next_pos],
    )?;
    // Only a tag that actually landed changes the list.
    if inserted > 0 {
        touch_folders(conn, now_ms)?;
    }
    Ok(())
}

/// Record that the tag list changed, so sync can compare it against a remote
/// copy without diffing every tag.
pub fn touch_folders(conn: &Connection, now_ms: i64) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES ('folders_updated_ms', ?1)",
        params![now_ms.to_string()],
    )?;
    Ok(())
}

/// When the tag list last changed, or 0 if it never has.
pub fn folders_updated_ms(conn: &Connection) -> Result<i64> {
    let raw: Option<String> = conn
        .query_row(
            "SELECT value FROM meta WHERE key = 'folders_updated_ms'",
            [],
            |r| r.get(0),
        )
        .optional()?;
    Ok(raw.and_then(|v| v.parse().ok()).unwrap_or(0))
}

/// Populate the default tag set the first time the app runs.
pub fn seed_folders_if_empty(conn: &Connection, now_ms: i64) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
    touch_folders(conn, now_ms)?;
    let defaults = [
        ("Work", "#4B85E8"),
        ("Personal", "#52B46B"),
        ("Projects", "#E8A23A"),
        ("Ideas", "#E85252"),
    ];
    for (i, (name, color)) in defaults.iter().enumerate() {
        conn.execute(
            "INSERT INTO folders (name, color, position) VALUES (?1, ?2, ?3)",
            params![name, color, (i + 1) as i64],
        )?;
    }
    Ok(())
}

/// The number of tags currently defined.
pub fn folder_count(conn: &Connection) -> Result<i64> {
    conn.query_row("SELECT COUNT(*) FROM folders", [], |r| r.get(0))
}

/// Delete a tag. Any notes carrying it are reassigned to `fallback`.
/// (Refusing to delete the last remaining tag is enforced in the command layer.)
pub fn delete_folder(conn: &Connection, name: &str, fallback: &str, now_ms: i64) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    // Reassigning a note is a change to that note, so it is stamped too.
    tx.execute(
        "UPDATE notes SET folder = ?2, updated_ms = ?3 WHERE folder = ?1",
        params![name, fallback, now_ms],
    )?;
    tx.execute("DELETE FROM folders WHERE name = ?1", params![name])?;
    touch_folders(&tx, now_ms)?;
    tx.commit()
}

/// First tag other than `exclude` (by display order) — used as a reassignment target.
pub fn first_folder_except(conn: &Connection, exclude: &str) -> Result<Option<String>> {
    conn.query_row(
        "SELECT name FROM folders WHERE name <> ?1 ORDER BY position ASC, name ASC LIMIT 1",
        params![exclude],
        |r| r.get(0),
    )
    .optional()
}

// ── Folder sync ─────────────────────────────────────────────────────────────

/// What was agreed with the library about one note at the last sync.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncState {
    pub synced_ms: i64,
    pub file_size: u64,
    pub file_mtime_ms: i64,
}

pub fn meta_get(conn: &Connection, key: &str) -> Result<Option<String>> {
    conn.query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| r.get(0))
        .optional()
}

pub fn meta_set(conn: &Connection, key: &str, value: &str) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO meta (key, value) VALUES (?1, ?2)",
        params![key, value],
    )?;
    Ok(())
}

pub fn meta_delete(conn: &Connection, key: &str) -> Result<()> {
    conn.execute("DELETE FROM meta WHERE key = ?1", params![key])?;
    Ok(())
}

const SYNC_FOLDER_KEY: &str = "sync_folder";
const TAGS_SYNCED_KEY: &str = "tags_synced_ms";

/// The library directory sync is pointed at, if any.
#[allow(dead_code)] // called by the sync commands (folder-sync phase 4)
pub fn sync_folder(conn: &Connection) -> Result<Option<String>> {
    meta_get(conn, SYNC_FOLDER_KEY)
}

#[allow(dead_code)] // called by the sync commands (folder-sync phase 4)
pub fn set_sync_folder(conn: &Connection, path: &str) -> Result<()> {
    meta_set(conn, SYNC_FOLDER_KEY, path)
}

/// Forget the folder and everything agreed with it. Notes, tags and local
/// tombstones stay: stopping sync never touches the user's data.
#[allow(dead_code)] // called by the sync commands (folder-sync phase 4)
pub fn clear_sync(conn: &Connection) -> Result<()> {
    let tx = conn.unchecked_transaction()?;
    tx.execute("DELETE FROM sync_state", [])?;
    meta_delete(&tx, SYNC_FOLDER_KEY)?;
    meta_delete(&tx, TAGS_SYNCED_KEY)?;
    tx.commit()
}

/// The tag-list timestamp both sides agreed on at the last sync; `None` until
/// this database has synced tags with a library at least once.
pub fn tags_synced_ms(conn: &Connection) -> Result<Option<i64>> {
    Ok(meta_get(conn, TAGS_SYNCED_KEY)?.and_then(|v| v.parse().ok()))
}

pub fn set_tags_synced_ms(conn: &Connection, ms: i64) -> Result<()> {
    meta_set(conn, TAGS_SYNCED_KEY, &ms.to_string())
}

pub fn sync_states(conn: &Connection) -> Result<HashMap<String, SyncState>> {
    let mut stmt =
        conn.prepare("SELECT id, synced_ms, file_size, file_mtime_ms FROM sync_state")?;
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            SyncState {
                synced_ms: r.get(1)?,
                file_size: r.get::<_, i64>(2)? as u64,
                file_mtime_ms: r.get(3)?,
            },
        ))
    })?;
    rows.collect()
}

pub fn set_sync_state(conn: &Connection, id: &str, state: SyncState) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO sync_state (id, synced_ms, file_size, file_mtime_ms)
         VALUES (?1, ?2, ?3, ?4)",
        params![id, state.synced_ms, state.file_size as i64, state.file_mtime_ms],
    )?;
    Ok(())
}

pub fn delete_sync_state(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM sync_state WHERE id = ?1", params![id])?;
    Ok(())
}

/// Every note's `updated_ms`, active and trashed.
pub fn note_versions(conn: &Connection) -> Result<HashMap<String, i64>> {
    let mut stmt = conn.prepare("SELECT id, updated_ms FROM notes")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// A note together with its `updated_ms`, read in one statement so the two
/// always describe the same version.
pub fn get_note_versioned(conn: &Connection, id: &str) -> Result<Option<(Note, i64)>> {
    let sql = format!("SELECT {SELECT_COLS}, updated_ms FROM notes WHERE id = ?1");
    conn.query_row(&sql, params![id], |r| Ok((row_to_note(r)?, r.get(8)?)))
        .optional()
}

fn current_version(conn: &Connection, id: &str) -> Result<Option<i64>> {
    conn.query_row(
        "SELECT updated_ms FROM notes WHERE id = ?1",
        params![id],
        |r| r.get(0),
    )
    .optional()
}

/// Apply a note read from the library — but only if the local row is still
/// the version sync looked at (`expected`, or absent for `None`).
///
/// Sync reads, decides and writes without holding the database for the whole
/// pass, so the user can keep typing. If they edited this note in between,
/// overwriting it would lose that edit; returning `false` leaves it for the
/// next pass, which will see both sides changed and keep both.
pub fn apply_synced_note(
    conn: &Connection,
    note: &Note,
    updated_ms: i64,
    expected: Option<i64>,
) -> Result<bool> {
    let tx = conn.unchecked_transaction()?;
    if current_version(&tx, &note.id)? != expected {
        return Ok(false);
    }
    write_note_row(&tx, note, updated_ms)?;
    tx.commit()?;
    Ok(true)
}

/// Purge a note because the library holds its tombstone — only if the local
/// row is still version `expected`. Records the tombstone with the original
/// deletion time, so it ages out on the same schedule everywhere.
pub fn purge_synced_note(
    conn: &Connection,
    id: &str,
    expected: i64,
    deleted_ms: i64,
) -> Result<bool> {
    let tx = conn.unchecked_transaction()?;
    if current_version(&tx, id)? != Some(expected) {
        return Ok(false);
    }
    tx.execute(
        "INSERT OR REPLACE INTO deletions (id, deleted_ms) VALUES (?1, ?2)",
        params![id, deleted_ms],
    )?;
    tx.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    tx.commit()?;
    Ok(true)
}

pub fn tombstones(conn: &Connection) -> Result<HashMap<String, i64>> {
    let mut stmt = conn.prepare("SELECT id, deleted_ms FROM deletions")?;
    let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
    rows.collect()
}

/// Remember a deletion learned from the library, unless the note exists here —
/// a local note always outranks a bare tombstone until sync compares them.
pub fn record_tombstone(conn: &Connection, id: &str, deleted_ms: i64) -> Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO deletions (id, deleted_ms)
         SELECT ?1, ?2 WHERE NOT EXISTS (SELECT 1 FROM notes WHERE id = ?1)",
        params![id, deleted_ms],
    )?;
    Ok(())
}

/// Drop tombstones older than `cutoff_ms`, returning their ids so the matching
/// files can be removed from the library too.
pub fn prune_tombstones(conn: &Connection, cutoff_ms: i64) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("DELETE FROM deletions WHERE deleted_ms < ?1 RETURNING id")?;
    let rows = stmt.query_map(params![cutoff_ms], |r| r.get(0))?;
    rows.collect()
}

/// Replace the whole tag list with one read from the library, in its order —
/// only if the local list is still the version sync compared (`expected`), so a
/// tag created while the pass ran is never wiped.
pub fn replace_folders(
    conn: &Connection,
    folders: &[Folder],
    updated_ms: i64,
    expected: i64,
) -> Result<bool> {
    let tx = conn.unchecked_transaction()?;
    if folders_updated_ms(&tx)? != expected {
        return Ok(false);
    }
    tx.execute("DELETE FROM folders", [])?;
    for (i, f) in folders.iter().enumerate() {
        tx.execute(
            "INSERT OR IGNORE INTO folders (name, color, position) VALUES (?1, ?2, ?3)",
            params![f.name, f.color, (i + 1) as i64],
        )?;
    }
    touch_folders(&tx, updated_ms)?;
    tx.commit()?;
    Ok(true)
}

/// Create every tag a note refers to but the tag list lacks, so no note is
/// left unreachable. Returns how many were created.
pub fn create_missing_folders(conn: &Connection, color: &str, now_ms: i64) -> Result<usize> {
    let missing: Vec<String> = conn
        .prepare(
            "SELECT DISTINCT folder FROM notes
              WHERE folder NOT IN (SELECT name FROM folders) ORDER BY folder",
        )?
        .query_map([], |r| r.get(0))?
        .collect::<Result<_>>()?;
    for name in &missing {
        insert_folder(conn, name, color, now_ms)?;
    }
    Ok(missing.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A note with the given id, tag and `updated` date.
    fn note(id: &str, updated: &str) -> Note {
        Note {
            id: id.into(),
            title: format!("Note {id}"),
            content: format!("<div>body of {id}</div>"),
            folder: "Personal".into(),
            favorite: false,
            created: updated.into(),
            updated: updated.into(),
            deleted_at: None,
        }
    }

    fn fresh_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init(&conn).unwrap();
        conn
    }

    fn updated_ms_of(conn: &Connection, id: &str) -> i64 {
        conn.query_row(
            "SELECT updated_ms FROM notes WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .unwrap()
    }

    fn tombstone_of(conn: &Connection, id: &str) -> Option<i64> {
        conn.query_row(
            "SELECT deleted_ms FROM deletions WHERE id = ?1",
            params![id],
            |r| r.get(0),
        )
        .optional()
        .unwrap()
    }

    // ── Migration ───────────────────────────────────────────────────────────

    /// A database written before sync existed: no `updated_ms`, no `deletions`.
    fn legacy_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute(
            "CREATE TABLE notes (
                id TEXT PRIMARY KEY, title TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL DEFAULT '', folder TEXT NOT NULL DEFAULT 'Personal',
                favorite INTEGER NOT NULL DEFAULT 0, created TEXT NOT NULL,
                updated TEXT NOT NULL, deleted_at TEXT,
                content_text TEXT NOT NULL DEFAULT ''
            )",
            [],
        )
        .unwrap();
        conn.execute(
            "CREATE TABLE folders (name TEXT PRIMARY KEY, color TEXT NOT NULL,
                                   position INTEGER NOT NULL DEFAULT 0)",
            [],
        )
        .unwrap();
        conn
    }

    #[test]
    fn migration_backfills_updated_ms_without_touching_content() {
        let conn = legacy_db();
        conn.execute(
            "INSERT INTO notes (id, title, content, content_text, created, updated)
             VALUES ('n1', 'Kept', '<p>Kept body</p>', 'Kept body', '2026-01-05', '2026-03-09')",
            [],
        )
        .unwrap();

        init(&conn).unwrap();

        // 2026-03-09T00:00:00Z — `date -u -d 2026-03-09 +%s` gives 1773014400.
        assert_eq!(updated_ms_of(&conn, "n1"), 1_773_014_400_000);
        let (title, content): (String, String) = conn
            .query_row("SELECT title, content FROM notes WHERE id = 'n1'", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(title, "Kept");
        assert_eq!(content, "<p>Kept body</p>");
    }

    #[test]
    fn migration_is_idempotent_and_preserves_stamped_rows() {
        let conn = fresh_db();
        insert_note(&conn, &note("n1", "2026-03-09"), 4_242).unwrap();

        init(&conn).unwrap(); // second startup

        // Re-running init must not re-backfill an already-stamped row.
        assert_eq!(updated_ms_of(&conn, "n1"), 4_242);
    }

    #[test]
    fn migration_survives_an_unparseable_updated_date() {
        let conn = legacy_db();
        conn.execute(
            "INSERT INTO notes (id, created, updated) VALUES ('n1', 'x', 'not-a-date')",
            [],
        )
        .unwrap();

        init(&conn).unwrap();

        // 0 sorts as old, so a real remote edit wins rather than losing to junk.
        assert_eq!(updated_ms_of(&conn, "n1"), 0);
    }

    // ── Stamping ────────────────────────────────────────────────────────────

    #[test]
    fn every_write_path_stamps_updated_ms() {
        let conn = fresh_db();
        insert_note(&conn, &note("n1", "2026-03-09"), 1_000).unwrap();
        assert_eq!(updated_ms_of(&conn, "n1"), 1_000);

        update_note(&conn, "n1", "T", "<p>c</p>", "2026-03-09", 2_000).unwrap();
        assert_eq!(updated_ms_of(&conn, "n1"), 2_000);

        update_note_folder(&conn, "n1", "Work", "2026-03-09", 3_000).unwrap();
        assert_eq!(updated_ms_of(&conn, "n1"), 3_000);

        toggle_favorite(&conn, "n1", 4_000).unwrap();
        assert_eq!(updated_ms_of(&conn, "n1"), 4_000);

        trash_note(&conn, "n1", "2026-03-10", 5_000).unwrap();
        assert_eq!(updated_ms_of(&conn, "n1"), 5_000);

        restore_note(&conn, "n1", 6_000).unwrap();
        assert_eq!(updated_ms_of(&conn, "n1"), 6_000);

        upsert_note(&conn, &note("n1", "2026-03-11"), 7_000).unwrap();
        assert_eq!(updated_ms_of(&conn, "n1"), 7_000);
    }

    #[test]
    fn deleting_a_tag_stamps_the_notes_it_reassigns() {
        let conn = fresh_db();
        insert_folder(&conn, "Work", "#000", 1_000).unwrap();
        insert_folder(&conn, "Personal", "#111", 1_000).unwrap();
        let mut n = note("n1", "2026-03-09");
        n.folder = "Work".into();
        insert_note(&conn, &n, 1_000).unwrap();

        delete_folder(&conn, "Work", "Personal", 9_000).unwrap();

        assert_eq!(updated_ms_of(&conn, "n1"), 9_000);
        let folder: String = conn
            .query_row("SELECT folder FROM notes WHERE id = 'n1'", [], |r| r.get(0))
            .unwrap();
        assert_eq!(folder, "Personal");
    }

    // ── Tombstones ──────────────────────────────────────────────────────────

    #[test]
    fn purging_a_note_records_a_tombstone() {
        let conn = fresh_db();
        insert_note(&conn, &note("n1", "2026-03-09"), 1_000).unwrap();
        trash_note(&conn, "n1", "2026-03-10", 2_000).unwrap();

        purge_note(&conn, "n1", 3_000).unwrap();

        assert_eq!(tombstone_of(&conn, "n1"), Some(3_000));
        assert!(get_note(&conn, "n1").unwrap().is_none());
    }

    #[test]
    fn emptying_the_trash_tombstones_every_purged_note() {
        let conn = fresh_db();
        for id in ["n1", "n2", "n3"] {
            insert_note(&conn, &note(id, "2026-03-09"), 1_000).unwrap();
        }
        trash_note(&conn, "n1", "2026-03-10", 2_000).unwrap();
        trash_note(&conn, "n2", "2026-03-10", 2_000).unwrap();

        empty_trash(&conn, 5_000).unwrap();

        assert_eq!(tombstone_of(&conn, "n1"), Some(5_000));
        assert_eq!(tombstone_of(&conn, "n2"), Some(5_000));
        // The untrashed note is untouched and leaves no marker.
        assert_eq!(tombstone_of(&conn, "n3"), None);
        assert!(get_note(&conn, "n3").unwrap().is_some());
    }

    #[test]
    fn trash_expiry_tombstones_only_what_it_purges() {
        let conn = fresh_db();
        insert_note(&conn, &note("old", "2026-01-01"), 1_000).unwrap();
        insert_note(&conn, &note("recent", "2026-03-09"), 1_000).unwrap();
        trash_note(&conn, "old", "2026-01-02", 1_000).unwrap();
        trash_note(&conn, "recent", "2026-03-10", 1_000).unwrap();

        let purged = purge_expired(&conn, "2026-02-01", 7_000).unwrap();

        assert_eq!(purged, 1);
        assert_eq!(tombstone_of(&conn, "old"), Some(7_000));
        assert_eq!(tombstone_of(&conn, "recent"), None);
    }

    #[test]
    fn trashing_a_note_leaves_no_tombstone() {
        let conn = fresh_db();
        insert_note(&conn, &note("n1", "2026-03-09"), 1_000).unwrap();

        trash_note(&conn, "n1", "2026-03-10", 2_000).unwrap();

        // A trashed note is still a row, so it syncs as a note carrying its
        // deletedAt — only a permanent purge needs a marker.
        assert_eq!(tombstone_of(&conn, "n1"), None);
    }

    #[test]
    fn reimporting_a_purged_note_clears_its_tombstone() {
        let conn = fresh_db();
        insert_note(&conn, &note("n1", "2026-03-09"), 1_000).unwrap();
        purge_note(&conn, "n1", 2_000).unwrap();
        assert_eq!(tombstone_of(&conn, "n1"), Some(2_000));

        upsert_note(&conn, &note("n1", "2026-03-09"), 3_000).unwrap();

        assert_eq!(tombstone_of(&conn, "n1"), None);
        assert!(get_note(&conn, "n1").unwrap().is_some());
    }

    // ── Tag list timestamp ──────────────────────────────────────────────────

    #[test]
    fn tag_list_changes_advance_the_timestamp() {
        let conn = fresh_db();
        assert_eq!(folders_updated_ms(&conn).unwrap(), 0);

        insert_folder(&conn, "Work", "#000", 1_000).unwrap();
        assert_eq!(folders_updated_ms(&conn).unwrap(), 1_000);

        insert_folder(&conn, "Personal", "#111", 2_000).unwrap();
        assert_eq!(folders_updated_ms(&conn).unwrap(), 2_000);

        delete_folder(&conn, "Work", "Personal", 3_000).unwrap();
        assert_eq!(folders_updated_ms(&conn).unwrap(), 3_000);
    }

    #[test]
    fn stopping_sync_forgets_the_folder_but_keeps_every_note_and_tag() {
        let conn = fresh_db();
        insert_folder(&conn, "Work", "#000", 1).unwrap();
        insert_note(&conn, &note("n1", "2026-03-09"), 1_000).unwrap();
        purge_note(&conn, "n1", 2_000).unwrap();
        insert_note(&conn, &note("n2", "2026-03-09"), 3_000).unwrap();
        set_sync_folder(&conn, "/home/u/Folioo").unwrap();
        set_sync_state(&conn, "n2", SyncState { synced_ms: 3_000, file_size: 9, file_mtime_ms: 9 }).unwrap();
        set_tags_synced_ms(&conn, 1).unwrap();

        clear_sync(&conn).unwrap();

        assert_eq!(sync_folder(&conn).unwrap(), None);
        assert!(sync_states(&conn).unwrap().is_empty());
        assert_eq!(tags_synced_ms(&conn).unwrap(), None);
        assert!(get_note(&conn, "n2").unwrap().is_some());
        assert_eq!(list_folders(&conn).unwrap().len(), 1);
        // Local tombstones stay: they still describe what the user deleted.
        assert_eq!(tombstone_of(&conn, "n1"), Some(2_000));
    }

    #[test]
    fn importing_an_existing_tag_does_not_advance_the_timestamp() {
        let conn = fresh_db();
        upsert_folder_ignore(&conn, "Work", "#000", 1_000).unwrap();
        assert_eq!(folders_updated_ms(&conn).unwrap(), 1_000);

        // Same name again: nothing lands, so the list did not change.
        upsert_folder_ignore(&conn, "Work", "#999", 2_000).unwrap();
        assert_eq!(folders_updated_ms(&conn).unwrap(), 1_000);
    }

    #[test]
    fn drops_tags_and_their_attributes() {
        let html = r#"<div>Hola <strong>mundo</strong></div><img src="data:image/png;base64,AAAA…">"#;
        assert_eq!(strip_html(html), "Hola mundo");
    }

    #[test]
    fn separates_adjacent_blocks_and_collapses_whitespace() {
        assert_eq!(strip_html("<div>uno</div><div>dos</div>"), "uno dos");
        assert_eq!(strip_html("a\n\n  b"), "a b");
    }

    #[test]
    fn decodes_common_entities() {
        assert_eq!(strip_html("a&nbsp;b &amp; c &lt;d&gt;"), "a b & c <d>");
        assert_eq!(strip_html("tom&yerry"), "tom&yerry");
    }
}
