// App.jsx — application shell, state, and wiring to the Rust/SQLite backend.
import React from "react";
import { Sidebar, NoteListPanel, Editor, NewTagModal } from "./components.jsx";
import { useSettings, SettingsModal } from "./settings.jsx";
import { ConfirmModal } from "./ui.jsx";
import { applyThemedIcon, applyWindowTheme } from "./appicon.js";
import { save, open } from "@tauri-apps/plugin-dialog";
import { listen } from "@tauri-apps/api/event";
import * as api from "./api.js";

export default function App() {
  // ── Appearance settings ──
  // Default mode is "auto" so a fresh install follows the system light/dark.
  const [settings, setSetting] = useSettings({ variant: "paper", theme: "auto", fontSize: 15, translucid: false });
  const [settingsOpen, setSettingsOpen] = React.useState(false);

  React.useEffect(() => {
    const mq = window.matchMedia("(prefers-color-scheme: dark)");
    // Resolve "auto" to the live system preference; "light"/"dark" pass through.
    const resolved = () =>
      settings.theme === "auto" ? (mq.matches ? "dark" : "light") : settings.theme;
    const apply = () => {
      const t = resolved();
      document.documentElement.className = `v-${settings.variant} t-${t}${settings.translucid ? " v-translucid" : ""}`;
      document.documentElement.style.setProperty("--font-size", settings.fontSize + "px");
      // Retint the window icon to match the active theme accent (best-effort).
      applyThemedIcon();
      // Keep the native window/titlebar in step: follow the system in "auto",
      // otherwise force the chosen mode.
      applyWindowTheme(settings.theme === "auto" ? null : t);
    };
    apply();
    // Repaint live when the system theme flips while in "auto" mode.
    if (settings.theme === "auto") {
      mq.addEventListener("change", apply);
      return () => mq.removeEventListener("change", apply);
    }
  }, [settings.variant, settings.theme, settings.fontSize, settings.translucid]);

  // ── Data (source of truth lives in SQLite via the backend) ──
  const [notes, setNotes] = React.useState([]);
  const [trash, setTrash] = React.useState([]);
  const [folders, setFolders] = React.useState([]);
  const [selectedId, setSelectedId] = React.useState(null);
  const [selectedFolder, setSelectedFolder] = React.useState(() => {
    try {
      // The "linux-notes-*" keys predate the rename; read them once so an
      // upgrade doesn't reset the last opened tag / note.
      return localStorage.getItem("folioo-folder") || localStorage.getItem("linux-notes-folder") || "all";
    } catch { return "all"; }
  });
  const [searchQuery, setSearchQuery] = React.useState("");
  // Debounced copy used for filtering, so the list isn't re-filtered on every keystroke.
  const [debouncedQuery, setDebouncedQuery] = React.useState("");
  React.useEffect(() => {
    if (!searchQuery) {
      setDebouncedQuery("");
      return;
    }
    const t = setTimeout(() => setDebouncedQuery(searchQuery), 150);
    return () => clearTimeout(t);
  }, [searchQuery]);
  const searchRef = React.useRef(null);

  const [newTag, setNewTag] = React.useState({ open: false, noteId: null, error: "" });
  // Generic confirmation dialog: { title, message, danger, confirmLabel, onConfirm } | null
  const [confirm, setConfirm] = React.useState(null);
  const [importMsg, setImportMsg] = React.useState("");

  // ── Folder sync ──
  const [syncStatus, setSyncStatus] = React.useState(null);
  const [syncMsg, setSyncMsg] = React.useState("");
  // Bumped when sync replaced the content of the note open in the editor.
  const [editorRevision, setEditorRevision] = React.useState(0);

  const trashMode = selectedFolder === "trash";

  // Initial load. Both lists hold lightweight summaries (no HTML content) —
  // the selected note's content is fetched on demand below.
  React.useEffect(() => {
    api.listFolders().then(setFolders).catch((e) => console.error("Load folders failed:", e));
    api
      .listNotes()
      .then((rows) => {
        setNotes(rows);
        // Restore the last opened note if it still exists.
        let saved = null;
        try { saved = localStorage.getItem("folioo-note") || localStorage.getItem("linux-notes-note"); } catch {}
        const pick = rows.find((n) => n.id === saved) || rows[0];
        if (pick) setSelectedId(pick.id);
      })
      .catch((e) => console.error("Load notes failed:", e));
    api.listTrash().then(setTrash).catch((e) => console.error("Load trash failed:", e));
  }, []);

  // Full note for the editor, loaded lazily whenever the selection changes.
  const [selectedNote, setSelectedNote] = React.useState(null);
  React.useEffect(() => {
    if (!selectedId) {
      setSelectedNote(null);
      return;
    }
    let stale = false;
    api
      .getNote(selectedId)
      .then((n) => { if (!stale) setSelectedNote(n); })
      .catch((e) => console.error("Load note failed:", e));
    return () => { stale = true; };
  }, [selectedId]);

  // Persist the current note / folder so they're restored next launch.
  React.useEffect(() => {
    try { localStorage.setItem("folioo-folder", selectedFolder); } catch {}
  }, [selectedFolder]);
  React.useEffect(() => {
    try { if (selectedId) localStorage.setItem("folioo-note", selectedId); } catch {}
  }, [selectedId]);

  // ── Derived: filtered + sorted ──
  const filteredNotes = React.useMemo(() => {
    const matchesQuery = (n) => {
      if (!debouncedQuery) return true;
      const q = debouncedQuery.toLowerCase();
      // searchText is the plain-text shadow of the content, precomputed in SQLite.
      return n.title.toLowerCase().includes(q) || (n.searchText || "").toLowerCase().includes(q);
    };

    if (trashMode) return trash.filter(matchesQuery); // already ordered by deletion date

    let list = notes;
    if (selectedFolder === "favorites") list = list.filter((n) => n.favorite);
    else if (selectedFolder === "recent")
      list = [...list].sort((a, b) => b.updated.localeCompare(a.updated)).slice(0, 8);
    else if (selectedFolder !== "all") list = list.filter((n) => n.folder === selectedFolder);

    return [...list.filter(matchesQuery)].sort((a, b) => b.updated.localeCompare(a.updated));
  }, [notes, trash, trashMode, selectedFolder, debouncedQuery]);

  // Keep a valid selection whenever the visible list changes.
  React.useEffect(() => {
    if (filteredNotes.length === 0) {
      if (selectedId !== null) setSelectedId(null);
    } else if (!filteredNotes.some((n) => n.id === selectedId)) {
      setSelectedId(filteredNotes[0].id);
    }
  }, [filteredNotes, selectedId]);

  // Summary shape (what the lists hold) derived from a freshly created note.
  const summaryOf = (n) => ({
    id: n.id,
    title: n.title,
    folder: n.folder,
    favorite: n.favorite,
    created: n.created,
    updated: n.updated,
    deletedAt: n.deletedAt,
    searchText: "",
  });

  // ── Notes CRUD ──
  // ── Folder sync: reacting to what other machines wrote ──
  // Sync events arrive outside React's render cycle, so the current selection
  // is read through refs rather than captured once.
  const selectedIdRef = React.useRef(selectedId);
  selectedIdRef.current = selectedId;
  const selectedNoteRef = React.useRef(selectedNote);
  selectedNoteRef.current = selectedNote;

  const reloadFromSync = React.useCallback(async () => {
    try {
      const [n, t, f] = await Promise.all([api.listNotes(), api.listTrash(), api.listFolders()]);
      setNotes(n);
      setTrash(t);
      setFolders(f);
      // The open note may have changed underneath the editor. The editor only
      // reloads its content when told to, and would otherwise autosave the old
      // version back over the one that just arrived.
      const id = selectedIdRef.current;
      const shown = selectedNoteRef.current;
      if (!id || !shown || shown.id !== id) return;
      const fresh = await api.getNote(id).catch(() => null);
      if (!fresh) return; // purged by sync: the list reload moves the selection
      const contentChanged = fresh.title !== shown.title || fresh.content !== shown.content;
      if (contentChanged || fresh.folder !== shown.folder || fresh.favorite !== shown.favorite || fresh.deletedAt !== shown.deletedAt) {
        setSelectedNote(fresh);
      }
      if (contentChanged) setEditorRevision((r) => r + 1);
    } catch (e) {
      console.error("Reload after sync failed:", e);
    }
  }, []);

  React.useEffect(() => {
    api.syncStatus().then(setSyncStatus).catch((e) => console.error("Sync status failed:", e));
    // listen() resolves asynchronously; if the effect is torn down first
    // (React strict mode mounts twice), unsubscribe as soon as it resolves.
    let disposed = false;
    const subscriptions = [
      listen("sync-status", (e) => setSyncStatus(e.payload)),
      listen("sync-changed", () => reloadFromSync()),
    ];
    return () => {
      disposed = true;
      subscriptions.forEach((p) => p.then((unlisten) => disposed && unlisten()));
    };
  }, [reloadFromSync]);

  // Sync replaced the open note while it had edits not yet autosaved: keep
  // those edits as a separate note before the editor loads the new version.
  const handleKeepLocalCopy = React.useCallback(async ({ title, content, folder }) => {
    try {
      const created = await api.createNote(folder);
      await api.updateNote(created.id, `${title || "Untitled"} (conflicted copy)`, content);
      setNotes(await api.listNotes());
    } catch (e) {
      console.error("Keeping the unsaved copy failed:", e);
    }
  }, []);

  const handleChooseSyncFolder = React.useCallback(async () => {
    setSyncMsg("");
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected) return;
      const path = Array.isArray(selected) ? selected[0] : selected;
      setSyncStatus(await api.syncSetFolder(path));
    } catch (e) {
      console.error("Choosing the sync folder failed:", e);
      setSyncMsg(typeof e === "string" ? e : "Could not use that folder.");
    }
  }, []);

  const handleSyncNow = React.useCallback(() => {
    setSyncMsg("");
    api.syncNow().catch((e) => console.error("Sync now failed:", e));
  }, []);

  const askStopSync = React.useCallback(() => {
    setConfirm({
      title: "Stop syncing",
      confirmLabel: "Stop syncing",
      message:
        "Folioo will stop syncing with this folder. Your notes stay on this computer, and the " +
        "folder keeps its copy — nothing is deleted.",
      onConfirm: async () => {
        try {
          setSyncStatus(await api.syncStop());
          setSyncMsg("");
        } catch (e) {
          console.error("Stopping sync failed:", e);
          setSyncMsg("Could not stop syncing.");
        }
      },
    });
  }, []);

  const handleCreate = React.useCallback(async () => {
    const fallback = folders[0]?.name || "Personal";
    const folder = ["all", "favorites", "recent", "trash"].includes(selectedFolder) ? fallback : selectedFolder;
    try {
      const note = await api.createNote(folder);
      if (trashMode) setSelectedFolder("all");
      setNotes((prev) => [summaryOf(note), ...prev]);
      setSelectedNote(note); // already complete — skip the lazy-load round trip
      setSelectedId(note.id);
    } catch (e) {
      console.error("Create failed:", e);
    }
  }, [selectedFolder, folders, trashMode]);

  const handleUpdate = React.useCallback(async (id, changes) => {
    try {
      const summary = await api.updateNote(id, changes.title, changes.content);
      setNotes((prev) => prev.map((n) => (n.id === id ? summary : n)));
      // Keep the editor's copy coherent (title/date shown in the meta row).
      setSelectedNote((prev) =>
        prev && prev.id === id
          ? { ...prev, title: summary.title, updated: summary.updated, content: changes.content }
          : prev
      );
    } catch (e) {
      console.error("Update failed:", e);
    }
  }, []);

  const handleToggleFavorite = React.useCallback(async (id) => {
    try {
      const favorite = await api.toggleFavorite(id);
      setNotes((prev) => prev.map((n) => (n.id === id ? { ...n, favorite } : n)));
      setSelectedNote((prev) => (prev && prev.id === id ? { ...prev, favorite } : prev));
    } catch (e) {
      console.error("Toggle favorite failed:", e);
    }
  }, []);

  const handleChangeFolder = React.useCallback(async (id, folder) => {
    try {
      const summary = await api.setNoteFolder(id, folder);
      setNotes((prev) => prev.map((n) => (n.id === id ? summary : n)));
      setSelectedNote((prev) =>
        prev && prev.id === id ? { ...prev, folder: summary.folder, updated: summary.updated } : prev
      );
    } catch (e) {
      console.error("Change tag failed:", e);
    }
  }, []);

  // ── Trash ──
  const askTrashNote = React.useCallback(
    (id) => {
      const note = notes.find((n) => n.id === id);
      if (!note) return;
      const title = note.title || "Untitled";
      setConfirm({
        title: "Move to trash",
        message: `"${title}" will be moved to the trash. You can restore it for 30 days.`,
        confirmLabel: "Move to trash",
        onConfirm: async () => {
          const summary = await api.deleteNote(id);
          setNotes((prev) => prev.filter((n) => n.id !== id));
          setTrash((prev) => [summary, ...prev]);
        },
      });
    },
    [notes]
  );

  const handleRestore = React.useCallback(async (id) => {
    try {
      const summary = await api.restoreNote(id);
      setTrash((prev) => prev.filter((n) => n.id !== id));
      setNotes((prev) => [summary, ...prev]);
    } catch (e) {
      console.error("Restore failed:", e);
    }
  }, []);

  const askPurgeNote = React.useCallback(
    (id) => {
      const note = trash.find((n) => n.id === id);
      const title = note?.title || "Untitled";
      setConfirm({
        title: "Delete permanently",
        danger: true,
        confirmLabel: "Delete",
        message: `"${title}" will be deleted forever. This cannot be undone.`,
        onConfirm: async () => {
          await api.purgeNote(id);
          setTrash((prev) => prev.filter((n) => n.id !== id));
        },
      });
    },
    [trash]
  );

  const askEmptyTrash = React.useCallback(() => {
    const count = trash.length;
    setConfirm({
      title: "Empty trash",
      danger: true,
      confirmLabel: "Empty trash",
      message: `${count} note${count === 1 ? "" : "s"} will be deleted forever. This cannot be undone.`,
      onConfirm: async () => {
        await api.emptyTrash();
        setTrash([]);
      },
    });
  }, [trash.length]);

  // ── Tags ──
  const openNewTag = React.useCallback((noteId) => {
    setNewTag({ open: true, noteId: noteId || null, error: "" });
  }, []);

  const handleCreateTag = React.useCallback(
    async (name, color) => {
      try {
        const folder = await api.createFolder(name, color);
        setFolders((prev) => [...prev, folder]);
        if (newTag.noteId) await handleChangeFolder(newTag.noteId, folder.name);
        setNewTag({ open: false, noteId: null, error: "" });
      } catch (e) {
        setNewTag((prev) => ({ ...prev, error: String(e) }));
      }
    },
    [newTag.noteId, handleChangeFolder]
  );

  // Per-folder and favorites counts in a single pass (Sidebar badges).
  const counts = React.useMemo(() => {
    const byFolder = {};
    let favorites = 0;
    for (const n of notes) {
      byFolder[n.folder] = (byFolder[n.folder] || 0) + 1;
      if (n.favorite) favorites += 1;
    }
    return { byFolder, favorites };
  }, [notes]);

  const askDeleteTag = React.useCallback(
    (folder) => {
      const used = counts.byFolder[folder.name] || 0;
      const fallback = folders.find((f) => f.name !== folder.name)?.name;
      setConfirm({
        title: "Delete tag",
        danger: true,
        confirmLabel: "Delete",
        message:
          used > 0
            ? `"${folder.name}" will be deleted. Its ${used} note${used === 1 ? "" : "s"} will move to "${fallback}".`
            : `The tag "${folder.name}" will be deleted. This cannot be undone.`,
        onConfirm: async () => {
          const { fallback: moved } = await api.deleteFolder(folder.name);
          setFolders((prev) => prev.filter((f) => f.name !== folder.name));
          setNotes((prev) => prev.map((n) => (n.folder === folder.name ? { ...n, folder: moved } : n)));
          setSelectedNote((prev) => (prev && prev.folder === folder.name ? { ...prev, folder: moved } : prev));
          if (selectedFolder === folder.name) setSelectedFolder("all");
        },
      });
    },
    [counts, folders, selectedFolder]
  );

  const handleFolderSelect = React.useCallback((folder) => {
    setSelectedFolder(folder);
    setSearchQuery("");
  }, []);

  const handleOpenSettings = React.useCallback(() => setSettingsOpen(true), []);

  // ── Backup: export / import ──
  const handleExport = React.useCallback(async () => {
    try {
      const path = await save({
        defaultPath: "folioo-backup.json",
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!path) return;
      await api.exportToPath(path);
      setImportMsg("Backup exported successfully.");
    } catch (e) {
      console.error("Export failed:", e);
      setImportMsg("Could not export the backup.");
    }
  }, []);

  const handleImport = React.useCallback(async () => {
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: "JSON", extensions: ["json"] }],
      });
      if (!selected) return;
      const path = Array.isArray(selected) ? selected[0] : selected;
      const count = await api.importFromPath(path);
      const [n, t, f] = await Promise.all([api.listNotes(), api.listTrash(), api.listFolders()]);
      setNotes(n);
      setTrash(t);
      setFolders(f);
      setImportMsg(`Imported ${count} note${count === 1 ? "" : "s"}.`);
    } catch (e) {
      console.error("Import failed:", e);
      setImportMsg("Could not import the file.");
    }
  }, []);

  // ── Global keyboard shortcuts ──
  React.useEffect(() => {
    const handler = (e) => {
      if (e.ctrlKey || e.metaKey) {
        if (e.key === "n") { e.preventDefault(); handleCreate(); }
        if (e.key === "f") { e.preventDefault(); searchRef.current?.focus(); }
      }
    };
    window.addEventListener("keydown", handler);
    return () => window.removeEventListener("keydown", handler);
  }, [handleCreate]);

  return (
    <div className="app-root">
      <Sidebar
        noteCount={notes.length}
        favCount={counts.favorites}
        folderCounts={counts.byFolder}
        folders={folders}
        trashCount={trash.length}
        selectedFolder={selectedFolder}
        onSelectFolder={handleFolderSelect}
        onNewTag={openNewTag}
        onDeleteTag={askDeleteTag}
        onOpenSettings={handleOpenSettings}
      />
      <NoteListPanel
        notes={filteredNotes}
        selectedId={selectedId}
        trashMode={trashMode}
        onSelectNote={setSelectedId}
        onCreateNote={handleCreate}
        onEmptyTrash={askEmptyTrash}
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        searchRef={searchRef}
      />
      <Editor
        note={selectedNote}
        folders={folders}
        trashMode={trashMode}
        onUpdate={handleUpdate}
        onDelete={askTrashNote}
        onToggleFavorite={handleToggleFavorite}
        onChangeFolder={handleChangeFolder}
        onNewTag={openNewTag}
        onRestore={handleRestore}
        onPurge={askPurgeNote}
        revision={editorRevision}
        onKeepLocalCopy={handleKeepLocalCopy}
      />

      <NewTagModal
        open={newTag.open}
        error={newTag.error}
        onClose={() => setNewTag({ open: false, noteId: null, error: "" })}
        onCreate={handleCreateTag}
      />

      <SettingsModal
        open={settingsOpen}
        onClose={() => { setSettingsOpen(false); setImportMsg(""); setSyncMsg(""); }}
        settings={settings}
        onChange={setSetting}
        onExport={handleExport}
        onImport={handleImport}
        importMsg={importMsg}
        sync={{
          status: syncStatus,
          message: syncMsg,
          onChooseFolder: handleChooseSyncFolder,
          onSyncNow: handleSyncNow,
          onStop: askStopSync,
        }}
      />

      {/* Last, so it stacks above Settings when opened from there. */}
      <ConfirmModal
        open={!!confirm}
        title={confirm?.title}
        message={confirm?.message}
        danger={confirm?.danger}
        confirmLabel={confirm?.confirmLabel}
        onConfirm={() => {
          const c = confirm;
          setConfirm(null);
          c?.onConfirm?.();
        }}
        onClose={() => setConfirm(null)}
      />
    </div>
  );
}
