// api.js — frontend ⇄ Rust bridge. Every call maps to a #[tauri::command]
// in src-tauri/src/lib.rs and ultimately to a SQLite query.
import { invoke } from "@tauri-apps/api/core";

// ── Notes ──
// Returns lightweight summaries (no content): id, title, folder, favorite,
// created, updated, deletedAt, searchText.
export function listNotes() {
  return invoke("list_notes");
}

// Full note, content included — fetched lazily when a note is opened.
export function getNote(id) {
  return invoke("get_note", { id });
}

export function createNote(folder) {
  return invoke("create_note", { folder });
}

export function updateNote(id, title, content) {
  return invoke("update_note", { id, title, content });
}

// Moves a note to the trash (recoverable for 30 days).
export function deleteNote(id) {
  return invoke("delete_note", { id });
}

export function listTrash() {
  return invoke("list_trash");
}

export function restoreNote(id) {
  return invoke("restore_note", { id });
}

// Permanently deletes a single trashed note.
export function purgeNote(id) {
  return invoke("purge_note", { id });
}

// Permanently empties the trash.
export function emptyTrash() {
  return invoke("empty_trash");
}

export function toggleFavorite(id) {
  return invoke("toggle_favorite", { id });
}

export function setNoteFolder(id, folder) {
  return invoke("set_note_folder", { id, folder });
}

// ── Tags / folders ──
export function listFolders() {
  return invoke("list_folders");
}

export function createFolder(name, color) {
  return invoke("create_folder", { name, color });
}

// Deletes a tag; returns { fallback } — the tag its notes were moved to.
export function deleteFolder(name) {
  return invoke("delete_folder", { name });
}

// ── Backup ──
export function exportToPath(path) {
  return invoke("export_to_path", { path });
}

export function importFromPath(path) {
  return invoke("import_from_path", { path }); // resolves to number of notes imported
}

// ── Folder sync ──
// Status: { folder, running, lastSyncMs, available, error }.
export function syncStatus() {
  return invoke("sync_status");
}

// Creates (or adopts) the library in `path`; rejects with a message if the
// folder can't be used. Resolves to the new status.
export function syncSetFolder(path) {
  return invoke("sync_set_folder", { path });
}

// Forgets the folder. Local notes and the folder's contents are kept.
export function syncStop() {
  return invoke("sync_stop");
}

export function syncNow() {
  return invoke("sync_now");
}
