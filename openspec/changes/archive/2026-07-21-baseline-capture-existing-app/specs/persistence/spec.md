## ADDED Requirements

### Requirement: Store Notes In A Local SQLite Database
The system SHALL store all notes and tags in a local SQLite database file inside the platform's
app-data directory, creating the directory and database on first run, with WAL journal mode and
`synchronous=NORMAL` so frequent autosave writes don't force a full journal fsync each time.

#### Scenario: First run
- **WHEN** the app runs for the first time on a machine
- **THEN** the app-data directory and a `notes.db` SQLite database are created, with WAL mode enabled

### Requirement: Apply Schema Migrations Idempotently
The system SHALL check, on every startup, whether the `notes` table already has the
`deleted_at` and `content_text` columns, and SHALL add whichever is missing without altering a
database that already has them. When `content_text` is newly added, it SHALL be backfilled for
every existing row from that row's current content.

#### Scenario: Opening a database created before the trash feature existed
- **WHEN** the app opens a `notes.db` that predates the `deleted_at` column
- **THEN** the column is added automatically and existing notes are treated as active (not trashed)

### Requirement: Maintain A Plain-Text Search Shadow
The system SHALL keep a plain-text derivative of each note's HTML content (`content_text`) in
sync on every insert, update, and import, produced by stripping tags and their attributes
(including embedded base64 image data), decoding the HTML entities the editor produces, and
collapsing whitespace, with block-tag boundaries treated as word separators.

#### Scenario: A note contains an embedded image
- **WHEN** a note's HTML content includes an `<img>` tag with a large base64 `src`
- **THEN** the search shadow excludes that data entirely, containing only the note's visible text

### Requirement: Generate Note Identifiers
The system SHALL generate each new note's id from the current time in nanoseconds, prefixed with
`n`, without relying on a database-assigned sequence.

#### Scenario: Creating two notes back to back
- **WHEN** two notes are created in quick succession
- **THEN** each receives a distinct id derived from its creation time

### Requirement: Track Dates Without An External Date Library
The system SHALL compute and format all note dates (`created`, `updated`, `deletedAt`) as
`YYYY-MM-DD` strings derived from whole days elapsed since the Unix epoch, using a self-contained
calendar calculation rather than a third-party date crate.

#### Scenario: Computing today's date
- **WHEN** the backend needs today's date for a note operation
- **THEN** it derives a `YYYY-MM-DD` string from the current system time without an external date dependency

### Requirement: Index Notes For Common Queries
The system SHALL maintain an index on `(deleted_at, updated DESC)` to keep listing active notes,
listing trashed notes, and finding expired trash efficient as the note count grows.

#### Scenario: Listing notes as the database grows
- **WHEN** the notes table holds thousands of rows
- **THEN** listing active notes ordered by most-recently-updated uses the `(deleted_at, updated DESC)` index instead of a full table scan
