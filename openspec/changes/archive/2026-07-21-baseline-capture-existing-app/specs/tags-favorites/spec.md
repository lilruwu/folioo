## ADDED Requirements

### Requirement: Create A Tag
The system SHALL let the user create a tag with a name and a color chosen from a fixed 8-color
palette. The name SHALL be trimmed, required to be non-empty, limited to 24 characters, and
must not match an existing tag name case-insensitively.

#### Scenario: Creating a valid tag
- **WHEN** the user submits a new tag with a unique, non-empty name of 24 characters or fewer
- **THEN** the tag is created and appears in the sidebar

#### Scenario: Duplicate tag name
- **WHEN** the user submits a tag name that already exists, differing only in case
- **THEN** the creation is rejected with an error explaining the name is already in use

### Requirement: Delete A Tag
The system SHALL require at least one tag to always exist and SHALL refuse to delete the last
remaining one. When a tag with notes assigned to it is deleted, those notes SHALL be reassigned
to a fallback tag (the next tag in display order, excluding the one being deleted).

#### Scenario: Deleting a tag with notes
- **WHEN** the user deletes a tag that has notes assigned to it
- **THEN** the tag is removed and its notes are moved to the fallback tag

#### Scenario: Deleting the only tag
- **WHEN** only one tag exists and the user attempts to delete it
- **THEN** the deletion is rejected

### Requirement: Reassign A Note's Tag From The Editor
The system SHALL show the note's current tag as a dropdown in the editor, listing every tag with
a checkmark on the current selection and an option to create a new tag inline.

#### Scenario: Changing tag from the editor
- **WHEN** the user opens the tag dropdown on an open note and selects a different tag
- **THEN** the note's tag updates immediately and the dropdown closes

### Requirement: Seed Default Tags On First Run
The system SHALL create four default tags — "Trabajo", "Personal", "Proyectos", "Ideas" — the
first time the app runs, only if no tags exist yet.

#### Scenario: First launch with an empty database
- **WHEN** the app starts with no tags in the database
- **THEN** the four default tags are created with their default colors

### Requirement: Toggle Favorite
The system SHALL let the user mark or unmark a note as a favorite from a star button in the
editor toolbar.

#### Scenario: Favoriting a note
- **WHEN** the user clicks the star button on an open note
- **THEN** the note is marked as a favorite and appears in the "Favoritas" view

### Requirement: Sidebar Tag And Favorite Counts
The system SHALL show, in the sidebar, a live count of notes per tag and the total number of
favorited notes.

#### Scenario: Counts update after moving a note
- **WHEN** a note is reassigned from one tag to another
- **THEN** the sidebar's per-tag counts update to reflect the note's new tag
