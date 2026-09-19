// components.jsx — Sidebar, NoteListPanel, Editor, NewTagModal
import React from "react";
import { formatDate, colorOf, TAG_PALETTE, trashDaysLeft } from "./data.js";
import { Modal } from "./ui.jsx";
import { readClipboardImageDataUrl } from "./clipboard.js";
import { DrawModal } from "./draw.jsx";

// ── ICONS ──────────────────────────────────────────────

function IcLines() {
  return (
    <svg className="nav-icon" viewBox="0 0 16 16" fill="none">
      <rect x="2" y="3" width="12" height="1.8" rx="0.9" fill="currentColor" />
      <rect x="2" y="7.1" width="12" height="1.8" rx="0.9" fill="currentColor" />
      <rect x="2" y="11.2" width="7.5" height="1.8" rx="0.9" fill="currentColor" />
    </svg>
  );
}

function IcStar({ filled }) {
  return (
    <svg className="nav-icon" viewBox="0 0 16 16" fill="none">
      <path
        d="M8 1.8 9.7 5.3l3.8.55-2.75 2.68.65 3.8L8 10.4l-3.4 1.93.65-3.8L2.5 5.85l3.8-.55z"
        fill={filled ? "#F5A623" : "none"}
        stroke={filled ? "#F5A623" : "currentColor"}
        strokeWidth="1.35"
        strokeLinejoin="round"
      />
    </svg>
  );
}

function IcClock() {
  return (
    <svg className="nav-icon" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="5.5" stroke="currentColor" strokeWidth="1.4" />
      <path d="M8 5v3l2 2" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

// Small filled tag dot used for folders/tags.
function TagDot({ color, size = 11 }) {
  return (
    <svg className="nav-icon" width={size} height={size} viewBox="0 0 16 16" fill="none">
      <path
        d="M2.5 3.2A.7.7 0 013.2 2.5h4.4c.2 0 .37.08.5.22l5.18 5.17a.7.7 0 010 1L9.4 13.06a.7.7 0 01-1 0L3.22 7.9a.7.7 0 01-.22-.5z"
        fill={color}
      />
      <circle cx="5.6" cy="5.6" r="1" fill="#fff" opacity="0.9" />
    </svg>
  );
}

function IcTrashNav() {
  return (
    <svg className="nav-icon" viewBox="0 0 16 16" fill="none">
      <path d="M3 4.5h10" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
      <path d="M6 4.5V3.4a.7.7 0 01.7-.7h2.6a.7.7 0 01.7.7v1.1" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
      <path d="M4.2 4.5l.6 8a1 1 0 001 .95h4.4a1 1 0 001-.95l.6-8" stroke="currentColor" strokeWidth="1.4" strokeLinejoin="round" />
      <path d="M6.6 7v3.5M9.4 7v3.5" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
    </svg>
  );
}

function IcRestore() {
  return (
    <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
      <path d="M3.5 8a4.5 4.5 0 104.5-4.5c-1.7 0-3.2.95-4 2.35" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" />
      <path d="M3 3v2.9h2.9" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function IcPlus({ size = 13 }) {
  return (
    <svg width={size} height={size} viewBox="0 0 13 13" fill="none">
      <path d="M6.5 2v9M2 6.5h9" stroke="currentColor" strokeWidth="1.8" strokeLinecap="round" />
    </svg>
  );
}

function IcChevron() {
  return (
    <svg width="10" height="10" viewBox="0 0 12 12" fill="none">
      <path d="M3 4.5 6 7.5l3-3" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function IcCheck() {
  return (
    <svg width="12" height="12" viewBox="0 0 14 14" fill="none">
      <path d="M2.5 7.5 5.5 10.5 11.5 4" stroke="currentColor" strokeWidth="1.7" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function IcTrash({ size = 15 }) {
  return (
    <svg width={size} height={size} viewBox="0 0 15 15" fill="none">
      <path d="M3 4.5h9" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
      <path d="M5.5 4.5V3.5a.5.5 0 01.5-.5h3a.5.5 0 01.5.5v1" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" />
      <rect x="3.5" y="4.5" width="8" height="7.5" rx="1" stroke="currentColor" strokeWidth="1.3" />
      <path d="M6 7v3M9 7v3" stroke="currentColor" strokeWidth="1.2" strokeLinecap="round" />
    </svg>
  );
}

function IcImage() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <rect x="2" y="3" width="12" height="10" rx="1.6" stroke="currentColor" strokeWidth="1.3" />
      <circle cx="5.4" cy="6.2" r="1.1" fill="currentColor" />
      <path d="M3 11.5l3-3 2.2 2.2L10.5 8l2.5 2.6" stroke="currentColor" strokeWidth="1.3" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function IcCheckSquare() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <rect x="2.5" y="2.5" width="11" height="11" rx="2.4" stroke="currentColor" strokeWidth="1.3" />
      <path d="M5 8l2.1 2.1L11 5.6" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" strokeLinejoin="round" />
    </svg>
  );
}

function IcSearch() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <circle cx="7" cy="7" r="4.3" stroke="currentColor" strokeWidth="1.4" />
      <path d="M10.2 10.2L14 14" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
    </svg>
  );
}

function IcTable() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <rect x="2" y="3" width="12" height="10" rx="1.3" stroke="currentColor" strokeWidth="1.3" />
      <path d="M2 6.5h12M2 9.8h12M6.5 3v10" stroke="currentColor" strokeWidth="1.2" />
    </svg>
  );
}

function IcPencil() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <path d="M11.1 2.6l2.3 2.3-7.4 7.4-2.8.5.5-2.8 7.4-7.4z" stroke="currentColor" strokeWidth="1.3" strokeLinejoin="round" />
      <path d="M10 3.7l2.3 2.3" stroke="currentColor" strokeWidth="1.3" />
    </svg>
  );
}

function IcAlignLeft() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <path d="M2.5 4h11M2.5 7.3h7M2.5 10.6h9" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
    </svg>
  );
}
function IcAlignCenter() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <path d="M2.5 4h11M4.5 7.3h7M3.5 10.6h9" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
    </svg>
  );
}
function IcAlignRight() {
  return (
    <svg width="15" height="15" viewBox="0 0 16 16" fill="none">
      <path d="M2.5 4h11M6.5 7.3h7M4.5 10.6h9" stroke="currentColor" strokeWidth="1.4" strokeLinecap="round" />
    </svg>
  );
}

export function IcSettings() {
  return (
    <svg className="nav-icon" viewBox="0 0 16 16" fill="none">
      <circle cx="8" cy="8" r="2.2" stroke="currentColor" strokeWidth="1.4" />
      <path
        d="M8 1.5v1.2M8 13.3v1.2M1.5 8h1.2M13.3 8h1.2M3.4 3.4l.85.85M11.75 11.75l.85.85M3.4 12.6l.85-.85M11.75 4.25l.85-.85"
        stroke="currentColor"
        strokeWidth="1.4"
        strokeLinecap="round"
      />
    </svg>
  );
}

// ── SIDEBAR ────────────────────────────────────────────

function NavItem({ icon, label, count, selected, onClick }) {
  return (
    <div className={`nav-item${selected ? " active" : ""}`} onClick={onClick}>
      {icon}
      <span className="nav-label">{label}</span>
      {count != null && <span className="nav-count">{count}</span>}
    </div>
  );
}

