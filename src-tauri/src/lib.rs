// lib.rs — Folioo backend.
// Persists notes in a local SQLite database and exposes CRUD commands to the
// React frontend over Tauri's IPC bridge.

use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use tauri::Manager;

mod db;

/// Days a note stays in the trash before it is purged automatically.
pub const TRASH_RETENTION_DAYS: i64 = 30;

/// A single note. Field names match the shape the frontend expects.
/// `deleted_at` is `None` for active notes and an ISO date for trashed ones.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub content: String,
    pub folder: String,
    pub favorite: bool,
    pub created: String,
    pub updated: String,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<String>,
}

/// Lightweight listing row: everything the note list and search need, but not
/// the HTML `content` (which may embed base64 images and dwarf the rest).
/// `search_text` is the plain-text shadow of the content, kept by the DB layer.
#[derive(Debug, Clone, Serialize)]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub folder: String,
    pub favorite: bool,
    pub created: String,
    pub updated: String,
    #[serde(rename = "deletedAt")]
    pub deleted_at: Option<String>,
    #[serde(rename = "searchText")]
    pub search_text: String,
}

/// A tag/folder a note can belong to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Folder {
    pub name: String,
    pub color: String,
}

/// Shared application state: a single SQLite connection behind a mutex.
pub struct AppState {
    pub db: Mutex<Connection>,
}

// ── Date / id helpers ───────────────────────────────────────────────────────

/// Whole days elapsed since the Unix epoch (UTC).
fn epoch_days() -> i64 {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    secs / 86_400
}

/// Format a day count (days since epoch) as an ISO `YYYY-MM-DD` string.
/// Uses Howard Hinnant's civil-from-days algorithm — no external date crate.
fn iso_from_days(days: i64) -> String {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as i64; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Today's date as `YYYY-MM-DD`.
pub fn today_iso() -> String {
    iso_from_days(epoch_days())
}

/// A date `days_ago` days before today, as `YYYY-MM-DD`.
pub fn iso_offset(days_ago: i64) -> String {
    iso_from_days(epoch_days() - days_ago)
}

/// A reasonably unique id derived from the current time in nanoseconds.
fn generate_id() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("n{:x}", nanos)
}

// ── Commands ────────────────────────────────────────────────────────────────

#[tauri::command]
fn list_notes(state: tauri::State<'_, AppState>) -> Result<Vec<NoteSummary>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_notes(&conn).map_err(|e| e.to_string())
}

/// Full note (including content) — fetched lazily when a note is opened.
#[tauri::command]
fn get_note(id: String, state: tauri::State<'_, AppState>) -> Result<Note, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::get_note(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Note {id} not found"))
}

#[tauri::command]
fn create_note(folder: String, state: tauri::State<'_, AppState>) -> Result<Note, String> {
    let today = today_iso();
    let note = Note {
        id: generate_id(),
        title: String::new(),
        content: String::new(),
        folder,
        favorite: false,
        created: today.clone(),
        updated: today,
        deleted_at: None,
    };
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::insert_note(&conn, &note).map_err(|e| e.to_string())?;
    Ok(note)
}

#[tauri::command]
fn update_note(
    id: String,
    title: String,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<NoteSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_note(&conn, &id, &title, &content, &today_iso()).map_err(|e| e.to_string())?;
    db::get_summary(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Note {id} not found"))
}

/// Move a note to the trash (recoverable for `TRASH_RETENTION_DAYS`).
#[tauri::command]
fn delete_note(id: String, state: tauri::State<'_, AppState>) -> Result<NoteSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::trash_note(&conn, &id, &today_iso()).map_err(|e| e.to_string())?;
    db::get_summary(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Note {id} not found"))
}

#[tauri::command]
fn list_trash(state: tauri::State<'_, AppState>) -> Result<Vec<NoteSummary>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_trash(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn restore_note(id: String, state: tauri::State<'_, AppState>) -> Result<NoteSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::restore_note(&conn, &id).map_err(|e| e.to_string())?;
    db::get_summary(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Note {id} not found"))
}

/// Permanently delete a single trashed note.
#[tauri::command]
fn purge_note(id: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::purge_note(&conn, &id).map_err(|e| e.to_string())
}

/// Permanently delete everything in the trash.
#[tauri::command]
fn empty_trash(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::empty_trash(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn toggle_favorite(id: String, state: tauri::State<'_, AppState>) -> Result<bool, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::toggle_favorite(&conn, &id).map_err(|e| e.to_string())
}

#[tauri::command]
fn list_folders(state: tauri::State<'_, AppState>) -> Result<Vec<Folder>, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::list_folders(&conn).map_err(|e| e.to_string())
}

#[tauri::command]
fn create_folder(
    name: String,
    color: String,
    state: tauri::State<'_, AppState>,
) -> Result<Folder, String> {
    let name = name.trim().to_string();
    if name.is_empty() {
        return Err("The tag name cannot be empty".into());
    }
    if name.chars().count() > 24 {
        return Err("The name is too long (max. 24 characters)".into());
    }
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if db::folder_exists(&conn, &name).map_err(|e| e.to_string())? {
        return Err(format!("The tag \"{name}\" already exists"));
    }
    db::insert_folder(&conn, &name, &color).map_err(|e| e.to_string())?;
    Ok(Folder { name, color })
}

/// Result of deleting a tag: the tag any orphaned notes were moved to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFolderResult {
    pub fallback: String,
}

#[tauri::command]
fn delete_folder(
    name: String,
    state: tauri::State<'_, AppState>,
) -> Result<DeleteFolderResult, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    if db::folder_count(&conn).map_err(|e| e.to_string())? <= 1 {
        return Err("At least one tag must exist".into());
    }
    let fallback = db::first_folder_except(&conn, &name)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "There is no other tag to move the notes to".to_string())?;
    db::delete_folder(&conn, &name, &fallback).map_err(|e| e.to_string())?;
    Ok(DeleteFolderResult { fallback })
}

#[tauri::command]
fn set_note_folder(
    id: String,
    folder: String,
    state: tauri::State<'_, AppState>,
) -> Result<NoteSummary, String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    db::update_note_folder(&conn, &id, &folder, &today_iso()).map_err(|e| e.to_string())?;
    db::get_summary(&conn, &id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| format!("Note {id} not found"))
}

