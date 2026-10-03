## Context

The app uses a fully opaque background for the window and note area. On KDE Plasma with a compositor (KWin), windows can have a transparent background and KDE applies its own blur effect behind them. Currently the app does not take advantage of this, looking non-native on KDE.

The existing settings system supports three appearance variants (Papel, Pizarra, Bosque) with light/dark/auto color modes, persisted via a key-value store. The translucid background toggle will integrate into this same system.

## Goals / Non-Goals

**Goals:**
- Add a "Fondo translúcido" toggle in Settings that makes the window background transparent
- Main content area blends with the desktop wallpaper through KDE's compositor blur
- The toggle state is persisted across launches
- All existing appearance variants and color modes still apply to remaining opaque UI chrome (toolbars, sidebar, scrollbars, buttons)

**Non-Goals:**
- Applying blur ourselves via CSS `backdrop-filter: blur()` — that would conflict with KDE's native blur and look wrong
- Per-monitor or per-virtual-desktop settings
- Supporting non-KDE compositors specifically (works as best-effort on GNOME/Wayland etc.)
- Animating the transition between opaque and translucent

## Decisions

1. **Transparency via Tauri window attribute** — Set the Tauri WebView window background to transparent using `window.setBackgroundColor([0,0,0,0])` or equivalent. This is the only reliable cross-platform way to get a transparent WebView that the compositor can blur behind.

2. **CSS class toggle on `<html>`** — When translucid is enabled, add a `data-translucid` attribute (or class) to `<html>`. The CSS will set `background: transparent` on the main containers and rely on the theme's `background-color` only on chrome elements.

3. **No separate translucid appearance variant** — The toggle works orthogonally to Papel/Pizarra/Bosque. A user can have "Papel + Translúcido" or "Pizarra + Translúcido". This minimizes complexity and keeps the UX simple.

4. **Settings UI** — A simple toggle/switch in the Configuración panel, below the appearance variant selector. Label: "Fondo translúcido". When enabled, the panel itself should also show a preview hint (e.g., the background behind the config panel becomes slightly visible).

5. **Persistence** — Save as a boolean in the same key-value settings store used for variant/mode/font-size. Key: `translucidBackground`.

## Risks / Trade-offs

- [KDE-only look] → On compositors without blur, the transparent window will look broken (see-through to desktop with no blur). Mitigation: show a small note in settings that says "Recomendado para KDE Plasma con compositor", or auto-detect KDE and enable by default.
- [Performance] → Transparent windows can be slightly more expensive to render. Mitigation: negligible on modern GPUs; user can toggle off.
- [Readability] → Text on a blurred background can be hard to read if contrast is too low. Mitigation: UI chrome elements (toolbar, sidebar, titlebar) remain opaque with their theme colors; only the note list and editor panes become transparent.
