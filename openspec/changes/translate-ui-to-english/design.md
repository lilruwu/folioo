## Context

The UI strings are Spanish literals inlined in JSX, plus six error strings returned from Rust
commands and the four tags seeded on first run. There is no locale layer of any kind — no
translation files, no locale detection, no formatting abstraction. `data.js` formats dates with
`es-ES`, and `index.html` declares `lang="es"`.

## Goals / Non-Goals

**Goals:**

- Every user-facing string reads in English.
- No behaviour, schema or API change; an existing database opens and works identically.
- A user's own data — their tags, their note titles — is never rewritten.

**Non-Goals:**

- Introducing an i18n framework, locale files, or a language picker.
- Translating code comments, commit history, or the archived OpenSpec changes, which are a
  historical record.

## Decisions

### Inline English, no i18n layer

Strings stay where they are, just in English. Adding `i18next` or a message catalogue would put
every new string in two places to serve a second language nobody has asked for. The moment a
real second locale is wanted, the extraction is mechanical and can be its own change — and it
starts from a consistent English base rather than a mixed one.

### Seeded tags change; existing tags do not

`seed_folders_if_empty` only runs when the `folders` table is empty, so switching the defaults
to Work · Personal · Projects · Ideas affects fresh installs only. Renaming tags in an existing
database was considered and rejected: those rows are the user's content, notes reference them by
name, and rewriting them would be a silent, unrequested edit to their data.

The consequence is deliberate and worth stating: a user upgrading from a Spanish install keeps
Spanish tag names in an otherwise English UI. That is correct — they named those tags, or
accepted the names, and only they should change them.

### `en-GB` for date formatting

`formatDate` falls back to `toLocaleDateString` for anything older than yesterday. `en-GB` gives
`5 Feb` — day before month, no comma — which matches the compact `es-ES` output the list layout
was designed around. `en-US` would produce `Feb 5` and reads oddly next to a European date order.

## Risks / Trade-offs

- **A missed string leaves the UI mixed.** → Verified by grepping the frontend and backend for
  Spanish diacritics and for the specific unaccented Spanish words in use, and by reading the
  translated files back.
- **Specs quote UI strings verbatim.** Six capability specs name Spanish labels in their
  scenarios and would silently go stale. → Each is updated as a delta spec in this change.
- **Existing users see their familiar labels change.** → Unavoidable in a translation, and the
  affected surface is labels rather than data. Their notes and tags are untouched.
