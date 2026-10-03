## MODIFIED Requirements

### Requirement: Export All Data To A JSON File
The system SHALL export every tag and every note (both active and trashed) to a JSON file
chosen via a native save dialog, tagged with an app identifier and a format version number.

#### Scenario: Exporting a backup
- **WHEN** the user chooses "Export backup…" and picks a destination path
- **THEN** a JSON file is written containing all tags and all notes, including trashed ones, with app/version metadata

### Requirement: Reflect A Successful Import Immediately
The system SHALL reload the note list, trash list, and tag list right after a successful import,
and SHALL display how many notes were imported.

#### Scenario: Import completes
- **WHEN** an import finishes successfully with 12 notes
- **THEN** the UI refreshes to show the imported notes and displays "Imported 12 notes."
