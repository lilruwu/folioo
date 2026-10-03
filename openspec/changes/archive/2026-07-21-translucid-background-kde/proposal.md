## Why

KDE Plasma users expect blurred transparent window backgrounds for a modern, cohesive desktop experience ("KDE blur" / "translucency"). Currently the app uses an opaque background regardless of the compositor, which looks foreign on KDE and misses an opportunity to feel native.

## What Changes

- Add a "Fondo translúcido" (Translucent background) toggle to the Settings panel
- When enabled, the main window background becomes transparent and the app relies on the compositor's blur (KDE's built-in blur) behind the content
- The existing appearance variants (Papel, Pizarra, Bosque) continue working — their colors are used for UI chrome but the main note area/background becomes translucent
- When disabled, behavior is identical to today (fully opaque)

## Capabilities

### New Capabilities
- `translucid-background`: translucent/transparent window background toggle with compositor blur support for KDE

### Modified Capabilities
- `settings`: add the "Fondo translúcido" toggle requirement
- `persistence`: persist the translucid-background preference across launches

## Impact

- Settings UI: new toggle switch in the Configuración panel
- CSS: new CSS variables/classes for translucid mode, applied to root and note container
- Tauri: `window.setBackgroundColor` or transparent window attributes for the WebView
- Persistence: a new key/value in the settings store for the toggle state
