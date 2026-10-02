# Folioo — Desktop (Tauri + React + SQLite)

A native, fast desktop build of **Folioo**, an open-source Apple Notes
alternative for Linux. The UI is the same warm, paper-inspired design from the
prototype; the Rust backend stores notes in a local **SQLite** database.

> [!NOTE]
> This project is my playground for experimenting with OpenSpec and AI-Driven Development while building an application I've always wanted to have.

## Features

- Rich-text notes with autosave, tags (create/delete), favourites and search.
- **Images** inside notes — toolbar button, paste (Ctrl+V, incl. screenshots via
  the system clipboard), or drag & drop. Images are downscaled (max 1600 px) and
  embedded in the note, so everything stays in the DB.
- **Sketches** — a drawing canvas (pen colours/sizes, eraser, undo, clear) whose
  result is embedded as an image in the note, Apple-Notes style.
- **Tables** — insert/edit tables with in-table +/− handles to add/remove rows &
  columns, column alignment, and position-preserving cell copy/paste.
- **To-do checklists** — toolbar button; click the circle to tick items off.
  Enter continues the list, Enter on an empty item exits it. **Drag the ⠿ handle
  to reorder** items.
- **Find in note** — magnifier in the toolbar; highlights matches (CSS Custom
  Highlight API) with next/prev navigation.
- **Remembers** the last opened note and tag across launches.
- **Markdown shortcuts** while typing: `# `/`## `/`### ` headings, `- `/`* ` bullets,
  `1. ` numbered, `> ` quote, `[] ` to-do, and inline `**bold**`, `*italic*`, `` `code` ``.
- **Trash** with 30-day retention (restore / delete forever / empty), plus a
  Settings panel for theme, mode and font size.
- **Backup** — export all notes + tags to a JSON file and import it on another
  machine (Settings → Data). Import merges by id, so it's non-destructive.
