## ADDED Requirements

### Requirement: Authorize Google Drive Through The System Browser
The system SHALL connect a Google account using an OAuth 2.0 authorization-code flow with PKCE
(RFC 7636) as a public client (RFC 8252): it SHALL generate a fresh code verifier and `state`
per attempt, open the authorization URL in the user's **system browser** — never in an embedded
webview, which Google rejects — and receive the authorization code on a loopback HTTP listener
bound to `127.0.0.1` on an ephemeral port. The system SHALL reject any redirect whose `state`
does not match the one it generated.

#### Scenario: Connecting an account
- **WHEN** the user chooses "Conectar con Google" in Configuración
- **THEN** the default browser opens Google's consent screen, and after the user approves, Folioo exchanges the returned code for tokens and reports the connected account's email

#### Scenario: Redirect carries a mismatched state
- **WHEN** the loopback listener receives a redirect whose `state` differs from the one generated for this attempt
- **THEN** the code is discarded, no token exchange is attempted, and the connection fails with an error

#### Scenario: User denies consent
- **WHEN** the user closes the consent screen or denies access
- **THEN** the attempt ends without an account connected, the local database is untouched, and Configuración reports that the connection was not completed

### Requirement: Request Only App-Created-File Access
The system SHALL request exactly the `https://www.googleapis.com/auth/drive.file` scope, which
grants access only to files the app itself creates, and SHALL NOT request any scope that exposes
the user's existing Drive contents.

#### Scenario: Reviewing the consent screen
- **WHEN** the user reaches Google's consent screen
- **THEN** the only Drive permission requested is access to files created by Folioo

### Requirement: Store Credentials Outside The Webview
The system SHALL keep the refresh token in the operating system's secret store via the Secret
Service API, falling back to a file in the app-data directory with `0600` permissions when no
secret store is reachable. Access tokens SHALL be held in memory only. No access token, refresh
token, or client credential SHALL ever be exposed to the frontend or stored in `localStorage`.

#### Scenario: Relaunching after connecting
- **WHEN** the user relaunches Folioo with an account already connected
- **THEN** the refresh token is read from the secret store and a new access token is obtained without asking the user to log in again

#### Scenario: No secret store available
- **WHEN** the desktop session exposes no Secret Service provider
- **THEN** the refresh token is written to a `0600` file in the app-data directory and sync continues to work

### Requirement: Perform All Drive Traffic From The Backend
The system SHALL issue every Google OAuth and Drive API request from the Rust backend over
HTTPS. The frontend SHALL interact with sync only through Tauri commands, and the webview's
Content-Security-Policy SHALL NOT be widened to permit Google origins.

#### Scenario: Inspecting network access from the frontend
- **WHEN** a sync is in progress
- **THEN** no request to a Google origin originates from the webview, and the CSP remains restricted to `'self'`

### Requirement: Refresh Expired Access Tokens
The system SHALL exchange the stored refresh token for a new access token when the current one
is absent or expired, and SHALL retry the failed request once after a successful refresh. When
the refresh token itself is rejected — revoked by the user or expired — the system SHALL mark
the account disconnected and surface that the user must reconnect, without deleting any local
note.

#### Scenario: Access token expires mid-session
- **WHEN** a Drive request fails with an authentication error and a valid refresh token is stored
- **THEN** a new access token is obtained and the request is retried once, transparently to the user

#### Scenario: User revokes access from their Google account page
- **WHEN** the refresh token is rejected by Google
- **THEN** Folioo reports that the account must be reconnected, keeps every local note intact, and stops attempting to sync

### Requirement: Maintain A Dedicated Drive Folder
The system SHALL keep all synced data in a single Drive folder it creates named `Folioo`,
storing each note as a JSON file named `<note-id>.json` and all tags as a single `folders.json`.
The system SHALL record the folder's Drive id locally and SHALL recreate the folder if it no
longer exists.

#### Scenario: First sync on a fresh account
- **WHEN** an account is connected that has never synced with Folioo
- **THEN** a `Folioo` folder is created in the user's Drive and every local note and tag is uploaded into it

#### Scenario: The remote folder was deleted by the user
- **WHEN** a sync runs and the recorded folder id no longer resolves
- **THEN** a new `Folioo` folder is created and the local state is uploaded into it, without deleting any local note

