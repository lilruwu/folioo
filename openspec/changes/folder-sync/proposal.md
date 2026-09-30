## Why

Folioo's notes live in a single SQLite file on one machine. The only way to move them is a
manual JSON export/import, so a second computer, a reinstall or a dead disk means tedious
bookkeeping or lost notes.

The obvious fix — integrating a cloud provider — means registering Folioo with that provider,
publishing a homepage and a privacy policy, shipping client credentials in the binary, and
maintaining all of it. Folioo will not run a cloud of its own or on anyone's behalf. Instead it
syncs with **a folder the user chooses**, and the user moves that folder between machines with
whatever they already trust: rclone (which reaches Google Drive, OneDrive, S3 and dozens more),
Syncthing, Nextcloud, Dropbox, or iCloud Drive on a Mac. This is how Joplin's "file system"
target works, and it keeps Folioo entirely offline.

## What Changes

- **Choose a sync folder** in Settings. Folioo keeps a library inside it (`Folioo/`), marked by
  a `library.json` file, so it never mixes its files with whatever else lives in that folder.
- **Bidirectional sync** of notes (active and trashed) and tags between the local database and
  the library: one JSON file per note, one file for the tag list, written atomically so a sync
  tool never picks up a half-written file.
- **Conflict resolution** by last-writer-wins on the millisecond modification timestamp. When
  both sides changed a note since the last sync, the loser is **kept as a copy** — never
  discarded. Conflict files created by the sync tools themselves (Syncthing's
  `.sync-conflict-…`, Dropbox's "conflicted copy") are imported the same way.
- **Deletions propagate only through explicit tombstones.** A note file that is simply missing
  never deletes anything: an unmounted rclone folder or an empty drive must not look like "the
  user deleted everything". If the library marker itself is missing, sync stops and says so.
- **Sync status in Settings**: the chosen folder, last sync time, in-progress/error state, a
  manual "Sync now", and "Stop syncing" (which forgets the folder and leaves both the local
  notes and the folder's contents untouched).
- **No network, no accounts, no credentials.** Folioo reads and writes local files only.
- **BREAKING (storage-internal, already shipped in phase 1):** the `notes` table has a
  millisecond `updated_ms` column, a `deletions` tombstone table and a `meta` table. Existing
  databases migrate in place without data loss; a database written by this version is not
  readable by older builds.

## Capabilities

### New Capabilities
- `folder-sync`: choosing a sync folder, the library layout and file formats, the sync engine
  (push/pull, conflict resolution, tombstones, tag sync, sync-tool conflict copies,
  scheduling), and the safety rules that keep a missing or unmounted folder from deleting data.

### Modified Capabilities
- `persistence`: notes gain a millisecond modification timestamp alongside the existing
  `YYYY-MM-DD` dates, purged notes leave tombstones instead of vanishing, and tag changes
  become observable to the sync engine.
- `settings`: Settings gains a "Sync" section (choose folder, status, sync now, stop syncing)
  alongside the existing Appearance, Typography and Data sections.

## Impact

- **`src-tauri/src/db.rs`:** schema migration (`updated_ms`, `deletions`, `meta`) and stamping
  in every write path — **already implemented** as phase 1 — plus per-note sync state.
- **`src-tauri/src/lib.rs`:** new commands (`sync_status`, `sync_set_folder`, `sync_stop`,
  `sync_now`) and a new `sync` module holding the library format and the engine.
- **`src/settings.jsx` / `src/api.js` / `src/App.jsx`:** the Sync section, its invoke wrappers,
  and refreshing the lists when a sync finishes.
- **Dependencies:** none new — folder selection reuses the existing dialog plugin, and file
  access is the standard library.
- **Flatpak:** the chosen folder must stay reachable across launches from inside the sandbox;
  via the document portal if its grant persists, else a filesystem permission.
- **Offline behaviour:** sync is strictly additive to local-first storage. With no folder set,
  or the folder unavailable, Folioo keeps working exactly as it does now.
