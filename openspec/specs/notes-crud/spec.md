# notes-crud Specification

## Purpose
TBD - created by archiving change baseline-capture-existing-app. Update Purpose after archive.
## Requirements
### Requirement: Create A New Note
The system SHALL create a new, empty note in the currently selected tag, or in the first
available tag if the current view is a virtual view ("Todas las notas", "Favoritas",
"Recientes", "Papelera") that isn't a real tag. The new note SHALL become the selected note
immediately, without a round trip to reload it.

#### Scenario: Creating a note while viewing a virtual view
- **WHEN** the user presses "Nueva nota" (or Ctrl+N) while "Todas las notas" is selected
- **THEN** a new untitled note is created in the first tag, inserted at the top of the list, and selected

#### Scenario: Creating a note while viewing the trash
- **WHEN** the user creates a note while the Papelera view is active
- **THEN** the view switches to "Todas las notas" and the new note is selected there

### Requirement: List Notes As Lightweight Summaries
The system SHALL load the note list as summaries that omit the note's HTML content, so listing
and filtering never depend on the size of embedded images.

#### Scenario: Initial load
- **WHEN** the app starts
- **THEN** it fetches note summaries (id, title, folder, favorite, created, updated, deletedAt, searchText) without content, plus the trash and tag lists

### Requirement: Load Full Note Content On Selection
The system SHALL fetch a note's full content lazily, only when it becomes the selected note, and
SHALL discard the result if the selection changes again before the fetch resolves.

#### Scenario: Rapid reselection
- **WHEN** the user selects note A and then note B before A's content finishes loading
- **THEN** A's late-arriving content is discarded and only B's content is shown

### Requirement: Autosave Edits
The system SHALL persist title and content edits automatically, debounced by 400ms of
inactivity, and SHALL flush any pending save immediately before the editor is pointed at a
different note (so edits are never lost or attributed to the wrong note).

#### Scenario: Typing pauses
- **WHEN** the user stops typing for 400ms
- **THEN** the current title and content are saved to the note being edited

#### Scenario: Switching notes with unsaved edits
- **WHEN** the user selects a different note while a debounced save is still pending
- **THEN** the pending save is flushed against the original note's id before the editor loads the new note

### Requirement: Reassign A Note's Tag
The system SHALL allow moving a note to a different tag and SHALL update the note's `updated`
timestamp when it does.

#### Scenario: Changing a note's tag
- **WHEN** the user picks a different tag for a note from the tag dropdown
- **THEN** the note's folder field and updated date change, and it disappears from the previous tag's filtered list

### Requirement: Restore Last Session
The system SHALL remember the last selected tag/view and the last selected note across
launches, and SHALL fall back to the first available note if the remembered note no longer
exists.

#### Scenario: Relaunching the app
- **WHEN** the app is relaunched
- **THEN** the previously selected tag/view is restored, and the previously selected note is reselected if it still exists, otherwise the first note in the list is selected

### Requirement: Global Keyboard Shortcut To Create A Note
The system SHALL create a new note when Ctrl+N (or Cmd+N) is pressed, regardless of focus.

#### Scenario: Pressing Ctrl+N
- **WHEN** the user presses Ctrl+N anywhere in the app
- **THEN** a new note is created and selected, the same as clicking "Nueva nota"

