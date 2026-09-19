## Why

Folioo's notes live in a single SQLite file on one machine. The only way to move them is a
manual JSON export/import, so a second computer, a reinstall or a dead disk means either
tedious bookkeeping or lost notes. Users expect a notes app to follow them across machines,
and that expectation is the main thing separating Folioo from the Apple Notes experience it
is modelled on.

Google Drive is the one mainstream consumer cloud with a documented, stable API that a Linux
desktop app can talk to directly. **iCloud is explicitly out of scope**: Apple publishes no
API that lets a third-party app on Linux read or write iCloud Drive. CloudKit Web Services
requires a paid Apple Developer membership, a container provisioned from the CloudKit
dashboard, and user sign-in through Apple's JS widget on registered web origins — none of
which a Linux-only Tauri app can rely on.

## What Changes

- **Sign in with Google.** A "Conectar con Google" action in Configuración runs an OAuth 2.0
  authorization-code flow with PKCE: Folioo opens the user's **system browser** (Google
  refuses OAuth inside embedded webviews) and catches the redirect on a loopback listener.
  The user sees nothing but a normal Google login and consent screen.
- **Narrow permission.** Folioo requests only `drive.file`, which grants access to the files
  the app itself creates — never the rest of the user's Drive. It is a non-sensitive scope,
  so it needs no Google verification review and shows no "unverified app" interstitial.
- **Bidirectional sync** of notes (active and trashed) and tags against a Folioo-owned folder
  in the user's Drive, stored as one JSON file per note plus one for tags. Sync runs on
  launch, on demand, and debounced after edits.
- **Conflict resolution** by last-writer-wins on a new millisecond-precision modification
  timestamp. When both sides changed a note since the last sync, the loser is **kept as a
  copy** rather than discarded, so no edit is ever silently destroyed.
- **Deletion propagation** via tombstones, so emptying the trash on one machine doesn't see
  the notes resurrected by the next sync from another.
- **BREAKING (storage-internal):** the `notes` table gains a millisecond `updated_ms` column
  and a `deletions` tombstone table. Existing databases migrate in place — `updated_ms` is
  backfilled from the existing day-granular `updated` date. No user-visible data is lost, but
  a database written by this version is not readable by older builds.
- **Sync status in the UI**: connected account, last sync time, in-progress/error state, a
  manual "Sincronizar ahora", and "Desconectar" (which revokes the token and forgets it).
- **Credentials.** The app ships a build-time OAuth client ID for the Folioo project. It is
  a public client per RFC 8252 — PKCE, not a secret, is what protects the flow.

## Capabilities

### New Capabilities
- `cloud-sync`: connecting a Google account, the sync engine (push/pull, conflict resolution,
  deletion propagation, scheduling), the on-disk remote format, and credential storage.

### Modified Capabilities
- `persistence`: notes gain a millisecond modification timestamp alongside the existing
  `YYYY-MM-DD` dates, purged notes leave tombstones instead of vanishing, and tag changes
  become observable to the sync engine.
- `settings`: Configuración gains a "Sincronización" section (connect/disconnect, status,
  manual sync) alongside the existing Apariencia and Datos sections.

## Impact

- **New Rust dependencies:** an HTTPS client with rustls (`reqwest`), an OS keyring binding
  (`keyring`) for the refresh token, a tiny loopback HTTP listener for the OAuth redirect, and
  `tauri-plugin-opener` to hand the authorization URL to the system browser.
- **`src-tauri/src/db.rs`:** schema migration (`updated_ms`, `deletions`), and every write path
  updated to stamp the new timestamp.
- **`src-tauri/src/lib.rs`:** new commands (`sync_status`, `sync_connect`, `sync_disconnect`,
  `sync_now`), plus a new `sync` module.
- **`src/settings.jsx` / `src/api.js`:** the Sincronización panel and its invoke wrappers.
- **Flatpak manifest:** `--talk-name=org.freedesktop.secrets` for the keyring; network access
  is already granted.
- **Security surface:** all token handling and Drive traffic stay in Rust — no access token
  ever reaches the webview, and the CSP stays as restrictive as it is today.
- **Offline behaviour:** sync is strictly additive to local-first storage. With no network or
  no account connected, Folioo keeps working exactly as it does now.
