## ADDED Requirements

### Requirement: Keep A Library Inside The Chosen Folder
The system SHALL keep synced data in a library directory inside the folder the user chooses:
`<chosen>/Folioo/`, or the chosen folder itself when it already contains a `library.json`. The
library SHALL contain a `library.json` marker carrying a format version, a `notes/` directory
with one file per note, a `deleted/` directory with one file per tombstone, and a `tags.json`
file holding the tag list. The system SHALL refuse to sync with a library whose format version
is newer than it supports.

#### Scenario: Choosing an empty folder
- **WHEN** the user chooses an empty folder as the sync folder
- **THEN** a `Folioo/` library with a `library.json` marker is created inside it, and every local note and the tag list are written into it

#### Scenario: Choosing a folder that is already a library
- **WHEN** the user chooses a folder that itself contains `library.json`
- **THEN** that folder is used as the library directly, without nesting a new `Folioo/` inside it

#### Scenario: Choosing a folder whose library is newer than this build
- **WHEN** the chosen library's `library.json` declares a format newer than this build supports
- **THEN** sync is refused with a message asking the user to update Folioo, and neither the library nor the local database is modified

### Requirement: Write Library Files Atomically
The system SHALL write every library file by writing a temporary file in the same directory and
renaming it over the target, so that an external sync tool never observes a partially written
file.

#### Scenario: The app is closed mid-write
- **WHEN** Folioo is interrupted while writing a note file
- **THEN** the library contains either the previous complete version of that file or the new complete version, never a truncated one

### Requirement: Synchronize Notes In Both Directions
The system SHALL, on each sync, compare local notes against the library and SHALL write notes
that are new or newer locally, read notes that are new or newer in the library, and leave notes
unchanged on both sides untouched. Trashed notes SHALL sync as notes carrying their `deletedAt`,
so the trash is consistent across machines.

#### Scenario: A note edited on another machine
- **WHEN** a sync finds a library note file newer than the local copy, which is unchanged since the last sync
- **THEN** the local note is replaced by the library content and appears updated in the note list

#### Scenario: A note created while the folder was unavailable
- **WHEN** a sync runs after notes were created while the sync folder could not be reached
- **THEN** those notes are written into the library

#### Scenario: A note trashed on another machine
- **WHEN** a sync reads a note file carrying a `deletedAt`
- **THEN** the local note moves to the trash rather than being removed outright

### Requirement: Never Delete Because A File Is Missing
The system SHALL NOT delete or trash a local note because its file is absent from the library.
A local note with no library file SHALL be written into the library. Deletion SHALL propagate
only through explicit tombstone files. When a library that was previously synced has lost its
`library.json` marker, the system SHALL NOT sync at all and SHALL report the folder as
unavailable; it SHALL initialise a library only when the user chooses the folder in Settings.

#### Scenario: The sync folder is an unmounted mount point
- **WHEN** a sync runs and the chosen folder exists but is empty because the remote storage is not mounted
- **THEN** no note is deleted or trashed, nothing is written, and Settings reports the sync folder as unavailable

#### Scenario: A note file was deleted by hand
- **WHEN** a note's file has been removed from the library outside Folioo
- **THEN** the next sync writes the note back into the library and the local note is untouched

### Requirement: Resolve Conflicting Edits Without Losing Content
The system SHALL treat a note as conflicted when both the local copy and the library copy changed
since the last successful sync of that note. It SHALL keep the copy with the later modification
timestamp as the note, and SHALL preserve the other copy as a new note — a distinct id, the same
tag, and a title marked `(conflicted copy)` — written to the library as well. The system SHALL
NOT discard either version.

#### Scenario: The same note is edited on two machines between syncs
- **WHEN** a sync finds that a note changed both locally and in the library since the last sync
- **THEN** the later-modified version remains as the note and the other version is kept as a separate note marked `(conflicted copy)`

### Requirement: Ignore Files Folioo Did Not Write
The system SHALL treat only `notes/<id>.json` and `deleted/<id>.json`, where `<id>` consists of
ASCII letters, digits, `-` and `_`, as library files. Every other file in the library — including
conflict copies created by an external sync tool — SHALL be ignored and left untouched.

#### Scenario: A sync tool left a conflict copy
- **WHEN** the library contains `notes/n1a2b.sync-conflict-20261001-120000-ABCDEFG.json`
- **THEN** sync neither imports nor removes it, and the canonical `notes/n1a2b.json` is synced as usual

