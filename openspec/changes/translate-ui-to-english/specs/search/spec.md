## MODIFIED Requirements

### Requirement: Browse Notes By View
The system SHALL support switching between "All notes" (all active notes), "Favourites"
(favorited notes only), "Recent" (the 8 most-recently-updated notes), "Trash" (trashed
notes), and any specific tag. Selecting a new view SHALL clear the current search query.

#### Scenario: Switching views clears search
- **WHEN** the user has an active search query and selects a different sidebar view
- **THEN** the search query is cleared and the list shows that view's notes unfiltered
