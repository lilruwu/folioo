## ADDED Requirements

### Requirement: Filter The Note List
The system SHALL filter the visible note list by a case-insensitive match against the note's
title or the plain-text shadow of its content, applied 150ms after the user stops typing in the
search box.

#### Scenario: Typing a search query
- **WHEN** the user types into the search box and pauses for 150ms
- **THEN** the note list shows only notes whose title or text content contains the query, case-insensitively

### Requirement: Browse Notes By View
The system SHALL support switching between "Todas las notas" (all active notes), "Favoritas"
(favorited notes only), "Recientes" (the 8 most-recently-updated notes), "Papelera" (trashed
notes), and any specific tag. Selecting a new view SHALL clear the current search query.

#### Scenario: Switching views clears search
- **WHEN** the user has an active search query and selects a different sidebar view
- **THEN** the search query is cleared and the list shows that view's notes unfiltered

### Requirement: Focus Search With A Keyboard Shortcut
The system SHALL focus the note-list search box when Ctrl+F (or Cmd+F) is pressed, regardless of
current focus.

#### Scenario: Pressing Ctrl+F
- **WHEN** the user presses Ctrl+F anywhere in the app
- **THEN** the note-list search box receives focus

### Requirement: Find Matches Inside The Open Note
The system SHALL provide an in-note find bar (opened from a magnifier button in the editor
toolbar) that highlights every case-insensitive match of the typed query within the open note's
content, with the current match visually distinguished from the others.

#### Scenario: Typing a query in the find bar
- **WHEN** the user opens the find bar and types a query that appears three times in the note
- **THEN** all three occurrences are highlighted and the match counter shows "1/3"

### Requirement: Navigate Between In-Note Matches
The system SHALL move to the next match on Enter or the down arrow, and to the previous match on
Shift+Enter or the up arrow, wrapping around at either end. Escape SHALL close the find bar.

#### Scenario: Navigating past the last match
- **WHEN** the current match is the last one and the user goes to the next match
- **THEN** the first match becomes current, wrapping around