### Requirement: Synchronize Notes In Both Directions
The system SHALL, on each sync, compare local notes against the remote folder and SHALL upload
notes that are new or newer locally, download notes that are new or newer remotely, and leave
notes identical on both sides untouched. Trashed notes SHALL sync as notes carrying their
`deletedAt`, so the trash is consistent across machines.

#### Scenario: A note edited on another machine
- **WHEN** a sync runs and the remote copy of a note has a newer modification timestamp than the local copy, which is unchanged since the last sync
- **THEN** the local note is replaced by the remote content and appears updated in the note list

#### Scenario: A note created locally while offline
- **WHEN** a sync runs after notes were created without a network connection
- **THEN** those notes are uploaded to the Drive folder and become available to other machines

#### Scenario: A note trashed on another machine
- **WHEN** a sync pulls a note whose remote copy carries a `deletedAt`
- **THEN** the local note moves to the trash rather than being removed outright

### Requirement: Resolve Conflicting Edits Without Losing Content
The system SHALL treat a note as conflicted when both the local and the remote copy changed
since the last successful sync of that note. It SHALL resolve the conflict by keeping the copy
with the later modification timestamp as the note, and SHALL preserve the losing copy as a new
note — a distinct id, the same tag, and a title marking it as a conflicted copy. The system
SHALL NOT discard either version.

#### Scenario: The same note is edited on two machines between syncs
- **WHEN** a sync finds that a note changed both locally and remotely since the last sync
- **THEN** the later-modified version remains as the note and the other version is kept as a separate note marked as a conflicted copy

### Requirement: Propagate Deletions Without Resurrection
The system SHALL record a tombstone when a note is permanently deleted — individually, by
emptying the trash, or by trash expiry — and SHALL delete the corresponding remote file on the
next sync. A note whose tombstone is newer than the remote copy's modification timestamp SHALL
NOT be re-downloaded.

#### Scenario: Emptying the trash on one machine
- **WHEN** the user empties the trash and syncs, then syncs on a second machine
- **THEN** the purged notes are removed from Drive and from the second machine, and do not reappear on any later sync

#### Scenario: A note is purged locally but edited remotely afterwards
- **WHEN** the remote copy's modification timestamp is newer than the local tombstone
- **THEN** the remote edit wins and the note is restored locally

### Requirement: Synchronize Tags
The system SHALL sync the tag list through `folders.json`, merging by tag name: tags present on
either side SHALL be preserved, and when the same tag name carries different colors, the copy
with the later modification timestamp SHALL win. Syncing SHALL NOT leave any note pointing at a
tag that does not exist.

#### Scenario: A tag created on another machine
- **WHEN** a sync pulls a `folders.json` containing a tag absent locally
- **THEN** that tag appears in the sidebar with its color

#### Scenario: A pulled note references an unknown tag
- **WHEN** a downloaded note names a tag that exists on neither side
- **THEN** the tag is created locally so the note remains reachable

### Requirement: Schedule Syncs Without Interrupting Editing
The system SHALL run a sync on application start when an account is connected, when the user
requests one explicitly, and on a debounced basis after local edits settle. A sync SHALL run in
the background without blocking the UI, and only one sync SHALL be in flight at a time.

#### Scenario: Typing while a sync runs
- **WHEN** a background sync is in progress
- **THEN** the editor stays responsive and autosave continues to work

#### Scenario: A sync is requested while one is running
- **WHEN** the user presses "Sincronizar ahora" during an in-flight sync
- **THEN** no second sync starts concurrently

### Requirement: Degrade Gracefully Without Connectivity
The system SHALL treat sync as strictly additive to local-first storage: with no account
connected, no network, or a failing Drive API, every existing feature SHALL keep working against
the local database, and the failure SHALL be reported as a sync status rather than an error that
interrupts the user.

#### Scenario: Working on a train with no connection
- **WHEN** the network is unavailable and the user creates, edits, and deletes notes
- **THEN** all of it works locally, the sync status reports the failure, and the changes upload on the next successful sync

### Requirement: Disconnect And Forget The Account
The system SHALL provide a disconnect action that revokes the token with Google on a best-effort
basis, erases the stored refresh token and the recorded Drive folder id, and leaves every local
note and tag untouched. It SHALL NOT delete anything from the user's Drive.

#### Scenario: Disconnecting
- **WHEN** the user chooses "Desconectar"
- **THEN** the stored credentials are erased, syncing stops, all local notes remain, and the Drive folder is left in place
