## 1. Frontend strings

- [x] 1.1 Translate `src/settings.jsx` — modal title, section headings, variant/mode/translucent
      labels, the KDE hint, and the data export/import actions
- [x] 1.2 Translate `src/components.jsx` — sidebar, note list, empty states, editor toolbar
      tooltips, table controls, find bar, and the new-tag modal
- [x] 1.3 Translate `src/App.jsx` — confirmation dialogs (trash, delete forever, empty trash,
      delete tag) and the backup status messages
- [x] 1.4 Translate `src/ui.jsx` — the close button label and the default confirm/cancel labels
- [x] 1.5 Translate `src/draw.jsx` — sketch toolbar and actions
- [x] 1.6 Translate `src/data.js` — "Today" / "Yesterday" and switch formatting to `en-GB`
- [x] 1.7 Set `lang="en"` on the document in `index.html`

## 2. Backend strings

- [x] 2.1 Translate the six error strings in `src-tauri/src/lib.rs` (empty tag name, name too
      long, duplicate tag, last tag, no fallback tag, invalid backup)
- [x] 2.2 Change the first-run tag seed in `src-tauri/src/db.rs` to Work · Personal · Projects ·
      Ideas, leaving `seed_folders_if_empty`'s guard intact so existing tags are never rewritten

## 3. Project surface

- [x] 3.1 Translate the release body in `.github/workflows/release.yml`

## 4. Verification

- [x] 4.1 Grep the frontend and backend for Spanish diacritics and common unaccented Spanish
      words; confirm only code comments and legacy-compatibility keys remain
- [x] 4.2 `cargo test` and `npm run build` pass
- [ ] 4.3 Launch the app and read every screen: sidebar, list, editor toolbar, find bar,
      settings, each confirmation dialog, the sketch modal and the new-tag modal
- [ ] 4.4 Open a database seeded with the old Spanish tags and confirm they are preserved
