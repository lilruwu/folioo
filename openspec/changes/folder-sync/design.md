## Context

Folioo is local-first: a Tauri 2 shell, a React frontend that reaches the backend only through
`invoke()`, and a SQLite database in the platform app-data directory. The webview has no network
egress, and after this change neither does the backend — sync is file I/O against a folder the
user picked.

Phase 1 of this change is already implemented and shapes everything below:

- **`notes.updated_ms`** — a millisecond modification time stamped by every write path, because
  the existing `updated` (`YYYY-MM-DD`) cannot order two edits made on the same day.
- **`deletions (id, deleted_ms)`** — a tombstone written in the same transaction as every
  permanent purge. Trashing a note writes none: a trashed note is still a row carrying
  `deleted_at` and syncs as an ordinary note.
- **`meta`** — a key/value table, holding `folders_updated_ms` for the tag list.

The folder is not something Folioo controls. A third-party tool (rclone, Syncthing, Nextcloud,
Dropbox…) copies it between machines, on its own schedule, with its own conflict handling, and
it may be absent entirely — an unmounted rclone mount, an unplugged drive. The design has to be
safe against every one of those.

Notes embed images as base64 inside their HTML `content`, so a single note can be megabytes.

## Goals / Non-Goals

**Goals:**

- A user points Folioo at a folder once; notes, trash state and tags then converge across every
  machine pointed at the same (externally synced) folder.
- No edit is ever silently lost — including edits the sync tool itself split into conflict files.
- Permanent deletions stick, and nothing is ever deleted *because a file is missing*.
- No network, no account, no credential anywhere in Folioo.
- Everything Folioo does today keeps working with no folder set or the folder unavailable.

**Non-Goals:**

- **Integrating any cloud provider directly** (Google Drive, Dropbox, OneDrive, iCloud). Each
  requires registering the app, publishing a homepage and privacy policy, and shipping
  credentials. Users reach those services through their own sync tool instead.
- Moving files between machines. That is the external tool's job.
- Real-time collaboration, or character-level merging of conflicting edits.
- Encryption of the library. The folder is the user's; they choose where it goes.

## Decisions

### The library lives in a `Folioo/` subfolder, marked by `library.json`

Choosing `~/Dropbox` must not scatter note files across the user's Dropbox root, so Folioo
creates and uses `<chosen>/Folioo/`. If the chosen folder already *is* a library (it contains
`library.json`), it is used as-is — that is what a second machine sees when the user picks the
synced folder directly.

`library.json` carries `{ "format": 1 }`. A library with a newer `format` than this build
understands is refused rather than half-read.

```
Folioo/
  library.json            marker + format version
  notes/<id>.json         one file per note (active or trashed)
  deleted/<id>.json       one tombstone per permanently deleted note
  tags.json               the whole tag list
```

*Alternative considered:* using the chosen folder directly as the library. Rejected because a
user's first instinct is to pick the root of their synced folder.

### Absence never deletes

This is the rule the whole design leans on. A note that exists locally but has no file in the
library is **uploaded**, never deleted — whether or not it was synced before. Deletion only ever
comes from an explicit tombstone in `deleted/`.

The reason is the unmounted folder. An rclone mount point that isn't mounted is an ordinary empty
directory; a design that inferred deletion from absence would read it as "the user deleted every
note" and purge the database. With this rule the worst an empty folder can cause is a
re-upload.

Beyond that, sync refuses to run at all when `library.json` is missing from a library that was
previously synced, and reports the folder as unavailable. A fresh, empty folder is only
initialised when the user chooses it in Settings — never silently during a background sync.

