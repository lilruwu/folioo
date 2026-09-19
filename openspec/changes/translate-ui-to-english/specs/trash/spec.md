## MODIFIED Requirements

### Requirement: Move A Note To Trash
The system SHALL move a note to the trash (soft delete) only after the user confirms in a
dialog naming the note's title. A trashed note SHALL disappear from all active views and appear
in "Trash".

#### Scenario: Deleting a note
- **WHEN** the user clicks delete on an open note and confirms the dialog
- **THEN** the note is removed from its active tag view and appears in the trash list

### Requirement: Show Time Remaining In The Trash
The system SHALL display, for each trashed note, the number of days remaining in a 30-day
retention window before it is permanently deleted, showing "Deleted today" on the final day.

#### Scenario: Viewing a recently trashed note
- **WHEN** a note was trashed today
- **THEN** the trash list shows "30 days left" for it

#### Scenario: Viewing a note on its last day
- **WHEN** a trashed note has 0 days of retention left
- **THEN** it shows "Deleted today" instead of a day count
