## 1. Schema foundation (shippable on its own, no sync code reachable)

- [x] 1.1 Add `updated_ms INTEGER` to `notes` in `db::init`, guarded by the same
      `pragma_table_info` idempotency check used for `deleted_at` / `content_text`
- [x] 1.2 Backfill `updated_ms` for existing rows from each row's `updated` date (midnight UTC
      of that day), in the same migration step that adds the column
- [x] 1.3 Create the `deletions (id TEXT PRIMARY KEY, deleted_ms INTEGER NOT NULL)` table if absent
- [x] 1.4 Add a `now_ms()` helper in `lib.rs` beside `today_iso()`, derived from `SystemTime`
      with no external date crate
- [x] 1.5 Stamp `updated_ms` in every note write path: `insert_note`, `update_note`,
      `update_note_folder`, `toggle_favorite`, `trash_note`, `restore_note`, `upsert_note`
- [x] 1.6 Write tombstones inside the same transaction as the row removal in `purge_note`,
      `empty_trash` and `purge_expired`
- [x] 1.7 Record a tag-list modification timestamp, updated by `insert_folder`, `delete_folder`
      and any color change
- [x] 1.8 Unit tests: migrating a pre-`updated_ms` database backfills without touching content;
      purging writes a tombstone; each write path advances `updated_ms`
- [x] 1.9 Verify `cargo test` passes and the app still launches against an existing `notes.db`

## 2. Library format

- [x] 2.1 Create `src-tauri/src/sync/mod.rs` with a `library` submodule; define the note file,
      tombstone file, `tags.json` and `library.json` formats (serde), versioned by `format: 1`
- [x] 2.2 Resolve the library directory from a chosen folder: `<chosen>/Folioo/`, or the chosen
      folder itself when it already contains `library.json`
- [x] 2.3 Initialise a library (marker, `notes/`, `deleted/`) — only ever called from the
      "choose folder" command, never from a background sync
- [x] 2.4 Refuse a library whose `format` is newer than supported; report a previously synced
      library whose marker is missing as unavailable
- [x] 2.5 Atomic writes: temp file in the same directory, then rename over the target
- [x] 2.6 List `notes/` and `deleted/` with size + mtime; recognise canonical `<id>.json` names
      and classify every other `.json` in `notes/` as a candidate sync-tool conflict file
- [x] 2.7 Tests against temp directories: layout resolution, init, newer-format refusal,
      missing-marker detection, atomic write leaves no partial file, name classification

## 3. Sync engine

- [ ] 3.1 Add a per-note sync record (`synced_ms`, file size, file mtime) and its idempotent
      migration; store the chosen folder path in `meta`
- [ ] 3.2 Implement the classification pass over local ids, note files and tombstone files,
      parsing only files whose size or mtime changed since the last pass
- [ ] 3.3 Write / read notes, committing each one independently so an interrupted pass resumes
- [ ] 3.4 Enforce "absence never deletes": a local note with no file is written, never removed
- [ ] 3.5 Conflicts: later `updated_ms` stays; the other copy becomes a new note titled with the
      English `(conflicted copy)` marker, in the same tag, and is written to the library
- [ ] 3.6 Import sync-tool conflict files that parse as notes as conflicted copies, then remove
      them; leave files that don't parse untouched
- [ ] 3.7 Tombstones: write a file for each local purge and remove the note file; purge locally
      for a newer tombstone file; let a newer local edit win over an older tombstone
- [ ] 3.8 Tags: whole-list last-writer-wins via `tags.json` and `folders_updated_ms`; afterwards
      create any tag a note references that the list lacks
- [ ] 3.9 Prune tombstones (local and files) older than 90 days
- [ ] 3.10 Tests for every branch: two-sided edit keeps two notes; unmounted (empty) folder
      deletes and writes nothing; hand-deleted note file is rewritten; purge on A removes the
      note on B; edit after purge wins; trashed notes round-trip with `deletedAt`; sync-tool
      conflict file becomes a note; tag deletion propagates; orphan tag is recreated; a second
      pass over an unchanged library parses no files

## 4. Commands, scheduling and events

- [ ] 4.1 Add `sync_status`, `sync_set_folder`, `sync_stop` and `sync_now` commands and register
      them; `sync_set_folder` initialises or adopts the library, then runs a first pass
- [ ] 4.2 Hold sync state (in progress, last success, last error, folder availability) in
      `AppState` behind a flag so only one pass is ever in flight
- [ ] 4.3 Run a pass on startup when a folder is set, off the UI thread
- [ ] 4.4 Debounce a pass after local edits settle, tuned not to chase autosave keystrokes
- [ ] 4.5 Run a pass every 5 minutes while the app is open
- [ ] 4.6 Emit a Tauri event when a pass finishes so the frontend reloads notes, trash and tags
- [ ] 4.7 Verify the editor stays responsive during a pass over an image-heavy library

## 5. Frontend

- [ ] 5.1 Add the invoke wrappers to `src/api.js`
- [ ] 5.2 Add the "Sync" section to `src/settings.jsx`: "Choose folder…" (dialog plugin, directory
      mode), the hint naming rclone / Syncthing / Nextcloud / Dropbox, the folder path, the
      state line (idle with last-sync time, in progress, unavailable, failed with reason), "Sync
      now" disabled while a pass runs, and "Stop syncing"
- [ ] 5.3 Confirm "Stop syncing", stating that local notes and the folder's contents are kept
- [ ] 5.4 Subscribe to the sync-finished event in `src/App.jsx` and reload the lists
- [ ] 5.5 Style the section consistently with Appearance and Data in `src/notes.css`, verified in
      all three variants and in light and dark
- [ ] 5.6 Confirm problems surface in Settings only — never as a modal while writing

## 6. Flatpak

- [ ] 6.1 Build the Flatpak, choose a folder through the portal, relaunch, and check the folder is
      still reachable
- [ ] 6.2 If the portal grant does not persist, add `--filesystem=home` to
      `flatpak/org.folioo.app.yml` with a comment explaining why

## 7. Verification

- [ ] 7.1 Two-profile run-through (two app-data directories sharing one folder, or two machines
      over Syncthing): create, edit, trash, restore, purge and retag on each side, and check
      convergence in both directions
- [ ] 7.2 Unmounted-folder run-through: point Folioo at an rclone mount, unmount it, sync, and
      confirm nothing local is deleted and Settings reports the folder unavailable
- [ ] 7.3 Offline run-through: full local use with the folder unreachable, then a successful pass
- [ ] 7.4 Choosing a folder that already holds another machine's library merges both sides and
      deletes nothing
- [ ] 7.5 Update `README.md`: how folder sync works, example setups with rclone (Google Drive)
      and Syncthing, that deleting files by hand in the folder does not delete notes, and why
      Folioo integrates no cloud provider directly

## 8. Follow-ups deliberately excluded

- Direct cloud provider integration (Google Drive, Dropbox, OneDrive, iCloud).
- Encryption of the library.
- Extracting embedded base64 images into separate files.
- A file-watcher on the library instead of the 5-minute interval.
