## Context

Folioo today is strictly local-first: a Tauri 2 shell, a React 18 frontend that reaches the
backend only through `invoke()`, and a SQLite database (`rusqlite`, bundled) in the platform
app-data directory. The frontend owns no persistence beyond appearance preferences in
`localStorage`, and the webview runs under a tight CSP (`default-src 'self'`) with no network
egress at all — the Lora font is self-hosted precisely to keep it that way.

Two properties of the current schema shape this design:

- **Timestamps are day-granular.** `created`, `updated` and `deleted_at` are `YYYY-MM-DD`
  strings produced by a hand-rolled civil-from-days calculation. Two edits on the same day are
  indistinguishable, which is unusable for last-writer-wins.
- **Permanent deletion leaves no trace.** `purge_note`, `empty_trash` and `purge_expired` drop
  rows outright. Without a record, a sync cannot tell "deleted here" from "not yet seen here",
  and purged notes would be re-downloaded from the remote forever.

Notes embed images as base64 inside their HTML `content`, so a single note can be megabytes.
That rules out any design that re-uploads the whole corpus on each sync.

## Goals / Non-Goals

**Goals:**

- A Google account connects with a plain login, from the app, in one action.
- Notes, trash state and tags converge across machines, in both directions.
- No edit is ever silently lost, including when two machines edit the same note.
- Permanent deletions stick; purged notes do not resurrect.
- Tokens never touch the webview and the CSP stays as closed as it is now.
- Everything Folioo does today keeps working with no account and no network.

**Non-Goals:**

- **iCloud.** Apple publishes no API a Linux app can use for iCloud Drive. CloudKit Web
  Services needs a paid developer membership, a container provisioned from the CloudKit
  dashboard, and Apple's JS sign-in widget on registered web origins. Out of scope, and not
  worked around with a scraping or reverse-engineered approach.
- Real-time collaboration or multi-device live presence. Sync is periodic reconciliation.
- Character-level merge of conflicting edits. Conflicts are resolved by keeping both versions.
- Sharing notes with other people, or any other Drive-hosted social feature.
- End-to-end encryption of the synced payload (see Open Questions).

## Decisions

### OAuth in the system browser, not a webview

Google refuses OAuth requests made from embedded webviews (`disallowed_useragent`), and doing it
in Folioo's own webview would put the user's Google password inside our window. The flow is
therefore RFC 8252: open the authorization URL in the system browser, listen on
`http://127.0.0.1:<ephemeral-port>` for the redirect, and shut the listener down as soon as the
code arrives or the attempt times out.

*Alternative considered:* the device-code flow, which avoids a local listener. Rejected because
it makes the user type a code into another device for what should be a one-click login.

### PKCE, and a client ID that is not a secret

The client ID ships in the binary. For an open-source desktop app this is unavoidable and
correct — RFC 8252 classifies installed apps as *public* clients, where the "client secret" is
not a secret and PKCE is what actually binds the authorization code to this specific attempt.
A fresh code verifier and `state` are generated per attempt, and a redirect whose `state`
doesn't match is dropped before any token exchange.

*Alternative considered:* each user registering their own Google Cloud project. Rejected as a
product decision — it turns a login into a twenty-minute console chore.

### `drive.file` scope only

`drive.file` grants access solely to files the app creates. It cannot read the user's existing
Drive, it is a non-sensitive scope (no Google verification review, no "unverified app"
interstitial, no 100-user cap), and it is sufficient because Folioo only ever touches its own
folder.

*Alternative considered:* `drive.appdata`, which hides the data in an app-private folder.
Rejected because the user then cannot see or back up their own notes from Drive, and the scope
carries a heavier review burden.

### All network and token handling in Rust

`reqwest` with `rustls` does every OAuth and Drive call from the backend. The frontend learns
only what `sync_status` tells it: connected email, state, last-sync time, error string. This
keeps access tokens out of a webview that renders user-authored HTML, and means the CSP needs
no Google origins added.

The refresh token goes to the OS secret store through the `keyring` crate (Secret Service on
Linux). Where no provider answers — a bare WM, some containers — it falls back to a `0600` file
in the app-data directory, which is no worse than the unencrypted `notes.db` sitting beside it.

### Remote layout: one JSON file per note

The `Folioo` Drive folder holds `<note-id>.json` per note and a single `folders.json`. Per-note
files mean a sync uploads only what changed — decisive when a note carries base64 images — and
give each note an independent `modifiedTime` and `appProperties` on the Drive side, so the
engine can diff cheaply with one `files.list`.

*Alternative considered:* a single `folioo.json` snapshot of the whole database. Simpler, but
every sync would re-upload every image in every note, and two machines writing the snapshot
would conflict on the whole corpus instead of on one note.

### Millisecond timestamps and a tombstone table

`notes` gains `updated_ms INTEGER`, stamped by every write path. Existing rows are backfilled
from their `updated` date (midnight of that day) — imprecise for pre-migration notes, but
monotonic and good enough, since the first sync after upgrade establishes a real baseline.

A `deletions (id TEXT PRIMARY KEY, deleted_ms INTEGER)` table records every permanent removal.
`purge_note`, `empty_trash` and `purge_expired` write to it in the same transaction that drops
the row. Tombstones are pruned once they are older than any plausible offline window.