#### Scenario: An unrelated file sits in the notes directory
- **WHEN** `notes/` contains a file that is not a canonical note file
- **THEN** it is ignored and left in place

### Requirement: Propagate Permanent Deletions Through Tombstones
The system SHALL write a tombstone file to `deleted/` for every note it permanently deletes —
purged individually, by emptying the trash, or by trash expiry — and SHALL remove that note's
file from `notes/`. When the library holds a tombstone newer than the local copy of a note, the
system SHALL purge the local note and record a local tombstone. When the local copy is newer
than the tombstone, the edit SHALL win: the note is written back and the tombstone file removed.

#### Scenario: Emptying the trash on one machine
- **WHEN** the user empties the trash and syncs, the folder reaches a second machine, and it syncs
- **THEN** the purged notes are removed from the second machine and do not reappear on any later sync

#### Scenario: A note is purged on one machine but edited on another afterwards
- **WHEN** the local copy's modification timestamp is newer than the tombstone in the library
- **THEN** the edited note is kept, written back to the library, and the tombstone file removed

### Requirement: Expire Tombstones After 90 Days
The system SHALL remove local tombstones and tombstone files older than 90 days.

#### Scenario: An old tombstone
- **WHEN** a sync finds a tombstone whose deletion time is more than 90 days ago
- **THEN** the tombstone is removed locally and from the library

### Requirement: Synchronize Tags
The system SHALL sync the whole tag list through `tags.json`, keeping whichever of the local list
and the file was modified more recently, so that creating, deleting and recolouring tags all
propagate. After each sync, any tag referenced by a note but missing from the tag list SHALL be
created, so no note is left pointing at a tag that does not exist.

#### Scenario: A tag deleted on another machine
- **WHEN** a sync reads a `tags.json` newer than the local tag list, and it no longer contains a tag
- **THEN** that tag disappears locally, and any note still carrying it keeps a tag that exists

#### Scenario: A note references a tag neither side has
- **WHEN** a synced note names a tag absent from the resulting tag list
- **THEN** the tag is created so the note remains reachable

### Requirement: Schedule Syncs Without Interrupting Editing
The system SHALL write locally changed notes, tombstones and the tag list to the library about
2 seconds after the user stops editing, without scanning the rest of the library, and SHALL write
any still-pending change when the app closes. It SHALL run a full sync — reading changes from
other machines — when the app starts with a sync folder set, when the user requests one, and
every 5 minutes while the app is open. All of it SHALL run in the background without blocking
the UI, only one sync SHALL be in flight at a time, and the note, trash and tag lists SHALL
refresh when a sync changes local data.

#### Scenario: Typing while a sync runs
- **WHEN** a background sync is in progress
- **THEN** the editor stays responsive and autosave continues to work

#### Scenario: An edit reaches the folder shortly after typing stops
- **WHEN** the user edits a note and stops typing
- **THEN** that note's file in the library is updated within a few seconds, and no other note file is rewritten

#### Scenario: Closing the app right after typing
- **WHEN** the user closes Folioo less than 2 seconds after editing a note
- **THEN** the edit is written to the library before the app exits

#### Scenario: Changes arrive from another machine while the app is open
- **WHEN** the external sync tool delivers an edited note file while Folioo is running
- **THEN** the edit appears in Folioo within 5 minutes without user action

#### Scenario: A sync is requested while one is running
- **WHEN** the user requests a sync during an in-flight sync
- **THEN** no second sync starts concurrently

### Requirement: Degrade Gracefully Without The Folder
The system SHALL treat sync as strictly additive to local-first storage: with no sync folder set,
or the folder unavailable or unwritable, every existing feature SHALL keep working against the
local database, and the problem SHALL be reported as a sync status rather than an error that
interrupts the user. The system SHALL NOT make any network request for sync.

#### Scenario: The external drive holding the folder is unplugged
- **WHEN** the sync folder becomes unreachable and the user creates, edits and deletes notes
- **THEN** all of it works locally, the sync status reports the folder as unavailable, and the changes are written on the next successful sync

### Requirement: Stop Syncing
The system SHALL let the user stop syncing, which forgets the chosen folder and the per-note sync
records, and SHALL leave every local note and tag, and every file in the library, untouched.

#### Scenario: Stopping sync
- **WHEN** the user stops syncing
- **THEN** no further syncs run, all local notes remain, and the library folder is left as it was
