## ADDED Requirements

### Requirement: Stamp Every Note Write With A Millisecond Timestamp
The system SHALL maintain an `updated_ms` column on `notes` holding milliseconds since the Unix
epoch, and SHALL set it on every operation that changes a note — create, content update, tag
change, favorite toggle, trash, restore, and import. The existing day-granular `updated` date
SHALL continue to be maintained unchanged for display and ordering.

#### Scenario: Two edits to different notes on the same day
- **WHEN** one note is edited in the morning and another in the afternoon of the same day
- **THEN** both carry the same `updated` date but distinct `updated_ms` values ordering them correctly

#### Scenario: Importing a backup
- **WHEN** a JSON backup is imported
- **THEN** each upserted note receives an `updated_ms` value, so imported notes participate in sync comparisons

### Requirement: Record Tombstones For Permanently Deleted Notes
The system SHALL keep a `deletions` table recording the id and a millisecond deletion timestamp
for every note removed permanently — individually purged, purged by emptying the trash, or
purged by trash expiry. Tombstones SHALL survive restarts and SHALL be readable independently of
the `notes` table.

#### Scenario: Purging a note from the trash
- **WHEN** a trashed note is deleted forever
- **THEN** the note row is removed and a tombstone with its id and the current time is recorded

#### Scenario: Trash expiry runs at startup
- **WHEN** notes past the 30-day retention window are purged automatically at launch
- **THEN** each purged note leaves a tombstone

### Requirement: Track When The Tag List Last Changed
The system SHALL record a millisecond timestamp of the most recent change to the tag list —
creation, deletion, or color change — so tag state can be compared against a remote copy.

#### Scenario: Creating a tag
- **WHEN** the user creates a new tag
- **THEN** the tag-list modification timestamp advances to the current time

## MODIFIED Requirements

### Requirement: Apply Schema Migrations Idempotently
The system SHALL check, on every startup, whether the `notes` table already has the
`deleted_at`, `content_text`, and `updated_ms` columns, and SHALL add whichever is missing
without altering a database that already has them. When `content_text` is newly added, it SHALL
be backfilled for every existing row from that row's current content. When `updated_ms` is newly
added, it SHALL be backfilled for every existing row from that row's `updated` date, so notes
predating the column still compare sensibly. The system SHALL likewise create the `deletions`
table if it is absent.

#### Scenario: Opening a database created before the trash feature existed
- **WHEN** the app opens a `notes.db` that predates the `deleted_at` column
- **THEN** the column is added automatically and existing notes are treated as active (not trashed)

#### Scenario: Opening a database created before sync existed
- **WHEN** the app opens a `notes.db` that has no `updated_ms` column and no `deletions` table
- **THEN** the column and the table are created, and `updated_ms` is backfilled from each row's `updated` date without changing any note's content

### Requirement: Track Dates Without An External Date Library
The system SHALL compute and format all note dates (`created`, `updated`, `deletedAt`) as
`YYYY-MM-DD` strings derived from whole days elapsed since the Unix epoch, using a self-contained
calendar calculation rather than a third-party date crate. Sync timestamps (`updated_ms`,
tombstone times) SHALL likewise be derived from the system clock as raw milliseconds since the
epoch, without an external date dependency.

#### Scenario: Computing today's date
- **WHEN** the backend needs today's date for a note operation
- **THEN** it derives a `YYYY-MM-DD` string from the current system time without an external date dependency

#### Scenario: Stamping a note for sync
- **WHEN** the backend records a note's modification time for sync comparison
- **THEN** it stores milliseconds since the Unix epoch taken from the system clock, with no external date dependency