Note that *trashing* needs no tombstone: a trashed note is still a row with `deleted_at` set, so
it syncs as an ordinary note and the trash is consistent across machines for free. Only the
permanent purge needs a marker.

### Sync algorithm

The engine keeps a local `sync_state` record per note: the remote file id and the `updated_ms`
at the last successful sync (`synced_ms`). One pass:

1. `files.list` the folder, collecting each remote file's id, name and `modifiedTime`.
2. For each note id present locally, remotely, or in `deletions`, classify:
   - local-only, or `updated_ms > synced_ms` while remote is unchanged → **upload**
   - remote-only, or remote newer while `updated_ms == synced_ms` → **download**
   - both changed since `synced_ms` → **conflict**
   - tombstone newer than remote `modifiedTime` → **delete remote**
   - tombstone older than remote `modifiedTime` → **download** (the remote edit wins; the
     tombstone is dropped)
3. Merge `folders.json` by tag name, later timestamp winning on color.
4. Create any tag a downloaded note references but neither side has, so no note is orphaned.
5. Record the new `synced_ms` per note.

**Conflicts keep both copies.** The later-modified version stays as the note; the loser is
re-inserted under a fresh id, same tag, with its title marked as a conflicted copy, and is then
uploaded as a new note. Losing a paragraph someone typed is a far worse failure than an extra
note in the list.

### Scheduling

A single background task, guarded by a flag so only one sync is ever in flight: on startup when
an account is connected, on explicit request, and debounced (order of seconds) after local edits
settle. Autosave already fires every few hundred milliseconds while typing, so the debounce is
what keeps sync from chasing every keystroke. The UI never blocks on a sync; results arrive as
a Tauri event that refreshes the note list.

## Risks / Trade-offs

- **Clock skew between machines breaks last-writer-wins.** A machine with a badly wrong clock
  can win every conflict or lose every one. → Conflicts keep both copies, so skew costs the user
  an extra note rather than their work. Drive's server-side `modifiedTime` is used for the
  remote side wherever possible rather than trusting a timestamp inside the payload.
- **Backfilled `updated_ms` is coarse.** Pre-migration notes all land on midnight of their
  `updated` date, so the first sync after upgrading may mis-order notes edited the same day on
  two machines. → Both copies are kept, and the first successful sync establishes real
  timestamps from then on.
- **Base64 images make notes large.** A few image-heavy notes can dominate sync time and Drive
  quota. → Per-note files mean only changed notes move; images are already downscaled to 1600px
  on insert. Extracting images into separate Drive files is a plausible follow-up, not this
  change.
- **The secret-store fallback stores a refresh token in a plain file.** → `0600`, in the same
  directory as the already-unencrypted database; the fallback is used only when no Secret
  Service provider exists, and the status UI can say so.
- **Flatpak sandboxing.** The keyring needs `--talk-name=org.freedesktop.secrets` and opening
  the browser goes through the portal. → Both are added to the manifest and must be verified in
  an actual Flatpak build, not only in `tauri dev`.
- **A half-finished sync leaves mixed state.** Interrupting mid-pass can leave some notes synced
  and others not. → Each note is committed independently and `synced_ms` is recorded per note,
  so the pass is resumable: the next sync simply re-classifies whatever is still behind.
- **Growing dependency surface.** `reqwest` + `rustls` + `keyring` add meaningfully to a binary
  whose release profile is deliberately tuned for small, low-memory links. → Default features
  trimmed (no `openssl`, no `reqwest` blocking/cookies); binary size checked before and after.

## Migration Plan

1. Ship the schema migration (`updated_ms`, `deletions`) first; it is idempotent and safe on its
   own, and leaves the app fully functional with no sync code reachable.
2. Sync stays inert until an account is connected — no account means not one Drive call.
3. Rollback: the added column and table are ignored by older builds' queries, so downgrading
   the binary leaves notes readable. Only `updated_ms` stamping is lost, which the next upgrade
   backfills.
4. The first sync on a connected account is upload-only when the remote folder is empty, so the
   initial state is always the user's existing local database.

## Resolved Decisions

These were open when the design was drafted and have since been decided.

- **The synced payload is not encrypted.** Notes are uploaded as readable JSON. The user can
  open and recover them from Drive by hand, and there is no passphrase to lose. Google can read
  them, as it can read anything else stored there. Opt-in encryption stays a possible follow-up,
  not part of this change.
- **Tombstones are retained for 90 days.** That is the window a machine can stay offline before
  a permanent deletion stops being enforced on it: a machine returning after longer still holds
  its local copy, finds no tombstone to learn from, and re-uploads the note. Ninety days covers
  a laptop shelved for a season while keeping the table bounded.
- **The OAuth client ID is read by `build.rs` from an environment variable**, falling back to the
  Folioo project's own ID when unset. Downstream packagers and forks can build with their own
  credentials without patching the source.
- **User-facing strings are English.** A conflicted copy is marked in English, consistent with
  the rest of the UI once `translate-ui-to-english` lands.

## Open Questions

None outstanding.
