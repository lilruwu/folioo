## ADDED Requirements

### Requirement: Export All Data To A JSON File
The system SHALL export every tag and every note (both active and trashed) to a JSON file
chosen via a native save dialog, tagged with an app identifier and a format version number.

#### Scenario: Exporting a backup
- **WHEN** the user chooses "Exportar copia…" and picks a destination path
- **THEN** a JSON file is written containing all tags and all notes, including trashed ones, with app/version metadata

### Requirement: Import A Backup Non-Destructively
The system SHALL import a JSON backup chosen via a native open dialog by merging it into the
current database: notes SHALL be upserted by id (an existing note with the same id is
overwritten, a new id is inserted), and tags SHALL be inserted only if no tag with that name
already exists. Existing data not present in the backup SHALL be left untouched.

#### Scenario: Importing into a non-empty database
- **WHEN** the user imports a backup while notes already exist locally
- **THEN** notes from the backup are added or overwritten by id, existing tags are preserved, and unrelated existing notes are unaffected

### Requirement: Reject An Invalid Backup File
The system SHALL reject a selected file that isn't valid JSON in the expected backup shape,
reporting an error without modifying the database.

#### Scenario: Selecting a malformed file
- **WHEN** the user selects a file that isn't a valid backup
- **THEN** the import is rejected with an error message and no data is changed

### Requirement: Reflect A Successful Import Immediately
The system SHALL reload the note list, trash list, and tag list right after a successful import,
and SHALL display how many notes were imported.

#### Scenario: Import completes
- **WHEN** an import finishes successfully with 12 notes
- **THEN** the UI refreshes to show the imported notes and displays "Importadas 12 notas."