- **Folder sync** — keep your notes in step across machines through a folder you
  choose and sync yourself (rclone, Syncthing, Nextcloud, Dropbox…). No accounts, no
  cloud run by Folioo. See [Sync between machines](#sync-between-machines).
- **Themed app icon** — a notepad mark whose accent follows the active theme
  (the window icon is retinted at runtime; honoured by desktops that show window icons).

## Stack

- **Shell / backend:** [Tauri 2](https://tauri.app) (Rust) — small binary, native Linux packaging.
- **UI:** React 18 + Vite (the exact prototype design, pixel-for-pixel).
- **Storage:** SQLite via `rusqlite` (bundled — no system SQLite needed). The
  database lives in the platform app-data directory (e.g.
  `~/.local/share/org.folioo.app/notes.db`). The app starts with **no notes**;
  a default set of **tags** (Work · Personal · Projects · Ideas) is seeded so
  notes can be categorised right away. Tags can be created and deleted from the UI.
  On first launch after the rename the app adopts the database from the old
  `org.linuxnotes.app` directory if it hasn't got one yet, so upgrading from
  Linux Notes keeps every note. This doesn't apply to **Flatpak**, where the old
  data sits in another sandbox the new app can't read — export a backup from the
  old build and import it here instead.

## Architecture

```
desktop/
├── index.html            # Vite entry
├── src/                  # React frontend
│   ├── main.jsx          # mount
│   ├── App.jsx           # shell + state, talks to the backend
│   ├── components.jsx    # Sidebar / NoteListPanel / Editor
│   ├── settings.jsx      # Settings modal: appearance, data, sync
│   ├── data.js           # folders + date/color helpers
│   ├── api.js            # invoke() wrappers → Rust commands
│   └── notes.css         # design system
└── src-tauri/            # Rust backend
    ├── src/lib.rs        # commands + app setup
    ├── src/db.rs         # SQLite schema, queries, first-run seed
    ├── src/sync/library.rs  # sync folder layout, atomic file writes
    ├── src/sync/engine.rs   # one sync pass: write, read, merge, delete
    ├── src/sync/service.rs  # background worker: when sync runs
    └── tauri.conf.json   # window + bundle config
```

The frontend persists notes exclusively through the backend
(`list_notes`, `create_note`, `update_note`, `delete_note`, `toggle_favorite`).
Only theme preferences are kept in `localStorage`.

## Sync between machines

Folioo doesn't run a cloud and doesn't sign you into one. Instead it keeps a copy of
your notes in **a folder you choose** (Settings → Sync → *Choose folder…*), and you
move that folder between machines with any tool you already trust. Point Folioo on
each machine at the same synced folder and they stay in step.

### What happens

- Choosing a folder creates a `Folioo/` subfolder inside it, holding one JSON file per
  note (`notes/`), one marker per permanently deleted note (`deleted/`), the tag list
  (`tags.json`) and `library.json`. It's plain JSON you can read without Folioo.
- **Your edits are written** to the folder about 2 seconds after you stop typing, and
  whatever is still pending when you close the app.
- **Changes from other machines are read** when Folioo starts, every 5 minutes, and
  when you press *Sync now*.
- **Edited the same note on two machines before they synced?** The newer version stays
  and the other appears as a separate note marked *(conflicted copy)*. Nothing is lost.
- **Deleting a note inside Folioo propagates** to the other machines. Deleting a file
  by hand in the folder does *not* delete the note — Folioo writes it back. A missing
  file is never treated as a deletion, so an unmounted or empty folder can't wipe
  your notes: Settings just reports the folder as unavailable.
- The folder is a mirror, not a history: a note you delete is deleted everywhere. Your
  cloud provider's own version history and trash, and Folioo's 30-day trash, are the
  way back.
- *Stop syncing* forgets the folder. Your local notes and the folder's contents are
  both left as they are.

### Example: Google Drive with rclone

Google has no official Drive client for Linux; [rclone](https://rclone.org) fills the
gap. On each machine:

```bash
rclone config                       # add a remote, e.g. "gdrive" → Google Drive; log in once in the browser
mkdir -p ~/GoogleDrive
rclone mount gdrive: ~/GoogleDrive --vfs-cache-mode writes --daemon
```

Then choose `~/GoogleDrive` in Folioo. `--vfs-cache-mode writes` is what rclone
recommends for applications that write files; give it a few seconds to upload before
suspending the machine. The same works for OneDrive, Dropbox, S3 and anything else
rclone supports. If you'd rather keep a local copy than a mount, `rclone bisync` on a
timer works too.

### Example: Syncthing (no cloud at all)

Share a folder between your machines with [Syncthing](https://syncthing.net) and choose
it in Folioo on each one. Notes travel directly between your devices.

### Why no built-in Google Drive / iCloud / Dropbox?

Connecting to a provider directly means registering Folioo with it, publishing a
homepage and a privacy policy, and shipping credentials inside the app — and iCloud
offers no API a Linux app can use at all. Folder sync gives you the same result with
the tool of your choice, and keeps Folioo entirely offline.

## Develop

Requires Rust, Node 18+, and the Tauri Linux system deps
(`libwebkit2gtk-4.1-dev`, `libgtk-3-dev`, `librsvg2-dev`).

```bash
cd desktop
npm install
npm run tauri dev      # launches the app with hot-reload
```

## Build a release bundle

```bash
cd desktop
npm run tauri build -- --bundles deb,rpm,appimage   # in src-tauri/target/release/bundle
```

- **`.deb`** — Debian/Ubuntu (`sudo dpkg -i`).
- **`.rpm`** — Fedora/openSUSE (`sudo dnf install ./*.rpm`). Tauri builds it with a
  pure-Rust packer, so no `rpmbuild` is required.
- **`.AppImage`** — distro-agnostic, just `chmod +x` and run.

Verify the `.deb` is well-formed before installing:

```bash
ar t "src-tauri/target/release/bundle/deb/Folioo_0.1.0_amd64.deb"
# should print: debian-binary / control.tar.gz / data.tar.gz
```

### Flatpak

The Flatpak installs the built `.deb` into the **GNOME 46 runtime** (which provides
`webkit2gtk-4.1`). Manifest + metainfo live in `flatpak/`.

> **glibc caveat:** the `.deb` binary must be built against a glibc **≤** the
> runtime's (GNOME 46 ≈ glibc 2.38). **Ubuntu 22.04 (glibc 2.35)** works — which is
> why CI builds there. If you build the `.deb` on a newer distro (e.g. Ubuntu 24.04,
> glibc 2.39) the Flatpak will build but fail at launch with
> `GLIBC_2.39 not found`. In that case build the `.deb` in a 22.04 container, or just
> let the **release workflow** produce the `.flatpak`.

To build locally (on Ubuntu 22.04, or adjust per the caveat above):

```bash
sudo apt install flatpak flatpak-builder      # or your distro's equivalent
flatpak install flathub org.gnome.Platform//46 org.gnome.Sdk//46

cd desktop
npm run tauri build -- --bundles deb
cp "src-tauri/target/release/bundle/deb/"*amd64.deb flatpak/folioo.deb

flatpak-builder --user --force-clean --repo flatpak-repo flatpak-build flatpak/org.folioo.app.yml
flatpak build-bundle flatpak-repo folioo.flatpak org.folioo.app \
  --runtime-repo=https://flathub.org/repo/flathub.flatpakrepo
# install & run:
flatpak install --user folioo.flatpak && flatpak run org.folioo.app
```

## Automated releases (GitHub Actions)

`.github/workflows/release.yml` builds the `.deb`, `.rpm`, `.AppImage` and a
`.flatpak` in the cloud and publishes a **GitHub Release** whenever you push a
version tag (the Flatpak runs as a second job that reuses the `.deb`). The app version in
`package.json`, `Cargo.toml` and `tauri.conf.json` is synced from the tag
automatically, so all you do is:

```bash
git tag v0.2.0
git push origin v0.2.0
```

The workflow (also runnable from the Actions tab via "Run workflow") creates a
**draft** release with the installers attached — review it and hit *Publish*.
No secrets to configure: it uses the built-in `GITHUB_TOKEN`.

### Troubleshooting: "malformed archive" / corrupt `.deb`

This almost always means the build **failed** (often the linker was OOM-killed on
a low-memory machine such as WSL) and `ar`/`dpkg` is reading a stale `.deb` from a
previous attempt. Fixes:

```bash
rm -rf src-tauri/target/release/bundle   # remove stale artifacts
npm run tauri build                      # must end with "Finished 1 bundle at ..."
```

The `release` profile here deliberately avoids `lto`/`codegen-units = 1` so the
final link stays light on memory. If you still hit OOM, give WSL more RAM via
`~/.wslconfig` (`[wsl2]\nmemory=8GB`) or build the AppImage instead
(`npm run tauri build --bundles appimage`).
