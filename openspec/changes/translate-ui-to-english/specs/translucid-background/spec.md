## MODIFIED Requirements

### Requirement: Toggle Translucid Background
The system SHALL provide a toggle in the Settings panel labeled "Translucent background" that
controls whether the main window background is transparent, letting the compositor's blur show
behind the app content.

#### Scenario: Enabling translucid background
- **WHEN** the user activates the "Translucent background" toggle in Settings
- **THEN** the main window background becomes transparent and the compositor blur becomes visible behind the content

#### Scenario: Disabling translucid background (default)
- **WHEN** the user deactivates the toggle
- **THEN** the window returns to its fully opaque theme background

### Requirement: Show Platform Recommendation
The system SHALL display a small hint next to the translucid toggle: "Recommended for KDE
Plasma with a compositor."

#### Scenario: Viewing the settings toggle
- **WHEN** the user opens Settings
- **THEN** the translucid toggle includes the recommendation text
