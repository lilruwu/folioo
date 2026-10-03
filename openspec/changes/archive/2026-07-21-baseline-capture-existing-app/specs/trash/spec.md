## ADDED Requirements

### Requirement: Move A Note To Trash
The system SHALL move a note to the trash (soft delete) only after the user confirms in a
dialog naming the note's title. A trashed note SHALL disappear from all active views and appear
in "Papelera".

#### Scenario: Deleting a note
- **WHEN** the user clicks delete on an open note and confirms the dialog
- **THEN** the note is removed from its active tag view and appears in the trash list

### Requirement: Show Time Remaining In The Trash
The system SHALL display, for each trashed note, the number of days remaining in a 30-day
retention window before it is permanently deleted, showing "Se elimina hoy" on the final day.

#### Scenario: Viewing a recently trashed note
- **WHEN** a note was trashed today
- **THEN** the trash list shows "Quedan 30 días" for it

#### Scenario: Viewing a note on its last day
- **WHEN** a trashed note has 0 days of retention left
- **THEN** it shows "Se elimina hoy" instead of a day count

### Requirement: Restore A Note From Trash
The system SHALL move a trashed note back into its active tag when the user restores it,
clearing its deletion date.

#### Scenario: Restoring a note
- **WHEN** the user clicks "Restaurar" on a trashed note
- **THEN** the note returns to its active tag view and is removed from the trash list

### Requirement: Permanently Delete A Single Trashed Note
The system SHALL permanently delete a single trashed note only after the user confirms a
destructive-styled dialog stating the action cannot be undone.

#### Scenario: Purging one note
- **WHEN** the user clicks "Eliminar definitivamente" on a trashed note and confirms
- **THEN** that note is permanently removed and cannot be recovered

### Requirement: Empty The Trash
The system SHALL permanently delete every trashed note in one action, only after the user
confirms a destructive-styled dialog stating how many notes will be removed.

#### Scenario: Emptying a non-empty trash
- **WHEN** the trash contains 5 notes and the user confirms "Vaciar papelera"
- **THEN** all 5 notes are permanently removed and the trash becomes empty

### Requirement: Auto-Purge Expired Trash On Startup
The system SHALL, on every app startup, permanently delete trashed notes whose deletion date is
more than 30 days in the past.

#### Scenario: Starting the app with expired trash
- **WHEN** the app starts and a trashed note was deleted 31 or more days ago
- **THEN** that note is permanently removed before the note lists are shown

### Requirement: View A Trashed Note Read-Only
The system SHALL display a selected trashed note's title and content as read-only, offering only
Restore and permanent-delete actions instead of the normal editing toolbar.

#### Scenario: Selecting a trashed note
- **WHEN** the user selects a note while viewing the Papelera
- **THEN** its content is shown without an editable toolbar, with Restore and Eliminar definitivamente actions in its place
