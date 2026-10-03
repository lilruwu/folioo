## ADDED Requirements

### Requirement: Choose An Appearance Variant
The system SHALL let the user choose between three appearance variants — "Papel", "Pizarra",
"Bosque" — applied immediately across the whole app.

#### Scenario: Switching variant
- **WHEN** the user selects "Pizarra" in Configuración
- **THEN** the app's appearance switches to the Pizarra variant immediately

### Requirement: Choose A Color Mode
The system SHALL let the user choose "Claro" (light), "Oscuro" (dark), or "Automático" (auto).
In auto mode, the effective mode SHALL follow the operating system's light/dark preference and
SHALL update live if that system preference changes while auto remains selected.

#### Scenario: System theme changes while in auto mode
- **WHEN** the mode is set to "Automático" and the OS switches from light to dark
- **THEN** the app's appearance switches to dark without user action

#### Scenario: Explicit mode overrides the system
- **WHEN** the mode is set to "Oscuro" and the OS is set to light
- **THEN** the app remains in dark mode

### Requirement: Adjust Text Size
The system SHALL let the user set the note text size via a slider ranging from 13px to 19px in
1px steps, applied immediately.

#### Scenario: Increasing text size
- **WHEN** the user drags the text-size slider to 19px
- **THEN** note text throughout the app is rendered at 19px

### Requirement: Persist Appearance Settings
The system SHALL persist the chosen variant, mode, and font size across launches, defaulting to
Papel / Automático / 15px when no settings have been saved yet.

#### Scenario: Relaunching after changing settings
- **WHEN** the user changes the font size and relaunches the app
- **THEN** the app opens with the previously chosen font size applied

### Requirement: Sync The Native Window Theme
The system SHALL, on a best-effort basis, set the native window/titlebar to match the chosen
mode — forcing light or dark when explicitly selected, or following the system when in auto
mode — and SHALL silently no-op where the runtime or desktop doesn't support it.

#### Scenario: Choosing dark mode explicitly
- **WHEN** the user selects "Oscuro"
- **THEN** the native window is asked to switch to a dark titlebar/decoration, without raising an error if the platform ignores the request

### Requirement: Retint The App Icon To The Active Theme
The system SHALL, on a best-effort basis, redraw and apply the window icon using the active
theme's accent color whenever that accent color changes, and SHALL skip the redraw if the accent
hasn't changed since the last time it was applied.

#### Scenario: Changing the appearance variant
- **WHEN** the user switches to a variant with a different accent color
- **THEN** the window icon is redrawn in the new accent color where the platform honors per-window icons
