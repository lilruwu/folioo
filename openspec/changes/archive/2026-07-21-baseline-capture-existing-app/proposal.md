## Why

The app already has a full, shipped feature set (rich-text notes, images, sketches, tables,
checklists, tags, search, trash, backup/restore, settings) but no OpenSpec specs exist yet —
`openspec/specs/` is empty. Without a baseline, there's nothing for future change proposals to
diff against, and no shared source of truth for what "correct" behavior currently means. This
change captures the existing behavior as specs, reverse-engineered from `desktop/src` and
`desktop/src-tauri/src`. It introduces no new capability and changes no code.

## What Changes

Add spec-only deltas for 11 capabilities covering the current app behavior end to end: note
CRUD/autosave, rich-text editing, image/sketch embeds, tables, checklists, tags & favorites,
search (list filter + in-note find), trash, backup/restore, appearance settings, and the SQLite
persistence layer. No implementation, UI, or schema changes accompany this change.

## Capabilities

### New Capabilities
- `notes-crud`: create, load, update (autosaved), reassign tag, and remove notes; last-opened note/tag restored on relaunch.
- `rich-text-editing`: the contenteditable note body — inline/block formatting, undo/redo, and Markdown-style shortcuts.
- `media-embeds`: inserting images (file picker, paste, drag-drop) and hand-drawn sketches into a note.
- `tables`: inserting and editing in-note tables (rows/columns, alignment, tabular paste).
- `checklists`: to-do items inside a note (toggle, reorder, Enter/Backspace flow).
- `tags-favorites`: tag (folder) management and per-note favorite marking.
- `search`: filtering the note list and finding matches inside the open note.
- `trash`: soft delete with a 30-day recovery window.
- `backup-restore`: exporting/importing all notes and tags as a JSON file.
- `settings`: appearance preferences (variant, light/dark/auto mode, font size) and their effect on the window/icon.
- `persistence`: the SQLite-backed storage layer notes and tags are read from and written to.

### Modified Capabilities
(none — `openspec/specs/` is currently empty)

## Impact

- No code changes. Specs are derived from `desktop/src/*.jsx`, `desktop/src/*.js`, and
  `desktop/src-tauri/src/{lib.rs,db.rs}`.
- Once archived, these become the baseline in `openspec/specs/` that subsequent change proposals
  (new features, bug fixes) should diff against.
