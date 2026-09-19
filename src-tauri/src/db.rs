// db.rs — SQLite schema, queries, and first-run seeding.

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

    // Covers list_notes, list_trash and purge_expired.
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_notes_deleted_updated
         ON notes (deleted_at, updated DESC)",
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
pub fn upsert_note(conn: &Connection, note: &Note) -> Result<()> {
    conn.execute(
        "INSERT INTO notes (id, title, content, content_text, folder, favorite, created, updated, deleted_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
         ON CONFLICT(id) DO UPDATE SET
            title=?2, content=?3, content_text=?4, folder=?5, favorite=?6, created=?7, updated=?8, deleted_at=?9",
        params![
            note.id,
            note.title,
            note.content,
            strip_html(&note.content),
            note.folder,
            note.favorite as i64,
            note.created,
            note.updated,
            note.deleted_at
        ],
    )?;
    Ok(())
}

pub fn insert_note(conn: &Connection, note: &Note) -> Result<()> {
    conn.execute(
        "INSERT INTO notes (id, title, content, content_text, folder, favorite, created, updated)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            note.id,
            note.title,
            note.content,
            strip_html(&note.content),
            note.folder,
            note.favorite as i64,
            note.created,
            note.updated
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
) -> Result<()> {
    conn.execute(
        "UPDATE notes SET title = ?2, content = ?3, content_text = ?4, updated = ?5 WHERE id = ?1",
        params![id, title, content, strip_html(content), updated],
    )?;
    Ok(())
}

/// Reassign a note to a different tag/folder.
pub fn update_note_folder(conn: &Connection, id: &str, folder: &str, updated: &str) -> Result<()> {
    conn.execute(
        "UPDATE notes SET folder = ?2, updated = ?3 WHERE id = ?1",
        params![id, folder, updated],
    )?;
    Ok(())
}

/// Move a note to the trash (soft delete).
pub fn trash_note(conn: &Connection, id: &str, when: &str) -> Result<()> {
    conn.execute(
        "UPDATE notes SET deleted_at = ?2 WHERE id = ?1",
        params![id, when],
    )?;
    Ok(())
}

/// Restore a note from the trash.
pub fn restore_note(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("UPDATE notes SET deleted_at = NULL WHERE id = ?1", params![id])?;
    Ok(())
}

/// Permanently remove a single note.
pub fn purge_note(conn: &Connection, id: &str) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
    Ok(())
}

/// Permanently remove every trashed note.
pub fn empty_trash(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM notes WHERE deleted_at IS NOT NULL", [])?;
    Ok(())
}

/// Drop trashed notes whose deletion date is strictly before `cutoff` (YYYY-MM-DD).
pub fn purge_expired(conn: &Connection, cutoff: &str) -> Result<usize> {
    let n = conn.execute(
        "DELETE FROM notes WHERE deleted_at IS NOT NULL AND deleted_at < ?1",
        params![cutoff],
    )?;
    Ok(n)
}

/// Flip a note's favorite flag and return the new value.
pub fn toggle_favorite(conn: &Connection, id: &str) -> Result<bool> {
    let fav: i64 = conn.query_row(
        "UPDATE notes SET favorite = 1 - favorite WHERE id = ?1 RETURNING favorite",
        params![id],
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

pub fn insert_folder(conn: &Connection, name: &str, color: &str) -> Result<()> {
    let next_pos: i64 =
        conn.query_row("SELECT COALESCE(MAX(position), 0) + 1 FROM folders", [], |r| {
            r.get(0)
        })?;
    conn.execute(
        "INSERT INTO folders (name, color, position) VALUES (?1, ?2, ?3)",
        params![name, color, next_pos],
    )?;
    Ok(())
}

/// Insert a folder only if its name doesn't already exist (used when importing).
pub fn upsert_folder_ignore(conn: &Connection, name: &str, color: &str) -> Result<()> {
    let next_pos: i64 =
        conn.query_row("SELECT COALESCE(MAX(position), 0) + 1 FROM folders", [], |r| r.get(0))?;
    conn.execute(
        "INSERT OR IGNORE INTO folders (name, color, position) VALUES (?1, ?2, ?3)",
        params![name, color, next_pos],
    )?;
    Ok(())
}

/// Populate the default tag set the first time the app runs.
pub fn seed_folders_if_empty(conn: &Connection) -> Result<()> {
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM folders", [], |r| r.get(0))?;
    if count > 0 {
        return Ok(());
    }
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
pub fn delete_folder(conn: &Connection, name: &str, fallback: &str) -> Result<()> {
    conn.execute(
        "UPDATE notes SET folder = ?2 WHERE folder = ?1",
        params![name, fallback],
    )?;
    conn.execute("DELETE FROM folders WHERE name = ?1", params![name])?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::strip_html;

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