*Alternative considered:* deleting a previously synced note whose file disappeared, the way many
two-way sync tools do. Rejected: the failure mode is total data loss, and it triggers on the
most ordinary of events (a mount that didn't come up after a reboot).

### One file per note, one tombstone per deletion, written atomically

Per-note files mean two machines only collide when they touch the same note, and a sync moves
only what changed — decisive when notes carry images. Tombstones are per-note files for the same
reason: a single shared deletions file would be rewritten by every machine and would itself
become a conflict hotspot for the external tool.

Every write goes to a temporary file in the same directory and is then renamed over the target.
Rename is atomic on a single filesystem, so the external tool never uploads a half-written note.

A note file holds every field of the note plus `updated_ms`, versioned by the library `format`.

### Change detection without reading every file

Per note, the local database keeps a sync record: the `updated_ms` at the last successful sync
(`synced_ms`) and the file's size and modification time as last seen. A pass lists `notes/` and
`deleted/` and only parses files whose size or mtime changed — so an unchanged library of
image-heavy notes costs a directory listing, not megabytes of JSON parsing.

### Classification

For every id seen locally, in `notes/`, or in `deleted/`:

- local-only, or local changed (`updated_ms > synced_ms`) while the file is unchanged → **write**
- file-only, or file changed while local is unchanged → **read into the database**
- both changed since `synced_ms` → **conflict**
- tombstone newer than the local copy → **purge locally** (writing the local tombstone too)
- tombstone older than the local copy → the note was edited after it was deleted elsewhere:
  **the edit wins**, the note is rewritten and the tombstone file removed
- local tombstone with no tombstone file → **write the tombstone file** and remove the note file

Notes whose file is missing fall under "local-only": they are written back (see *Absence never
deletes*).

### Conflicts keep both copies

The later `updated_ms` stays as the note; the other version is inserted under a fresh id, in the
same tag, titled with an English `(conflicted copy)` marker, and written to the library as a new
note. Losing a paragraph someone typed is a far worse failure than an extra note in the list.

### Conflict files made by the sync tool

Syncthing, Dropbox and Nextcloud resolve their own conflicts by writing a sibling file —
`<id>.sync-conflict-<date>-<device>.json`, `<id> (conflicted copy …).json`. Folioo only treats
`notes/<id>.json` with a well-formed id as the canonical note file. Any other `.json` in `notes/`
that parses as a Folioo note is imported as a conflicted copy (same rule as above) and then
removed from the folder, so the tool's conflict copy becomes a visible note instead of a file
nobody opens.

### Tags: the whole list, last writer wins

`tags.json` holds the complete tag list and the `folders_updated_ms` it was written at. The newer
of the local list and the file wins as a whole. That makes deleting or recolouring a tag
propagate — a name-by-name union could never express a deletion.

The cost: if two machines change tags between syncs, the older machine's tag edit is lost. No
note can be stranded by it, though: after every pass, any tag referenced by a note but absent
from the list is recreated.

### Scheduling

One sync at a time, guarded by a flag. A pass runs on startup when a folder is set, when the user
presses "Sync now", a few seconds after local edits settle (autosave fires every few hundred
milliseconds while typing; the debounce keeps sync from chasing keystrokes), and every 5 minutes
while the app is open — the only way to notice changes the external tool delivered from other
machines without adding a file-watcher dependency. Passes run off the UI thread; when one
finishes, a Tauri event tells the frontend to reload its lists.

### Choosing the folder, and the Flatpak sandbox

The folder is picked with the existing dialog plugin in directory mode, and its path stored in
`meta` — not `localStorage`, since the backend needs it at startup before any page loads.

Inside Flatpak, a folder chosen through the file-chooser portal is exposed via the document
portal. If that grant persists across launches, no manifest change is needed. If it does not, the
manifest gains `--filesystem=home` so the folder stays reachable. Which one applies is verified in
a real Flatpak build, not assumed.

## Risks / Trade-offs

- **Clock skew between machines** can make last-writer-wins pick the wrong copy. → Conflicts
  keep both copies, so skew costs an extra note, never a lost edit.
- **Backfilled `updated_ms` is coarse.** Pre-migration notes all sit at midnight of their
  `updated` date, so the first sync may mis-order same-day edits made on two machines. → Both
  copies are kept; real timestamps apply from the first sync on.
- **Absence-never-deletes means a note deleted by hand in the folder comes back.** → Intended.
  Deleting inside Folioo is the supported path; it writes a tombstone. The README says so.
- **Tag edits made on two machines between syncs lose the older one.** → Accepted for a much
  simpler model that can propagate deletions; notes are never left without their tag.
- **The external tool is invisible to Folioo.** Folioo cannot tell whether the folder has
  finished syncing from another machine. → Last-writer-wins plus conflict copies make an early
  or late pass safe; the 5-minute interval picks up whatever arrives later.
- **Tombstones expire after 90 days.** A machine offline for longer returns still holding a note
  that was deleted elsewhere and, finding no tombstone, re-uploads it. → The window covers a
  laptop shelved for a season; the resurrected note is visible and can be deleted again.
- **Flatpak folder access** may need a broad `--filesystem=home` if the portal grant doesn't
  persist. → Verified in a real Flatpak build before release.
- **A half-finished pass** (app closed mid-sync) leaves some notes synced and others not. →
  Each note is committed independently with its own sync record; the next pass resumes.

## Migration Plan

1. Phase 1 (schema) is shipped and idempotent; it leaves the app fully functional with no sync
   code reachable.
2. Sync stays inert until the user chooses a folder — no folder, not one file touched.
3. Choosing an empty folder initialises a library from the local database. Choosing a folder
   that already holds a library merges both sides; nothing is deleted by a first sync, because
   the only deletions come from tombstones.
4. Rollback: older builds ignore the added column and tables, so a downgrade leaves notes
   readable. The library folder is plain JSON and stays readable without Folioo at all.

## Resolved Decisions

- **No cloud provider integration.** Registering Folioo with Google (or any provider) requires
  a published homepage and privacy policy, and shipping credentials; the project will not take
  that on. Users bring their own sync tool.
- **The library is not encrypted.** It is plain JSON the user can read and recover by hand.
- **Tombstones are retained for 90 days**, locally and as files in the library.
- **User-facing strings are English**, including the `(conflicted copy)` marker.

## Open Questions

None outstanding.
