# translucid-background Specification

## Purpose
Provides a translucent/transparent window background toggle with compositor blur support for KDE Plasma.

## Requirements

### Requirement: Toggle Translucid Background
The system SHALL provide a toggle in the Configuración panel labeled "Fondo translúcido" that
controls whether the main window background is transparent, letting the compositor's blur show
behind the app content.

#### Scenario: Enabling translucid background
- **WHEN** the user activates the "Fondo translúcido" toggle in Configuración
- **THEN** the main window background becomes transparent and the compositor blur becomes visible behind the content

#### Scenario: Disabling translucid background (default)
- **WHEN** the user deactivates the toggle
- **THEN** the window returns to its fully opaque theme background

### Requirement: Apply Translucid Mode Orthogonally
The translucid background mode SHALL work independently of the chosen appearance variant
(Papel / Pizarra / Bosque) and color mode (Claro / Oscuro / Automático).

#### Scenario: Papel variant with translucid
- **WHEN** the variant is "Papel" and translucid is enabled
- **THEN** UI chrome keeps Papel colors, the note area and background are transparent

#### Scenario: Switching variants preserves translucid
- **WHEN** translucid is enabled and the user switches from Pizarra to Bosque
- **THEN** the app stays in translucid mode

### Requirement: Keep Chrome Opaque
When translucid mode is active, the system SHALL keep the toolbar, sidebar, scrollbars,
and the settings panel visually opaque with their theme colors applied.

#### Scenario: Sidebar stays opaque
- **WHEN** translucid is enabled
- **THEN** the sidebar panels retain their theme-defined background color

### Requirement: Show Platform Recommendation
The system SHALL display a small hint next to the translucid toggle: "Recomendado para
KDE Plasma con compositor."

#### Scenario: Viewing the settings toggle
- **WHEN** the user opens Configuración
- **THEN** the translucid toggle includes the recommendation text