export const Sidebar = React.memo(function Sidebar({ noteCount, favCount, folderCounts, folders, trashCount, selectedFolder, onSelectFolder, onNewTag, onDeleteTag, onOpenSettings }) {
  return (
    <aside className="sidebar">
      <div className="sidebar-title">Notes</div>
      <nav className="sidebar-nav">
        <NavItem icon={<IcLines />} label="All notes" count={noteCount} selected={selectedFolder === "all"} onClick={() => onSelectFolder("all")} />
        <NavItem icon={<IcStar />} label="Favourites" count={favCount} selected={selectedFolder === "favorites"} onClick={() => onSelectFolder("favorites")} />
        <NavItem icon={<IcClock />} label="Recent" count={null} selected={selectedFolder === "recent"} onClick={() => onSelectFolder("recent")} />
        <NavItem icon={<IcTrashNav />} label="Trash" count={trashCount || null} selected={selectedFolder === "trash"} onClick={() => onSelectFolder("trash")} />

        <div className="sidebar-divider" />
        <div className="sidebar-section-row">
          <span className="sidebar-section-label">Tags</span>
          <button className="tag-add-btn" title="New tag" onClick={() => onNewTag(null)}>
            <IcPlus size={11} />
          </button>
        </div>

        {folders.map((f) => (
          <div
            key={f.name}
            className={`nav-item tag-item${selectedFolder === f.name ? " active" : ""}`}
            onClick={() => onSelectFolder(f.name)}
          >
            <TagDot color={f.color} />
            <span className="nav-label">{f.name}</span>
            <span className="nav-count">{folderCounts[f.name] || 0}</span>
            {folders.length > 1 && (
              <button
                className="tag-del-btn"
                title="Delete tag"
                onClick={(e) => {
                  e.stopPropagation();
                  onDeleteTag(f);
                }}
              >
                <IcTrash size={13} />
              </button>
            )}
          </div>
        ))}
      </nav>

      <div className="sidebar-footer">
        <NavItem icon={<IcSettings />} label="Settings" count={null} selected={false} onClick={onOpenSettings} />
      </div>
    </aside>
  );
});

// ── NOTE LIST ──────────────────────────────────────────

// Memoized: after an autosave only the edited note's summary changes identity,
// so the rest of the list doesn't re-render.
const NoteCard = React.memo(function NoteCard({ note, selected, trashMode, onSelect }) {
  const days = trashMode ? trashDaysLeft(note.deletedAt) : null;
  return (
    <div className={`note-card${selected ? " selected" : ""}`} onClick={() => onSelect(note.id)}>
      <div className="note-card-title">
        {note.favorite && !trashMode && <span className="note-fav-dot" />}
        {note.title || <span style={{ opacity: 0.38 }}>Untitled</span>}
      </div>
      <div className="note-card-date">
        {trashMode ? (days === 0 ? "Deleted today" : `${days} day${days === 1 ? "" : "s"} left`) : formatDate(note.updated)}
      </div>
    </div>
  );
});

export function NoteListPanel({ notes, selectedId, trashMode, onSelectNote, onCreateNote, onEmptyTrash, searchQuery, onSearchChange, searchRef }) {
  return (
    <div className="note-list-panel">
      <div className="list-header">
        {trashMode ? (
          <button className="empty-trash-btn" onClick={onEmptyTrash} disabled={notes.length === 0}>
            Vaciar papelera
          </button>
        ) : (
          <button className="new-note-btn" onClick={onCreateNote}>
            <IcPlus />
            <span>New note</span>
          </button>
        )}
        <input
          ref={searchRef}
          className="search-box"
          type="text"
          placeholder={trashMode ? "Search the trash…" : "Search notes…"}
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
        />
      </div>
      {trashMode && notes.length > 0 && (
        <div className="trash-hint">Notes are deleted after 30 days in the trash.</div>
      )}
      <div className="notes-scroll">
        {notes.length === 0 ? (
          <div className="empty-state">
            {searchQuery ? "No results" : trashMode ? "The trash is empty" : "No notes in this folder"}
          </div>
        ) : (
          notes.map((note) => (
            <NoteCard key={note.id} note={note} selected={note.id === selectedId} trashMode={trashMode} onSelect={onSelectNote} />
          ))
        )}
      </div>
    </div>
  );
}

// ── TAG DROPDOWN (editor) ──────────────────────────────

function TagDropdown({ note, folders, onChangeFolder, onNewTag }) {
  const [open, setOpen] = React.useState(false);
  const ref = React.useRef(null);
  const color = colorOf(folders, note.folder);

  React.useEffect(() => {
    if (!open) return;
    const onDoc = (e) => {
      if (ref.current && !ref.current.contains(e.target)) setOpen(false);
    };
    window.addEventListener("mousedown", onDoc);
    return () => window.removeEventListener("mousedown", onDoc);
  }, [open]);

  // Collapse the menu whenever the selected note changes.
  React.useEffect(() => setOpen(false), [note.id]);

  return (
    <div className="tag-dropdown" ref={ref}>
      <button
        className="folder-chip folder-chip-btn"
        style={{ background: color + "20", color }}
        onClick={() => setOpen((o) => !o)}
        title="Change tag"
      >
        <span className="folder-chip-dot" style={{ background: color }} />
        {note.folder}
        <IcChevron />
      </button>

      {open && (
        <div className="tag-menu" role="menu">
          {folders.map((f) => (
            <button
              key={f.name}
              className="tag-menu-item"
              role="menuitemradio"
              aria-checked={f.name === note.folder}
              onClick={() => {
                if (f.name !== note.folder) onChangeFolder(note.id, f.name);
                setOpen(false);
              }}
            >
              <span className="tag-menu-dot" style={{ background: f.color }} />
              <span className="tag-menu-name">{f.name}</span>
              {f.name === note.folder && (
                <span className="tag-menu-check">
                  <IcCheck />
                </span>
              )}
            </button>
          ))}
          <div className="tag-menu-sep" />
          <button
            className="tag-menu-item tag-menu-new"
            onClick={() => {
              setOpen(false);
              onNewTag(note.id);
            }}
          >
            <span className="tag-menu-plus">
              <IcPlus size={11} />
            </span>
            Nueva etiqueta…
          </button>
        </div>
      )}
    </div>
  );
}

// ── EDITOR ─────────────────────────────────────────────

// Read-only view shown when a trashed note is selected.
function TrashedNoteView({ note, onRestore, onPurge }) {
  const days = trashDaysLeft(note.deletedAt);
  return (
    <div className="editor-panel">
      <div className="trash-bar">
        <span className="trash-bar-text">
          In the trash · {days === 0 ? "deleted today" : `deleted in ${days} day${days === 1 ? "" : "s"}`}
        </span>
        <div className="trash-bar-actions">
          <button className="trash-action restore" onClick={() => onRestore(note.id)}>
            <IcRestore />
            <span>Restore</span>
          </button>
          <button className="trash-action purge" onClick={() => onPurge(note.id)}>
            <IcTrash size={13} />
            <span>Delete permanently</span>
          </button>
        </div>
      </div>
      <div className="editor-body">
        <div className="editor-title editor-title-static">{note.title || "Untitled"}</div>
        <div className="editor-content editor-content-static" dangerouslySetInnerHTML={{ __html: note.content }} />
      </div>
    </div>
  );
}

