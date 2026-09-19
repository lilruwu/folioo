## MODIFIED Requirements

### Requirement: Choose An Appearance Variant
The system SHALL let the user choose between three appearance variants — "Paper", "Slate",
"Forest" — applied immediately across the whole app.

#### Scenario: Switching variant
- **WHEN** the user selects "Slate" in Settings
- **THEN** the app's appearance switches to the Slate variant immediately

### Requirement: Choose A Color Mode
The system SHALL let the user choose "Light", "Dark", or "Auto".
In auto mode, the effective mode SHALL follow the operating system's light/dark preference and
SHALL update live if that system preference changes while auto remains selected.

#### Scenario: System theme changes while in auto mode
- **WHEN** the mode is set to "Auto" and the OS switches from light to dark
- **THEN** the app's appearance switches to dark without user action

#### Scenario: Explicit mode overrides the system
- **WHEN** the mode is set to "Dark" and the OS is set to light
- **THEN** the app remains in dark mode

### Requirement: Toggle Translucid Background
The system SHALL provide a toggle in the Settings panel labeled "Translucent background" that
controls whether the main window background is transparent.

#### Scenario: Enabling translucid background
- **WHEN** the user activates the "Translucent background" toggle
- **THEN** the toggle is saved and the translucid mode activates

### Requirement: Show Platform Recommendation
The system SHALL display "Recommended for KDE Plasma with a compositor" next to the toggle.

#### Scenario: Viewing the toggle
- **WHEN** the user opens Settings
- **THEN** the recommendation text is shown beneath the translucent-background toggle

## ADDED Requirements

### Requirement: Present The Interface In English
The system SHALL present every user-facing string in English — navigation, note list, editor
toolbar, find bar, modals, status messages, and errors surfaced from the backend. The system
SHALL NOT ship a locale-selection mechanism or translation files.

#### Scenario: Using the app in any environment
- **WHEN** the user opens Folioo, regardless of the operating system's locale
- **THEN** the whole interface reads in English

### Requirement: Seed English Tags On A Fresh Install Only
The system SHALL seed the tags Work · Personal · Projects · Ideas when the database has no tags
yet, and SHALL leave the tags of an existing database untouched.

#### Scenario: Upgrading a database seeded with the previous Spanish tags
- **WHEN** the app opens a database whose tags are Trabajo · Personal · Proyectos · Ideas
- **THEN** those tag names are preserved exactly, and no note is reassigned
