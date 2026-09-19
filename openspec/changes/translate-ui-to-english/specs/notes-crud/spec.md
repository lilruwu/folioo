## MODIFIED Requirements

### Requirement: Create A New Note
The system SHALL create a new, empty note in the currently selected tag, or in the first
available tag if the current view is a virtual view ("All notes", "Favourites",
"Recent", "Trash") that isn't a real tag. The new note SHALL become the selected note
immediately, without a round trip to reload it.

#### Scenario: Creating a note while viewing a virtual view
- **WHEN** the user presses "New note" (or Ctrl+N) while "All notes" is selected
- **THEN** a new untitled note is created in the first tag, inserted at the top of the list, and selected

#### Scenario: Creating a note while viewing the trash
- **WHEN** the user creates a note while the Trash view is active
- **THEN** the view switches to "All notes" and the new note is selected there