export function Editor({ note, folders, trashMode, onUpdate, onDelete, onToggleFavorite, onChangeFolder, onNewTag, onRestore, onPurge }) {
  const titleRef = React.useRef(null);
  const contentRef = React.useRef(null);
  const saveTimer = React.useRef(null);
  const activeId = React.useRef(null);
  const fileInputRef = React.useRef(null);
  const draggedTodoRef = React.useRef(null);

  // In-note find state
  const [find, setFind] = React.useState({ open: false, query: "", count: 0, index: 0 });
  const findRanges = React.useRef([]);

  // Drawing modal
  const [drawOpen, setDrawOpen] = React.useState(false);
  const [drawInitial, setDrawInitial] = React.useState(null); // PNG when re-editing
  const drawRangeRef = React.useRef(null);
  const editingImgRef = React.useRef(null); // the sketch <img> being re-edited

  // Undo/redo history (snapshot-based, since we mutate the DOM directly)
  const histRef = React.useRef({ stack: [], idx: -1 });

  // Save whatever the editor DOM currently holds, attributed to the note it
  // belongs to (activeId) \u2014 never to the `note` prop, which may already point
  // at a different note while a debounced save is still pending.
  const doSave = () => {
    if (!activeId.current || !contentRef.current) return;
    normalizeWithCaret(); // keep every line wrapped in a block (structural safety)
    recordHistory(); // commit an undo snapshot at each pause
    const title = (titleRef.current && titleRef.current.textContent.trim()) || "";
    // Strip the zero-width spaces used as empty-block caret anchors.
    const content = contentRef.current.innerHTML.replace(/\u200B/g, "");
    onUpdate(activeId.current, { title, content });
  };
  const doSaveRef = React.useRef(doSave);
  doSaveRef.current = doSave;

  // Run a pending debounced save right now. Called before the editor DOM is
  // replaced with another note, so edits are neither lost nor written under
  // the wrong note id.
  const flushSave = () => {
    if (saveTimer.current == null) return;
    clearTimeout(saveTimer.current);
    saveTimer.current = null;
    doSaveRef.current();
  };

  const scheduleSave = () => {
    clearTimeout(saveTimer.current);
    saveTimer.current = setTimeout(() => {
      saveTimer.current = null;
      doSaveRef.current();
    }, 400);
  };

  React.useEffect(() => {
    if (!note || !contentRef.current) {
      // The editable DOM is unmounted (no selection / trash view); forget the
      // active note so selecting it again reloads the content instead of
      // hitting the same-id guard below with an empty editor.
      activeId.current = null;
      return;
    }
    if (note.id === activeId.current) return;
    flushSave(); // persist pending edits of the previous note before swapping the DOM
    activeId.current = note.id;
    if (titleRef.current) titleRef.current.textContent = note.title;
    if (contentRef.current) {
      contentRef.current.innerHTML = note.content;
      injectTodoHandles(contentRef.current);
      normalizeRoot();
    }
    clearFind();
    setFind({ open: false, query: "", count: 0, index: 0 });
    resetHistory();
    if (!note.title && titleRef.current) {
      setTimeout(() => titleRef.current && titleRef.current.focus(), 60);
    }
  }, [note && note.id]);

  // \u2500\u2500 Caret <-> character offset (for history & normalization) \u2500\u2500
  const caretOffset = () => {
    const sel = window.getSelection();
    const root = contentRef.current;
    if (!sel || !sel.rangeCount || !root || !root.contains(sel.anchorNode)) return null;
    const range = sel.getRangeAt(0);
    const pre = range.cloneRange();
    pre.selectNodeContents(root);
    pre.setEnd(range.endContainer, range.endOffset);
    return pre.toString().length;
  };

  const restoreCaret = (offset) => {
    const root = contentRef.current;
    if (offset == null || !root) return;
    const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
    let remaining = offset;
    let node;
    while ((node = walker.nextNode())) {
      const len = node.nodeValue.length;
      if (remaining <= len) {
        const r = document.createRange();
        r.setStart(node, remaining);
        r.collapse(true);
        const sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(r);
        return;
      }
      remaining -= len;
    }
    const r = document.createRange();
    r.selectNodeContents(root);
    r.collapse(false);
    const sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(r);
  };

  // \u2500\u2500 Structural normalization: wrap loose root-level inline runs in <div> \u2500\u2500
  // Prevents WebKit from deleting unrelated content when a sibling list/block
  // is removed (the "delete a list and the text above vanishes" bug).
  const BLOCK_RE = /^(DIV|P|H1|H2|H3|UL|OL|BLOCKQUOTE|PRE|HR|TABLE)$/;
  const isBlockNode = (n) => n.nodeType === 1 && BLOCK_RE.test(n.tagName);

  const normalizeRoot = () => {
    const root = contentRef.current;
    if (!root) return;
    let run = [];
    const flush = (before) => {
      if (!run.length) return;
      const div = document.createElement("div");
      run.forEach((n) => div.appendChild(n));
      root.insertBefore(div, before);
      run = [];
    };
    let child = root.firstChild;
    while (child) {
      const next = child.nextSibling;
      if (isBlockNode(child)) flush(child);
      else run.push(child);
      child = next;
    }
    flush(null);
  };

  const hasLooseContent = () => {
    const root = contentRef.current;
    if (!root) return false;
    return Array.from(root.childNodes).some(
      (n) =>
        (n.nodeType === 3 && n.nodeValue.replace(/\u200B/g, "").trim() !== "") ||
        (n.nodeType === 1 && !isBlockNode(n) && n.tagName !== "BR")
    );
  };

  const normalizeWithCaret = () => {
    if (!hasLooseContent()) return;
    const off = caretOffset();
    normalizeRoot();
    restoreCaret(off);
  };

  // \u2500\u2500 Undo / redo \u2500\u2500
  const snapshot = () => ({
    html: contentRef.current ? contentRef.current.innerHTML : "",
    caret: caretOffset(),
  });

  const resetHistory = () => {
    histRef.current = { stack: [snapshot()], idx: 0 };
  };

  // Snapshots hold the full HTML — with embedded base64 images they can be
  // megabytes each, so the stack is capped by total size, not just entries.
  const MAX_HIST_ENTRIES = 120;
  const MAX_HIST_CHARS = 8_000_000; // ≈16 MB of string memory

  const recordHistory = () => {
    const h = histRef.current;
    const snap = snapshot();
    if (h.idx >= 0 && h.stack[h.idx] && h.stack[h.idx].html === snap.html) {
      h.stack[h.idx].caret = snap.caret;
      return;
    }
    h.stack = h.stack.slice(0, h.idx + 1);
    h.stack.push(snap);
    let total = 0;
    for (const s of h.stack) total += s.html.length;
    // Always keep the current state plus at least one undo step.
    while (h.stack.length > 2 && (h.stack.length > MAX_HIST_ENTRIES || total > MAX_HIST_CHARS)) {
      total -= h.stack.shift().html.length;
    }
    h.idx = h.stack.length - 1;
  };

  const persistCurrent = () => {
    if (!activeId.current || !contentRef.current) return;
    const title = (titleRef.current && titleRef.current.textContent.trim()) || "";
    const content = contentRef.current.innerHTML.replace(/\u200B/g, "");
    onUpdate(activeId.current, { title, content });
  };

  const applySnapshot = (snap) => {
    if (!snap || !contentRef.current) return;
    contentRef.current.innerHTML = snap.html;
    injectTodoHandles(contentRef.current);
    contentRef.current.focus();
    restoreCaret(snap.caret);
    persistCurrent();
    refreshMarks();
  };

  const undo = () => {
    recordHistory(); // capture the latest typing before stepping back
    const h = histRef.current;
    if (h.idx > 0) {
      h.idx -= 1;
      applySnapshot(h.stack[h.idx]);
    }
  };

  const redo = () => {
    const h = histRef.current;
    if (h.idx < h.stack.length - 1) {
      h.idx += 1;
      applySnapshot(h.stack[h.idx]);
    }
  };

  const fmt = (cmd, value) => {
    document.execCommand(cmd, false, value || undefined);
    contentRef.current && contentRef.current.focus();
    refreshMarks();
    scheduleSave();
  };

  // ── Block formatting (headings, quote, code) via deterministic DOM edits ──
  // execCommand("formatBlock") is unreliable in WebKitGTK (no-ops on empty
  // blocks, leaks formatting at the editor root), so we manage blocks ourselves.

  // The block element (direct child of the editor) holding the caret, or null.
  const currentBlockEl = () => {
    const sel = window.getSelection();
    if (!sel || !sel.anchorNode || !contentRef.current) return null;
    let n = sel.anchorNode;
    if (n.nodeType === 3) n = n.parentElement;
    while (n && n.parentElement && n.parentElement !== contentRef.current) n = n.parentElement;
    return n && n.nodeType === 1 && n.parentElement === contentRef.current ? n : null;
  };

  // Ensure the current line is wrapped in a real block element; returns it.
  const ensureLineBlock = () => {
    const existing = currentBlockEl();
    if (existing) return existing;
    // Loose inline content directly under the root (the first line of a note):
    // wrap everything into a single <div>.
    const div = document.createElement("div");
    while (contentRef.current.firstChild) div.appendChild(contentRef.current.firstChild);
    if (!div.firstChild) div.appendChild(document.createElement("br"));
    contentRef.current.appendChild(div);
    const r = document.createRange();
    r.selectNodeContents(div);
    r.collapse(false);
    const sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(r);
    return div;
  };

  const replaceBlockTag = (block, tag) => {
    const el = document.createElement(tag);
    while (block.firstChild) el.appendChild(block.firstChild);
    block.replaceWith(el);
    const r = document.createRange();
    const sel = window.getSelection();
    if (el.textContent.replace(/\u200B/g, "").length === 0) {
      // Empty block: seed a zero-width space so the caret lives in a real text
      // node. Typing then appends to it and stays inside the block — WebKitGTK
      // drops the formatting if you type into a block that holds only a <br>.
      el.textContent = "\u200B";
      r.setStart(el.firstChild, 1);
    } else {
      r.selectNodeContents(el);
      r.collapse(false);
    }
    r.collapse(true);
    sel.removeAllRanges();
    sel.addRange(r);
    return el;
  };

  // Always set the current line to `tag`.
  const setBlock = (tag) => {
    const block = ensureLineBlock();
    if (block) replaceBlockTag(block, tag);
    contentRef.current && contentRef.current.focus();
    refreshMarks();
    scheduleSave();
  };

  // Toggle the current line between `tag` and a normal paragraph (toolbar buttons).
  const toggleBlock = (tag) => {
    const block = ensureLineBlock();
    if (!block) return;
    const cur = block.tagName.toLowerCase();
    replaceBlockTag(block, cur === tag ? "div" : tag);
    contentRef.current && contentRef.current.focus();
    refreshMarks();
    scheduleSave();
  };

  const handleContentInput = () => {
    refreshMarks();
    scheduleSave();
  };

  // ── Active toolbar state ──
  const [marks, setMarks] = React.useState({});
  // Geometry of the active table + its rows/cols, in editor-body content coords.
  const [tableBox, setTableBox] = React.useState(null);
  const tableElRef = React.useRef(null);

  const measureTable = (table) => {
    if (!table) return setTableBox(null);
    const eb = table.closest(".editor-body");
    if (!eb) return setTableBox(null);
    const ebr = eb.getBoundingClientRect();
    const toX = (vx) => vx - ebr.left + eb.scrollLeft;
    const toY = (vy) => vy - ebr.top + eb.scrollTop;
    const tr0 = table.rows[0];
    const cols = tr0
      ? Array.from(tr0.cells).map((td) => {
          const r = td.getBoundingClientRect();
          return { left: toX(r.left), width: r.width };
        })
      : [];
    const rows = Array.from(table.rows).map((tr) => {
      const r = tr.getBoundingClientRect();
      return { top: toY(r.top), height: r.height };
    });
    const tr = table.getBoundingClientRect();
    setTableBox({ top: toY(tr.top), left: toX(tr.left), w: tr.width, h: tr.height, cols, rows });
  };

  const refreshMarks = React.useCallback(() => {
    const el = contentRef.current;
    const sel = window.getSelection();
    if (!el || !sel || !sel.anchorNode || !el.contains(sel.anchorNode)) return;
    let node = sel.anchorNode;
    if (node.nodeType === 3) node = node.parentElement;
    const blockEl = node && node.closest ? node.closest("h1,h2,h3,pre,blockquote") : null;
    const tableEl = node && node.closest ? node.closest("table") : null;
    const q = (c) => {
      try { return document.queryCommandState(c); } catch { return false; }
    };
    tableElRef.current = tableEl && el.contains(tableEl) ? tableEl : null;
    measureTable(tableElRef.current);
    setMarks({
      bold: q("bold"),
      italic: q("italic"),
      underline: q("underline"),
      ul: q("insertUnorderedList"),
      ol: q("insertOrderedList"),
      block: blockEl ? blockEl.tagName.toLowerCase() : "",
      inTable: !!tableElRef.current,
    });
  }, []);

  React.useEffect(() => {
    const handler = () => refreshMarks();
    document.addEventListener("selectionchange", handler);
    return () => document.removeEventListener("selectionchange", handler);
  }, [refreshMarks]);

  // ── To-do lists ──
  // Find the enclosing element matching `selector` for the current caret.
  const closestInContent = (selector) => {
    const sel = window.getSelection();
    if (!sel || !sel.anchorNode || !contentRef.current) return null;
    let node = sel.anchorNode;
    if (node.nodeType === 3) node = node.parentElement;
    const found = node && node.closest ? node.closest(selector) : null;
    return found && contentRef.current.contains(found) ? found : null;
  };

  const placeCaretInBody = (todo) => {
    const body = todo.querySelector(".todo-body");
    if (!body) return;
    contentRef.current.focus();
    const range = document.createRange();
    range.setStart(body, 0);
    range.collapse(true);
    const sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(range);
  };

  // True if the collapsed caret sits at the very start of `el`.
  const caretAtStartOf = (el) => {
    const sel = window.getSelection();
    if (!sel || !sel.isCollapsed || !sel.rangeCount || !el) return false;
    const r = sel.getRangeAt(0);
    if (!el.contains(r.startContainer)) return false;
    const test = document.createRange();
    test.selectNodeContents(el);
    test.setEnd(r.startContainer, r.startOffset);
    return test.toString().replace(/\u200B/g, "").length === 0;
  };

  const placeCaretStart = (el) => {
    const r = document.createRange();
    r.setStart(el, 0);
    r.collapse(true);
    const sel = window.getSelection();
    sel.removeAllRanges();
    sel.addRange(r);
  };

  const makeDragHandle = () => {
    const h = document.createElement("span");
    h.className = "todo-drag";
    h.setAttribute("contenteditable", "false");
    h.setAttribute("draggable", "true");
    h.setAttribute("aria-hidden", "true");
    h.textContent = "⠿";
    return h;
  };

  const createTodoEl = (text = "") => {
    const todo = document.createElement("div");
    todo.className = "todo";
    todo.setAttribute("data-done", "false");
    const check = document.createElement("span");
    check.className = "todo-check";
    check.setAttribute("contenteditable", "false");
    const body = document.createElement("div");
    body.className = "todo-body";
    body.textContent = text;
    todo.appendChild(makeDragHandle());
    todo.appendChild(check);
    todo.appendChild(body);
    return todo;
  };

  // Add drag handles to to-dos loaded from saved HTML that predate them.
  const injectTodoHandles = (root) => {
    if (!root) return;
    root.querySelectorAll(".todo").forEach((todo) => {
      if (!todo.querySelector(".todo-drag")) todo.insertBefore(makeDragHandle(), todo.firstChild);
    });
  };

  const insertTodo = () => {
    contentRef.current.focus();
    const existing = closestInContent(".todo");
    const todo = createTodoEl("");
    if (existing) {
      existing.parentNode.insertBefore(todo, existing.nextSibling);
    } else {
      const sel = window.getSelection();
      if (sel && sel.rangeCount) {
        const range = sel.getRangeAt(0);
        range.collapse(false);
        range.insertNode(todo);
      } else {
        contentRef.current.appendChild(todo);
      }
    }
    placeCaretInBody(todo);
    scheduleSave();
  };

  // Toggle a checkbox when its circle is clicked (event delegation).
  const handleContentClick = (e) => {
    // Single click on a sketch reopens it for editing.
    const sketch = e.target.closest && e.target.closest('img[data-sketch="1"]');
    if (sketch && contentRef.current.contains(sketch)) {
      openDrawEdit(sketch);
      return;
    }
    const check = e.target.closest && e.target.closest(".todo-check");
    if (check && contentRef.current.contains(check)) {
      const todo = check.closest(".todo");
      const done = todo.getAttribute("data-done") === "true";
      todo.setAttribute("data-done", done ? "false" : "true");
      scheduleSave();
    }
  };

  // ── Tables ──
  const makeCell = () => {
    const td = document.createElement("td");
    td.appendChild(document.createElement("br"));
    return td;
  };

  const currentCell = () => closestInContent("td") || closestInContent("th");

  const insertTable = (rows = 2, cols = 2) => {
    contentRef.current.focus();
    const table = document.createElement("table");
    table.className = "note-table";
    const tbody = document.createElement("tbody");
    for (let r = 0; r < rows; r++) {
      const tr = document.createElement("tr");
      for (let c = 0; c < cols; c++) tr.appendChild(makeCell());
      tbody.appendChild(tr);
    }
    table.appendChild(tbody);

    const block = currentBlockEl();
    if (block && block.parentNode) block.after(table);
    else {
      const sel = window.getSelection();
      if (sel && sel.rangeCount) sel.getRangeAt(0).insertNode(table);
      else contentRef.current.appendChild(table);
    }
    // a normal line after the table so the caret can escape below it
    const after = document.createElement("div");
    after.appendChild(document.createElement("br"));
    table.after(after);

    placeCaretStart(table.querySelector("td"));
    scheduleSave();
    refreshMarks();
  };

  // Add a row below the caret's row (used by Tab past the last cell).
  const addRow = () => {
    const td = currentCell();
    if (!td) return;
    const tr = td.parentNode;
    const ntr = document.createElement("tr");
    for (let i = 0; i < tr.children.length; i++) ntr.appendChild(makeCell());
    tr.after(ntr);
    placeCaretStart(ntr.children[0]);
    scheduleSave();
    measureTable(td.closest("table"));
  };

  // Append a row/column at the end (used by the in-table "+" handles).
  const addRowEnd = () => {
    const table = tableElRef.current;
    if (!table) return;
    const cols = table.querySelector("tr") ? table.querySelector("tr").children.length : 2;
    const tr = document.createElement("tr");
    for (let i = 0; i < cols; i++) tr.appendChild(makeCell());
    (table.querySelector("tbody") || table).appendChild(tr);
    scheduleSave();
    measureTable(table);
  };

  const addColumnEnd = () => {
    const table = tableElRef.current;
    if (!table) return;
    table.querySelectorAll("tr").forEach((tr) => tr.appendChild(makeCell()));
    scheduleSave();
    measureTable(table);
  };

  // ── Per-row / per-column actions (driven by the in-table handles) ──
  const deleteRowAt = (i) => {
    const table = tableElRef.current;
    if (!table || !table.rows[i]) return;
    if (table.rows.length <= 1) return deleteTable();
    table.deleteRow(i);
    scheduleSave();
    measureTable(table);
  };

  const deleteColumnAt = (i) => {
    const table = tableElRef.current;
    if (!table) return;
    const colCount = table.rows[0] ? table.rows[0].cells.length : 0;
    if (colCount <= 1) return deleteTable();
    Array.from(table.rows).forEach((tr) => tr.cells[i] && tr.deleteCell(i));
    scheduleSave();
    measureTable(table);
  };


  // Align the whole column the caret is in (Obsidian-style column alignment).
  const setColumnAlign = (align) => {
    const td = currentCell();
    if (!td) return;
    const idx = Array.from(td.parentNode.children).indexOf(td);
    td.closest("table").querySelectorAll("tr").forEach((tr) => {
      if (tr.children[idx]) tr.children[idx].style.textAlign = align;
    });
    scheduleSave();
  };

  const deleteTable = () => {
    const table = tableElRef.current || closestInContent("table");
    if (!table) return;
    const sib = table.nextElementSibling || table.previousElementSibling;
    table.remove();
    tableElRef.current = null;
    setTableBox(null);
    if (sib) placeCaretStart(sib);
    scheduleSave();
    refreshMarks();
  };

  // ── Paste tabular data into a table, preserving cell positions ──
  const parseGridFromText = (text) =>
    text
      .replace(/\r/g, "")
      .replace(/\n+$/, "")
      .split("\n")
      .map((line) => line.split("\t"));

  const parseGridFromHtml = (html) => {
    try {
      const doc = new DOMParser().parseFromString(html, "text/html");
      const table = doc.querySelector("table");
      if (!table) return null;
      return Array.from(table.rows).map((tr) => Array.from(tr.cells).map((td) => td.textContent));
    } catch {
      return null;
    }
  };

  const pasteGridIntoTable = (grid, targetCell) => {
    const table = targetCell.closest("table");
    const startRow = targetCell.parentNode.rowIndex;
    const startCol = targetCell.cellIndex;
    const needCols = startCol + Math.max(...grid.map((r) => r.length));
    const needRows = startRow + grid.length;
    // grow columns / rows so the pasted block fits
    while (table.rows[0].cells.length < needCols) {
      Array.from(table.rows).forEach((r) => r.insertCell(-1).appendChild(document.createElement("br")));
    }
    while (table.rows.length < needRows) {
      const tr = table.insertRow(-1);
      for (let c = 0; c < needCols; c++) tr.insertCell(-1).appendChild(document.createElement("br"));
    }
    // fill, preserving relative positions
    for (let i = 0; i < grid.length; i++) {
      const tr = table.rows[startRow + i];
      for (let j = 0; j < grid[i].length; j++) {
        const cell = tr.cells[startCol + j];
        if (!cell) continue;
        cell.textContent = grid[i][j];
        if (!cell.firstChild) cell.appendChild(document.createElement("br"));
      }
    }
    placeCaretStart(targetCell);
    scheduleSave();
    measureTable(table);
  };

  // ── Images ──
  // Insert an image (given as a data URL), downscaling if it's large.
  const insertDataUrl = (dataUrl, preferJpeg = false) =>
    new Promise((resolve) => {
      const img = new Image();
      img.onload = () => {
        const max = 1600;
        const scale = Math.min(1, max / Math.max(img.width, img.height));
        let url = dataUrl;
        if (scale < 1) {
          const w = Math.round(img.width * scale);
          const h = Math.round(img.height * scale);
          const canvas = document.createElement("canvas");
          canvas.width = w;
          canvas.height = h;
          canvas.getContext("2d").drawImage(img, 0, 0, w, h);
          url = canvas.toDataURL(preferJpeg ? "image/jpeg" : "image/png", 0.85);
        }
        contentRef.current.focus();
        document.execCommand("insertImage", false, url);
        scheduleSave();
        resolve();
      };
      img.onerror = () => resolve();
      img.src = dataUrl;
    });

  const embedImageFile = (file) => {
    const reader = new FileReader();
    reader.onload = () => insertDataUrl(reader.result, file.type !== "image/png");
    reader.readAsDataURL(file);
  };

  const handleFileChange = (e) => {
    const file = e.target.files && e.target.files[0];
    if (file) embedImageFile(file);
    e.target.value = "";
  };

  // ── Drawing ──
  const openDraw = () => {
    const sel = window.getSelection();
    drawRangeRef.current =
      sel && sel.rangeCount && contentRef.current && contentRef.current.contains(sel.anchorNode)
        ? sel.getRangeAt(0).cloneRange()
        : null;
    editingImgRef.current = null;
    setDrawInitial(null);
    setDrawOpen(true);
  };

  // Re-open an existing sketch to keep drawing on it.
  const openDrawEdit = (img) => {
    editingImgRef.current = img;
    drawRangeRef.current = null;
    setDrawInitial(img.src);
    setDrawOpen(true);
  };


  const handleDrawSave = (dataUrl) => {
    setDrawOpen(false);
    if (editingImgRef.current) {
      // Re-edit: just swap the existing sketch's pixels.
      editingImgRef.current.src = dataUrl;
      editingImgRef.current = null;
      scheduleSave();
      return;
    }
    contentRef.current.focus();
    const sel = window.getSelection();
    if (drawRangeRef.current) {
      sel.removeAllRanges();
      sel.addRange(drawRangeRef.current);
    }
    const img = document.createElement("img");
    img.src = dataUrl;
    img.className = "sketch";
    img.setAttribute("data-sketch", "1");
    img.title = "Click to edit the sketch";
    const range = sel.rangeCount ? sel.getRangeAt(0) : null;
    if (range) {
      range.collapse(false);
      range.insertNode(img);
      range.setStartAfter(img);
      range.collapse(true);
      sel.removeAllRanges();
      sel.addRange(range);
    } else {
      contentRef.current.appendChild(img);
    }
    scheduleSave();
  };

  const handlePaste = async (e) => {
    const dt = e.clipboardData;
    if (dt) {
      // Image exposed directly by the webview (some sources / Wayland builds).
      const file = dt.files && [...dt.files].find((f) => f.type.startsWith("image/"));
      if (file) { e.preventDefault(); embedImageFile(file); return; }
      for (let i = 0; i < (dt.items ? dt.items.length : 0); i++) {
        const it = dt.items[i];
        if (it.type && it.type.startsWith("image/")) {
          e.preventDefault();
          const f = it.getAsFile();
          if (f) embedImageFile(f);
          return;
        }
      }
      // Tabular clipboard (selected cells / spreadsheet). Inside a table we
      // distribute the values across cells preserving their positions; outside
      // we flatten to text so no nested table is dropped in.
      const html = dt.getData ? dt.getData("text/html") : "";
      const plain = dt.getData ? dt.getData("text/plain") || "" : "";
      const isTabular = (html && /<table[\s>]/i.test(html)) || plain.includes("\t");
      if (isTabular) {
        e.preventDefault();
        let grid = html && /<table[\s>]/i.test(html) ? parseGridFromHtml(html) : null;
        if (!grid) grid = parseGridFromText(plain);
        const cell = currentCell();
        if (cell && grid && (grid.length > 1 || (grid[0] && grid[0].length > 1))) {
          pasteGridIntoTable(grid, cell);
        } else {
          contentRef.current.focus();
          document.execCommand("insertText", false, grid ? grid.map((r) => r.join("\t")).join("\n") : plain);
          scheduleSave();
        }
        return;
      }
      // Real text on the clipboard → let the webview paste it (keeps formatting).
      const types = dt.types ? [...dt.types] : [];
      if (types.includes("text/plain") || types.includes("text/html")) return;
    }
    // WebKitGTK screenshot case: the image lives only on the system clipboard.
    e.preventDefault();
    const dataUrl = await readClipboardImageDataUrl();
    if (dataUrl) insertDataUrl(dataUrl, false);
  };

  // ── Drag-reorder to-dos ──
  const handleDragStart = (e) => {
    const handle = e.target.closest && e.target.closest(".todo-drag");
    if (!handle || !contentRef.current.contains(handle)) return;
    const todo = handle.closest(".todo");
    draggedTodoRef.current = todo;
    todo.classList.add("dragging");
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", ""); // Firefox needs data set
    }
  };

  const handleDragEnd = () => {
    if (draggedTodoRef.current) draggedTodoRef.current.classList.remove("dragging");
    draggedTodoRef.current = null;
  };

  const handleDrop = (e) => {
    // Reordering a to-do takes precedence over image drops.
    const dragged = draggedTodoRef.current;
    if (dragged) {
      e.preventDefault();
      const targetTodo = e.target.closest && e.target.closest(".todo");
      if (targetTodo && targetTodo !== dragged && contentRef.current.contains(targetTodo)) {
        const rect = targetTodo.getBoundingClientRect();
        const after = e.clientY > rect.top + rect.height / 2;
        targetTodo.parentNode.insertBefore(dragged, after ? targetTodo.nextSibling : targetTodo);
        scheduleSave();
      }
      handleDragEnd();
      return;
    }
    const files = e.dataTransfer && e.dataTransfer.files;
    if (!files || !files.length) return;
    const img = [...files].find((f) => f.type.startsWith("image/"));
    if (img) {
      e.preventDefault();
      embedImageFile(img);
    }
  };

  // ── In-note find (CSS Custom Highlight API) ──
  const clearFind = () => {
    findRanges.current = [];
    try {
      if (window.CSS && CSS.highlights) {
        CSS.highlights.delete("note-find");
        CSS.highlights.delete("note-find-current");
      }
    } catch {}
  };

  const paintFind = (index) => {
    if (!window.CSS || !CSS.highlights || typeof Highlight === "undefined") return;
    const ranges = findRanges.current;
    CSS.highlights.delete("note-find");
    CSS.highlights.delete("note-find-current");
    if (!ranges.length) return;
    const rest = ranges.filter((_, i) => i !== index);
    if (rest.length) CSS.highlights.set("note-find", new Highlight(...rest));
    if (ranges[index]) CSS.highlights.set("note-find-current", new Highlight(ranges[index]));
  };

  const runFind = (query) => {
    findRanges.current = [];
    if (query && contentRef.current) {
      const q = query.toLowerCase();
      const walker = document.createTreeWalker(contentRef.current, NodeFilter.SHOW_TEXT);
      let node;
      while ((node = walker.nextNode())) {
        const text = node.nodeValue.toLowerCase();
        let i = text.indexOf(q);
        while (i !== -1) {
          const r = document.createRange();
          r.setStart(node, i);
          r.setEnd(node, i + q.length);
          findRanges.current.push(r);
          i = text.indexOf(q, i + q.length);
        }
      }
    }
    const count = findRanges.current.length;
    setFind((f) => ({ ...f, query, count, index: 0 }));
    paintFind(0);
    if (count) scrollToMatch(0);
  };

  const scrollToMatch = (index) => {
    const r = findRanges.current[index];
    const el = r && (r.startContainer.nodeType === 3 ? r.startContainer.parentElement : r.startContainer);
    if (el && el.scrollIntoView) el.scrollIntoView({ block: "center", behavior: "smooth" });
  };

  const gotoMatch = (dir) => {
    const count = findRanges.current.length;
    if (!count) return;
    setFind((f) => {
      const index = (f.index + dir + count) % count;
      paintFind(index);
      scrollToMatch(index);
      return { ...f, index };
    });
  };

  const openFind = () => {
    setFind((f) => ({ ...f, open: true }));
    setTimeout(() => {
      const inp = document.getElementById("note-find-input");
      if (inp) inp.focus();
    }, 30);
  };

  const closeFind = () => {
    clearFind();
    setFind({ open: false, query: "", count: 0, index: 0 });
    contentRef.current && contentRef.current.focus();
  };

  // ── Markdown shortcuts ──
  const BLOCK_RULES = {
    "#": ["formatBlock", "h1"],
    "##": ["formatBlock", "h2"],
    "###": ["formatBlock", "h3"],
    "-": ["insertUnorderedList"],
    "*": ["insertUnorderedList"],
    "1.": ["insertOrderedList"],
    ">": ["formatBlock", "blockquote"],
    "[]": ["todo"],
    "[ ]": ["todo"],
  };

  const convertBlockToTodo = (block) => {
    const todo = createTodoEl("");
    if (block && block !== contentRef.current && block.parentNode) {
      block.parentNode.insertBefore(todo, block);
      block.remove();
    } else {
      const sel = window.getSelection();
      if (sel && sel.rangeCount) sel.getRangeAt(0).insertNode(todo);
      else contentRef.current.appendChild(todo);
    }
    placeCaretInBody(todo);
  };

  // Markdown prefix at the start of a line (triggered by Space). Returns true if applied.
  const applyMarkdownBlock = (e) => {
    const sel = window.getSelection();
    if (!sel || !sel.rangeCount || !sel.isCollapsed) return false;
    if (closestInContent(".todo") || closestInContent("pre")) return false;
    const range = sel.getRangeAt(0);
    let block = range.startContainer;
    if (block.nodeType === 3) block = block.parentElement;
    block = (block && block.closest && block.closest("p,div,h1,h2,h3,blockquote,li")) || contentRef.current;
    if (block.tagName === "LI") return false; // already in a list
    const pre = range.cloneRange();
    try { pre.setStart(block, 0); } catch { return false; }
    // `prefix === token` guarantees nothing precedes it, so deleting it is safe.
    const action = BLOCK_RULES[pre.toString()];
    if (!action) return false;
    e.preventDefault();
    pre.deleteContents(); // remove the markdown prefix ("#", ">", "-", …)
    if (action[0] === "todo") {
      convertBlockToTodo(block);
    } else {
      // deleteContents (on a cloned range) leaves the *live* selection pointing
      // at the removed nodes. Restore a valid caret at the line start before
      // formatting — otherwise setBlock/execCommand act on a stale selection
      // (this was why headings reverted while toolbar buttons worked).
      const caret = document.createRange();
      caret.setStart(block, 0);
      caret.collapse(true);
      sel.removeAllRanges();
      sel.addRange(caret);
      if (action[0] === "formatBlock") setBlock(action[1]); // h1/h2/h3/blockquote
      else document.execCommand(action[0]); // lists
    }
    refreshMarks();
    scheduleSave();
    return true;
  };

  // Inline markdown (**bold**, *italic*, _italic_, `code`) resolved on Space.
  const applyInlineMarkdown = () => {
    const sel = window.getSelection();
    if (!sel || !sel.isCollapsed) return;
    const node = sel.anchorNode;
    if (!node || node.nodeType !== 3) return;
    const before = node.textContent.slice(0, sel.anchorOffset);
    const rules = [
      { re: /\*\*([^*]+)\*\*$/, tag: "strong" },
      { re: /`([^`]+)`$/, tag: "code" },
      { re: /\*([^*]+)\*$/, tag: "em" },
      { re: /_([^_]+)_$/, tag: "em" },
    ];
    for (const { re, tag } of rules) {
      const m = before.match(re);
      if (!m) continue;
      const start = sel.anchorOffset - m[0].length;
      const r = document.createRange();
      r.setStart(node, start);
      r.setEnd(node, sel.anchorOffset);
      r.deleteContents();
      const el = document.createElement(tag);
      el.textContent = m[1];
      r.insertNode(el);
      const after = document.createRange();
      after.setStartAfter(el);
      after.collapse(true);
      sel.removeAllRanges();
      sel.addRange(after);
      scheduleSave();
      return;
    }
  };

  const handleContentKeyDown = (e) => {
    if (e.ctrlKey || e.metaKey) {
      const k = e.key.toLowerCase();
      if (k === "z" && !e.shiftKey) { e.preventDefault(); undo(); return; }
      if (k === "y" || (k === "z" && e.shiftKey)) { e.preventDefault(); redo(); return; }
      if (k === "b") { e.preventDefault(); fmt("bold"); }
      if (k === "i") { e.preventDefault(); fmt("italic"); }
      if (k === "u") { e.preventDefault(); fmt("underline"); }
      return;
    }
    // Tab moves between table cells (and adds a row past the last cell).
    if (e.key === "Tab") {
      const td = currentCell();
      if (td) {
        e.preventDefault();
        const cells = Array.from(td.closest("table").querySelectorAll("td"));
        const i = cells.indexOf(td);
        if (e.shiftKey) {
          if (i > 0) placeCaretStart(cells[i - 1]);
        } else if (i < cells.length - 1) {
          placeCaretStart(cells[i + 1]);
        } else {
          addRow(); // adds a row and focuses its first cell
        }
        return;
      }
    }
    // Backspace at the start of a table cell: never let WebKit merge across
    // cells (it deletes far too much). An all-empty table is removed instead.
    if (e.key === "Backspace") {
      const cell = currentCell();
      if (cell && caretAtStartOf(cell)) {
        e.preventDefault();
        const table = cell.closest("table");
        const allEmpty = Array.from(table.querySelectorAll("td")).every(
          (td) => td.textContent.replace(/\u200B/g, "").trim() === ""
        );
        if (allEmpty) deleteTable();
        return;
      }
    }
    // Backspace near a to-do's contenteditable=false parts (checkbox / drag
    // handle) makes WebKitGTK delete way too much. Handle these boundaries
    // ourselves instead of letting the browser do it.
    if (e.key === "Backspace") {
      const todo = closestInContent(".todo");
      if (todo) {
        const body = todo.querySelector(".todo-body");
        if (body && caretAtStartOf(body)) {
          // At the start of a to-do → turn it back into a normal line.
          e.preventDefault();
          const div = document.createElement("div");
          while (body.firstChild) div.appendChild(body.firstChild);
          if (!div.firstChild) div.appendChild(document.createElement("br"));
          todo.replaceWith(div);
          placeCaretStart(div);
          scheduleSave();
          refreshMarks();
          return;
        }
      } else {
        // At the start of a normal line right below a to-do → merge into it.
        const block = currentBlockEl();
        const prev = block && block.previousElementSibling;
        if (block && prev && prev.classList && prev.classList.contains("todo") && caretAtStartOf(block)) {
          e.preventDefault();
          const body = prev.querySelector(".todo-body");
          if (body) {
            const marker = block.firstChild;
            while (block.firstChild) body.appendChild(block.firstChild);
            block.remove();
            const r = document.createRange();
            if (marker && marker.parentNode === body) r.setStartBefore(marker);
            else { r.selectNodeContents(body); r.collapse(false); }
            r.collapse(true);
            const sel = window.getSelection();
            sel.removeAllRanges();
            sel.addRange(r);
            scheduleSave();
            refreshMarks();
          }
          return;
        }
      }
    }
    if (e.key === " ") {
      try {
        if (applyMarkdownBlock(e)) return; // block rule consumed the space
        applyInlineMarkdown(); // inline conversion; space still inserts normally
      } catch (err) {
        console.debug("markdown shortcut skipped:", err);
      }
      return;
    }
    // Enter at the end of a heading/quote drops to a normal line (exits the style).
    if (e.key === "Enter") {
      const hb = currentBlockEl();
      const sel = window.getSelection();
      if (
        hb &&
        /^(h1|h2|h3|blockquote)$/.test(hb.tagName.toLowerCase()) &&
        sel &&
        sel.isCollapsed &&
        sel.anchorOffset === (sel.anchorNode.nodeType === 3 ? sel.anchorNode.length : sel.anchorNode.childNodes.length) &&
        !sel.anchorNode.nextSibling
      ) {
        e.preventDefault();
        const div = document.createElement("div");
        div.appendChild(document.createElement("br"));
        hb.after(div);
        const r = document.createRange();
        r.setStart(div, 0);
        r.collapse(true);
        sel.removeAllRanges();
        sel.addRange(r);
        scheduleSave();
        refreshMarks();
        return;
      }
    }
    // Enter inside a to-do continues the checklist; Enter on an empty item exits it.
    if (e.key === "Enter") {
      const todo = closestInContent(".todo");
      if (todo) {
        e.preventDefault();
        const body = todo.querySelector(".todo-body");
        const text = body ? body.textContent.replace(/\u200B/g, "").trim() : "";
        if (text === "") {
          const p = document.createElement("p");
          p.innerHTML = "<br>";
          todo.parentNode.insertBefore(p, todo.nextSibling);
          todo.remove();
          const range = document.createRange();
          range.setStart(p, 0);
          range.collapse(true);
          const sel = window.getSelection();
          sel.removeAllRanges();
          sel.addRange(range);
        } else {
          const next = createTodoEl("");
          todo.parentNode.insertBefore(next, todo.nextSibling);
          placeCaretInBody(next);
        }
        scheduleSave();
      }
    }
  };

  const handleTitleKeyDown = (e) => {
    if (e.key === "Enter") {
      e.preventDefault();
      contentRef.current && contentRef.current.focus();
    }
  };

  if (!note) {
    return (
      <div className="editor-panel no-selection" style={{ display: "flex" }}>
        <p>{trashMode ? "Select a note from the trash" : "Select a note or create a new one"}</p>
        {!trashMode && <span className="shortcut-hint">Ctrl + N</span>}
      </div>
    );
  }

  if (trashMode) {
    return <TrashedNoteView note={note} onRestore={onRestore} onPurge={onPurge} />;
  }

  return (
    <div className="editor-panel">
      {/* ── Toolbar ── */}
      <div className="editor-toolbar">
        <button className={`toolbar-btn bold${marks.bold ? " active" : ""}`} onClick={() => fmt("bold")} title="Bold (Ctrl+B)">B</button>
        <button className={`toolbar-btn italic${marks.italic ? " active" : ""}`} onClick={() => fmt("italic")} title="Italic (Ctrl+I)">I</button>
        <button className={`toolbar-btn underline${marks.underline ? " active" : ""}`} onClick={() => fmt("underline")} title="Underline (Ctrl+U)">U</button>
        <div className="toolbar-sep" />
        <button className={`toolbar-btn${marks.block === "h1" ? " active" : ""}`} onClick={() => toggleBlock("h1")} title="Heading 1">H1</button>
        <button className={`toolbar-btn${marks.block === "h2" ? " active" : ""}`} onClick={() => toggleBlock("h2")} title="Heading 2">H2</button>
        <button className={`toolbar-btn${marks.block === "h3" ? " active" : ""}`} onClick={() => toggleBlock("h3")} title="Heading 3">H3</button>
        <div className="toolbar-sep" />
        <button className={`toolbar-btn${marks.ul ? " active" : ""}`} onClick={() => fmt("insertUnorderedList")} title="Bullet list">· List</button>
        <button className={`toolbar-btn${marks.ol ? " active" : ""}`} onClick={() => fmt("insertOrderedList")} title="Numbered list">1. List</button>
        <button className="toolbar-btn icon" onClick={insertTodo} title="To-do list">
          <IcCheckSquare />
        </button>
        <div className="toolbar-sep" />
        <button className="toolbar-btn icon" onClick={() => fileInputRef.current && fileInputRef.current.click()} title="Insert image">
          <IcImage />
        </button>
        <button className="toolbar-btn icon" onMouseDown={(e) => e.preventDefault()} onClick={openDraw} title="Draw">
          <IcPencil />
        </button>
        <button className={`toolbar-btn icon${marks.inTable ? " active" : ""}`} onMouseDown={(e) => e.preventDefault()} onClick={() => insertTable(2, 2)} title="Insert table">
          <IcTable />
        </button>
        <button className={`toolbar-btn${marks.block === "pre" ? " active" : ""}`} onClick={() => toggleBlock("pre")} title="Code block" style={{ fontFamily: "monospace", letterSpacing: "0.03em" }}>
          {"</>"}
        </button>

        {marks.inTable && (
          <>
            <div className="toolbar-sep" />
            <button className="toolbar-btn icon" onMouseDown={(e) => e.preventDefault()} onClick={() => setColumnAlign("left")} title="Align column left"><IcAlignLeft /></button>
            <button className="toolbar-btn icon" onMouseDown={(e) => e.preventDefault()} onClick={() => setColumnAlign("center")} title="Align column centre"><IcAlignCenter /></button>
            <button className="toolbar-btn icon" onMouseDown={(e) => e.preventDefault()} onClick={() => setColumnAlign("right")} title="Align column right"><IcAlignRight /></button>
            <div className="toolbar-sep" />
            <button className="toolbar-btn tbl-del" onMouseDown={(e) => e.preventDefault()} onClick={deleteTable} title="Delete the whole table">Delete table</button>
          </>
        )}

        <div style={{ marginLeft: "auto", display: "flex", gap: "2px", alignItems: "center" }}>
          <button
            className={`toolbar-btn icon${find.open ? " active" : ""}`}
            onClick={() => (find.open ? closeFind() : openFind())}
            title="Find in note"
          >
            <IcSearch />
          </button>
          <button
            className={`toolbar-btn fav${note.favorite ? " on" : ""}`}
            onClick={() => onToggleFavorite(note.id)}
            title={note.favorite ? "Remove from favourites" : "Add to favourites"}
          >
            {note.favorite ? "★" : "☆"}
          </button>
          <button className="toolbar-btn del" onClick={() => onDelete(note.id)} title="Delete note">
            <IcTrash />
          </button>
        </div>
      </div>

      {find.open && (
        <div className="find-bar">
          <IcSearch />
          <input
            id="note-find-input"
            className="find-input"
            type="text"
            placeholder="Find in note…"
            value={find.query}
            onChange={(e) => runFind(e.target.value)}
            onKeyDown={(e) => {
              if (e.key === "Enter") { e.preventDefault(); gotoMatch(e.shiftKey ? -1 : 1); }
              if (e.key === "Escape") { e.preventDefault(); closeFind(); }
            }}
          />
          <span className="find-count">{find.count ? `${find.index + 1}/${find.count}` : "0/0"}</span>
          <button className="find-nav" onClick={() => gotoMatch(-1)} title="Previous" disabled={!find.count}>↑</button>
          <button className="find-nav" onClick={() => gotoMatch(1)} title="Next" disabled={!find.count}>↓</button>
          <button className="find-nav" onClick={closeFind} title="Close">✕</button>
        </div>
      )}

      {/* ── Body ── */}
      <div className="editor-body">
        <div className="editor-meta-row">
          <TagDropdown note={note} folders={folders} onChangeFolder={onChangeFolder} onNewTag={onNewTag} />
          <span className="editor-date-text">{formatDate(note.updated)}</span>
          <span className="autosave-hint">Saved automatically</span>
        </div>

        <div
          ref={titleRef}
          className="editor-title"
          contentEditable
          suppressContentEditableWarning
          onInput={scheduleSave}
          onKeyDown={handleTitleKeyDown}
          data-placeholder="Untitled"
        />

        <div
          ref={contentRef}
          className="editor-content"
          contentEditable
          suppressContentEditableWarning
          onInput={handleContentInput}
          onKeyDown={handleContentKeyDown}
          onKeyUp={refreshMarks}
          onMouseUp={refreshMarks}
          onFocus={refreshMarks}
          onBlur={() => { flushSave(); setMarks({}); setTableBox(null); }}
          onClick={handleContentClick}
          onPaste={handlePaste}
          onDrop={handleDrop}
          onDragStart={handleDragStart}
          onDragEnd={handleDragEnd}
          onDragOver={(e) => e.preventDefault()}
          data-placeholder="Start writing…"
        />

        {/* In-table "+" handles (Obsidian-style), anchored to the active table */}
        {tableBox && (
          <>
            <button
              className="tbl-add tbl-add-col"
              style={{ top: tableBox.top, left: tableBox.left + tableBox.w + 5, height: tableBox.h }}
              onMouseDown={(e) => e.preventDefault()}
              onClick={addColumnEnd}
              title="Add column"
            >
              +
            </button>
            <button
              className="tbl-add tbl-add-row"
              style={{ top: tableBox.top + tableBox.h + 5, left: tableBox.left, width: tableBox.w }}
              onMouseDown={(e) => e.preventDefault()}
              onClick={addRowEnd}
              title="Add row"
            >
              +
            </button>

            {/* "−" delete a column (above each column) */}
            {tableBox.cols.map((c, i) => (
              <button
                key={"col" + i}
                className="tbl-handle tbl-handle-col"
                style={{ left: c.left, top: tableBox.top - 17, width: c.width }}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => deleteColumnAt(i)}
                title="Delete column"
              >
                −
              </button>
            ))}
            {/* "−" delete a row (left of each row) */}
            {tableBox.rows.map((r, i) => (
              <button
                key={"row" + i}
                className="tbl-handle tbl-handle-row"
                style={{ top: r.top, left: tableBox.left - 17, height: r.height }}
                onMouseDown={(e) => e.preventDefault()}
                onClick={() => deleteRowAt(i)}
                title="Delete row"
              >
                −
              </button>
            ))}
          </>
        )}
      </div>

      <input
        ref={fileInputRef}
        type="file"
        accept="image/*"
        style={{ display: "none" }}
        onChange={handleFileChange}
      />

      <DrawModal open={drawOpen} onClose={() => setDrawOpen(false)} onSave={handleDrawSave} initialImage={drawInitial} />
    </div>
  );
}

// ── NEW TAG MODAL ──────────────────────────────────────

export function NewTagModal({ open, onClose, onCreate, error }) {
  const [name, setName] = React.useState("");
  const [color, setColor] = React.useState(TAG_PALETTE[0]);
  const inputRef = React.useRef(null);

  React.useEffect(() => {
    if (open) {
      setName("");
      setColor(TAG_PALETTE[0]);
      setTimeout(() => inputRef.current && inputRef.current.focus(), 50);
    }
  }, [open]);

  const submit = () => {
    if (name.trim()) onCreate(name.trim(), color);
  };

  return (
    <Modal
      open={open}
      title="New tag"
      onClose={onClose}
      width={360}
      footer={
        <>
          <button className="btn-ghost" onClick={onClose}>Cancel</button>
          <button className="btn-primary" onClick={submit} disabled={!name.trim()}>Create</button>
        </>
      }
    >
      <label className="field-label" htmlFor="tag-name">Name</label>
      <input
        id="tag-name"
        ref={inputRef}
        className="text-input"
        type="text"
        maxLength={24}
        placeholder="e.g. Recipes"
        value={name}
        onChange={(e) => setName(e.target.value)}
        onKeyDown={(e) => e.key === "Enter" && submit()}
      />

      <label className="field-label" style={{ marginTop: 14 }}>Colour</label>
      <div className="color-grid">
        {TAG_PALETTE.map((c) => (
          <button
            key={c}
            type="button"
            className={`color-swatch${c === color ? " selected" : ""}`}
            style={{ background: c }}
            aria-label={c}
            onClick={() => setColor(c)}
          >
            {c === color && <IcCheck />}
          </button>
        ))}
      </div>

      {error && <div className="form-error">{error}</div>}
    </Modal>
  );
}
