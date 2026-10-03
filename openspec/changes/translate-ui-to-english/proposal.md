## Why

Folioo's interface is written in Spanish while its code, comments and documentation are in
English. For an app published as an open-source alternative to Apple Notes, English is the
wider audience and the language the rest of the project already speaks. The split also makes
the codebase inconsistent with itself: an English identifier holding a Spanish literal on the
next line.

## What Changes

- Every user-facing string becomes English: sidebar and navigation labels, the note list,
  editor toolbar tooltips, the find bar, all modals (settings, confirmations, new tag), the
  sketch canvas, and the backup export/import messages.
- The four backend error messages returned to the UI (empty tag name, name too long, duplicate
  tag, invalid backup file) become English, along with the two messages guarding tag deletion.
- Relative dates read "Today" / "Yesterday", and absolute dates format with `en-GB` instead of
  `es-ES`.
- The tags seeded on a **fresh** install become Work · Personal · Projects · Ideas. **Existing
  databases are untouched** — a user's own tags are their data, not UI strings, and renaming
  them would silently rewrite content they created.
- The GitHub release body becomes English.
- **No internationalization framework.** Strings stay inline. This is a translation, not an
  i18n migration; adding locale files would mean every new string lives in two places for a
  second language nobody has asked for yet.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `settings`: the appearance variant, colour mode and translucent-background labels are quoted
  in the spec in Spanish and now read in English.
- `trash`: the retention countdown strings quoted in the spec are now English.
- `backup-restore`: the export action label and the import confirmation message are now English.
- `notes-crud`: the reserved view names quoted in the spec are now English.
- `search`: the view names quoted in the scope requirement are now English.
- `translucid-background`: the toggle label and panel name are now English.

## Impact

- `src/components.jsx`, `src/App.jsx`, `src/settings.jsx`, `src/ui.jsx`, `src/draw.jsx`,
  `src/data.js` — inline literals.
- `src-tauri/src/lib.rs` — six error strings; `src-tauri/src/db.rs` — the first-run tag seed.
- `.github/workflows/release.yml` — the release body.
- **No schema, API or behaviour change.** Command names, note fields and the on-disk format are
  untouched, so an existing database opens unchanged and keeps its Spanish tag names.
- `index.html` keeps `lang="es"`, which becomes wrong — it is corrected to `lang="en"`.
