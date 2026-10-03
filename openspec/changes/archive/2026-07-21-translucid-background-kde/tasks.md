## 1. Tauri Window — Make WebView Background Transparent

- [x] 1.1 Set `transparent: true` in `desktop/src-tauri/tauri.conf.json` under the window config
- [x] 1.2 Verify on Linux: window chrome compositor blur shows through when the CSS background is transparent

## 2. CSS — Translucid mode variables and overrides

- [x] 2.1 Add `html.v-translucid` CSS selectors in `desktop/src/notes.css` that set `background: transparent` on the root (`html`), `.app-root`, `.editor-panel`, `.note-list-panel` and `.editor-body`
- [x] 2.2 Ensure sidebar, toolbar, modals, trash bar, and scrollbars retain their theme `background` colors even in translucid mode

## 3. Settings UI — Toggle switch

- [x] 3.1 Add a "Fondo translúcido" toggle in `desktop/src/settings.jsx` using a Segmented on/off control
- [x] 3.2 Add a platform hint: `Recomendado para KDE Plasma con compositor` next to the toggle
- [x] 3.3 Wire the toggle to `settings.translucid` via the `onChange` callback (default: `false`)
- [x] 3.4 Pass the new `translucid` field through `App.jsx` defaults and the `<SettingsModal>` props

## 4. App.jsx — Wire translucid mode to DOM class

- [x] 4.1 Add `settings.translucid` to the `useSettings` defaults in `desktop/src/App.jsx`: `translucid: false`, to the defaults object
- [x] 4.2 In the `apply()` effect dependency list and body, add/remove `v-translucid` class on `<html>` based on `settings.translucid`
- [x] 4.3 Set CSS transition on translucid attribute toggling for a smooth fade

## 5. Verify

- [x] 5.1 Build the app and toggle the translucid setting; confirm the background becomes transparent
- [x] 5.2 Confirm that switching between Papel/Pizarra/Bosque variants preserves the translucid state
- [x] 5.3 Confirm that disabling translucid restores the original opaque background
- [x] 5.4 Confirm the setting persists across app relaunches