// ── Export / import ─────────────────────────────────────────────────────────

#[derive(Debug, Serialize, Deserialize)]
struct ExportData {
    app: String,
    version: u32,
    folders: Vec<Folder>,
    notes: Vec<Note>,
}

/// Write all folders + notes to `path` as a JSON backup.
#[tauri::command]
fn export_to_path(path: String, state: tauri::State<'_, AppState>) -> Result<(), String> {
    let conn = state.db.lock().map_err(|e| e.to_string())?;
    let data = ExportData {
        app: "folioo".into(),
        version: 1,
        folders: db::list_folders(&conn).map_err(|e| e.to_string())?,
        notes: db::list_all_notes(&conn).map_err(|e| e.to_string())?,
    };
    let json = serde_json::to_string_pretty(&data).map_err(|e| e.to_string())?;
    std::fs::write(&path, json).map_err(|e| e.to_string())?;
    Ok(())
}

/// Merge a JSON backup into the database (upsert by id / tag name). Returns the
/// number of notes imported.
#[tauri::command]
fn import_from_path(path: String, state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let json = std::fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let data: ExportData =
        serde_json::from_str(&json).map_err(|_| "The file is not a valid backup".to_string())?;
    let mut guard = state.db.lock().map_err(|e| e.to_string())?;
    let tx = guard.transaction().map_err(|e| e.to_string())?;
    for f in &data.folders {
        db::upsert_folder_ignore(&tx, &f.name, &f.color).map_err(|e| e.to_string())?;
    }
    for n in &data.notes {
        db::upsert_note(&tx, n).map_err(|e| e.to_string())?;
    }
    tx.commit().map_err(|e| e.to_string())?;
    Ok(data.notes.len())
}

// ── Legacy data migration ───────────────────────────────────────────────────

/// Bundle identifier used before the app was renamed to Folioo.
const LEGACY_IDENTIFIER: &str = "org.linuxnotes.app";
const IDENTIFIER: &str = "org.folioo.app";

/// Adopt the database left behind by the pre-rename app, if any.
///
/// The app-data directory is derived from the bundle identifier, so renaming
/// the app points it at a brand-new (empty) directory while the user's notes
/// still sit in the old one. Swapping the identifier inside the resolved path
/// finds the old directory for a plain install (`~/.local/share/<id>`).
///
/// It does NOT rescue a Flatpak install: the old data lives in
/// `~/.var/app/org.linuxnotes.app/`, which this sandbox cannot read. Those
/// users have to export a backup from the old build and import it here.
///
/// The copy only happens when the new database does not exist yet, so it never
/// overwrites newer data and is a no-op on every launch after the first.
fn migrate_legacy_db(dir: &std::path::Path, db_path: &std::path::Path) {
    if db_path.exists() {
        return;
    }
    let Some(legacy_dir) = dir
        .to_str()
        .map(|p| p.replace(IDENTIFIER, LEGACY_IDENTIFIER))
        .map(std::path::PathBuf::from)
    else {
        return;
    };
    if legacy_dir == dir {
        return;
    }
    // SQLite may keep the WAL and shared-memory files alongside the database;
    // copy whichever exist so a non-checkpointed WAL isn't left behind.
    for suffix in ["", "-wal", "-shm"] {
        let name = format!("notes.db{suffix}");
        let from = legacy_dir.join(&name);
        if from.exists() {
            if let Err(e) = std::fs::copy(&from, dir.join(&name)) {
                eprintln!("Could not migrate {name} from the previous version: {e}");
            }
        }
    }
}

// ── Entry point ─────────────────────────────────────────────────────────────

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Store the database in the platform-appropriate app-data directory.
            let dir = app.path().app_data_dir()?;
            std::fs::create_dir_all(&dir)?;
            let db_path = dir.join("notes.db");
            // Carry over the notes of anyone upgrading from the Linux Notes era.
            migrate_legacy_db(&dir, &db_path);
            let conn = Connection::open(&db_path)?;
            db::init(&conn)?;
            db::seed_folders_if_empty(&conn)?;
            // Purge notes that have sat in the trash past the retention window.
            let _ = db::purge_expired(&conn, &iso_offset(TRASH_RETENTION_DAYS));
            app.manage(AppState { db: Mutex::new(conn) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_notes,
            get_note,
            create_note,
            update_note,
            delete_note,
            toggle_favorite,
            list_folders,
            create_folder,
            set_note_folder,
            delete_folder,
            list_trash,
            restore_note,
            purge_note,
            empty_trash,
            export_to_path,
            import_from_path
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
