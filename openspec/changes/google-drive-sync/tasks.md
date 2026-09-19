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

## 2. Dependencies and Flatpak permissions

- [ ] 2.1 Add `reqwest` (rustls, JSON; no default TLS, no blocking, no cookies), `keyring`, and
      `tauri-plugin-opener` to `src-tauri/Cargo.toml` with trimmed default features
- [ ] 2.2 Record release binary size before and after, to keep the low-memory release profile honest
- [ ] 2.3 Add `--talk-name=org.freedesktop.secrets` to `flatpak/org.folioo.app.yml`
- [ ] 2.4 Read the OAuth client ID in `build.rs` from an environment variable, falling back to
      the Folioo project ID when unset, so forks can build with their own credentials

## 3. OAuth: connecting an account

- [ ] 3.1 Create `src-tauri/src/sync/mod.rs` with `auth`, `drive` and `engine` submodules
- [ ] 3.2 Implement PKCE: random code verifier, S256 challenge, random `state`, fresh per attempt
- [ ] 3.3 Bind a loopback listener on `127.0.0.1:0`, build the authorization URL with the bound
      port as `redirect_uri`, and open it in the system browser via the opener plugin
- [ ] 3.4 Handle the redirect: reject on `state` mismatch, serve a small "puedes cerrar esta
      pestaña" page, then shut the listener down; time the attempt out and free the port
- [ ] 3.5 Exchange the code for tokens; fetch the account email for display
- [ ] 3.6 Store the refresh token via `keyring`, falling back to a `0600` file in app-data;
      hold access tokens in memory only
- [ ] 3.7 Implement refresh-on-401 with a single retry, and mark the account disconnected
      (without deleting local notes) when the refresh token is rejected
- [ ] 3.8 Implement disconnect: best-effort token revocation, erase stored token and folder id,
      leave local notes and the Drive folder untouched
- [ ] 3.9 Tests: `state` mismatch is rejected before any token exchange; a rejected refresh
      token surfaces as "reconnect" and purges nothing

## 4. Drive client

- [ ] 4.1 Find-or-create the `Folioo` folder; persist its id locally; recreate it when the id
      no longer resolves
- [ ] 4.2 Implement `list` (id, name, `modifiedTime`), `download`, `upload_new`,
      `update_existing` and `delete` against the Drive v3 API
- [ ] 4.3 Define the on-disk note payload (note fields incl. `deletedAt` and `updated_ms`) and
      the `folders.json` payload, both versioned
- [ ] 4.4 Map HTTP failures to typed errors distinguishing auth failure, transient network
      failure, and quota/rate limiting; back off and retry the transient ones
- [ ] 4.5 Tests against recorded API responses — no live Drive calls in the test suite

## 5. Sync engine

- [ ] 5.1 Add per-note sync state (remote file id, `synced_ms`) and its schema migration
- [ ] 5.2 Implement the classification pass: upload / download / conflict / delete-remote /
      drop-tombstone, over the union of local ids, remote ids and tombstones
- [ ] 5.3 Implement upload and download, committing each note independently so an interrupted
      pass resumes cleanly on the next run
- [ ] 5.4 Implement conflict resolution: later timestamp stays as the note, loser re-inserted
      under a fresh id with an English conflicted-copy title marker, then uploaded
- [ ] 5.5 Implement deletion propagation: tombstone newer than remote → delete remote; tombstone
      older → download and drop the tombstone
- [ ] 5.6 Merge `folders.json` by tag name, later timestamp winning on color
- [ ] 5.7 Create any tag referenced by a downloaded note that exists on neither side
- [ ] 5.8 Prune tombstones older than 90 days
- [ ] 5.9 Tests for each branch: both-changed keeps two notes; purge-then-sync removes the
      remote file; remote-edit-after-local-purge restores the note; trashed notes round-trip
      with `deletedAt` intact; a note referencing an unknown tag creates it

## 6. Commands, scheduling and events

- [ ] 6.1 Add the `sync_status`, `sync_connect`, `sync_disconnect` and `sync_now` commands and
      register them in the invoke handler
- [ ] 6.2 Hold sync state in `AppState` behind a flag so only one sync is ever in flight
- [ ] 6.3 Run a sync on startup when an account is connected, off the UI thread
- [ ] 6.4 Debounce a sync after local edits settle, tuned so it doesn't chase autosave keystrokes
- [ ] 6.5 Emit a Tauri event when a sync finishes so the frontend refreshes notes, trash and tags
- [ ] 6.6 Verify the editor stays responsive while a sync of an image-heavy corpus runs

## 7. Frontend

- [ ] 7.1 Add the invoke wrappers to `src/api.js`
- [ ] 7.2 Add the "Sincronización" section to `src/settings.jsx`: connect / connected email +
      disconnect, state line (idle with last-sync time, in progress, failed with reason), and
      "Sincronizar ahora" disabled while a sync runs
- [ ] 7.3 Confirm disconnection, stating that local notes are kept
- [ ] 7.4 Subscribe to the sync-finished event and refresh the lists in `src/App.jsx`
- [ ] 7.5 Style the section consistently with Apariencia and Datos in `src/notes.css`, verified
      in all three variants and in light/dark
- [ ] 7.6 Confirm failures surface in Configuración only — never as a modal while writing

## 8. Verification

- [ ] 8.1 Two-machine (or two-profile) run-through: create, edit, trash, restore, purge and
      retag on each side, verifying convergence in both directions
- [ ] 8.2 Offline run-through: full local use with no network, then a successful sync afterwards
- [ ] 8.3 Verify no Google origin is ever requested from the webview and the CSP is unchanged
- [ ] 8.4 Verify the whole flow inside an actual Flatpak build — keyring over the portal and
      the browser handoff, not just `tauri dev`
- [ ] 8.5 Verify a first sync against an account that has never synced is upload-only
- [ ] 8.6 Update `README.md`: what syncs, the `drive.file` scope, where credentials live, and
      that iCloud is not supported and why

## 9. Follow-ups deliberately excluded

- Payload encryption before upload (decided against for this change; see design.md).
- Extracting embedded base64 images into separate Drive files.
