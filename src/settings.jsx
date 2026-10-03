// settings.jsx — theme preferences, persisted to localStorage and presented
// in a proper Settings modal (no more floating panel).
import React from "react";
import { Modal, Segmented, Slider, Field } from "./ui.jsx";

const LS_KEY = "folioo-settings";
// Key used before the app was renamed; read once so upgraders keep their theme.
const LEGACY_LS_KEY = "linux-notes-settings";

// useSettings — single source of truth for appearance prefs.
export function useSettings(defaults) {
  const [values, setValues] = React.useState(() => {
    try {
      const raw = localStorage.getItem(LS_KEY) ?? localStorage.getItem(LEGACY_LS_KEY);
      return raw ? { ...defaults, ...JSON.parse(raw) } : defaults;
    } catch {
      return defaults;
    }
  });
  const set = React.useCallback((key, val) => {
    setValues((prev) => {
      const next = { ...prev, [key]: val };
      try {
        localStorage.setItem(LS_KEY, JSON.stringify(next));
      } catch {
        /* ignore quota errors */
      }
      return next;
    });
  }, []);
  return [values, set];
}

// "just now", "5 min ago", "3 h ago", or a date.
function ago(ms, now) {
  const seconds = Math.round((now - ms) / 1000);
  if (seconds < 45) return "just now";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `${minutes} min ago`;
  const hours = Math.round(minutes / 60);
  if (hours < 24) return `${hours} h ago`;
  return new Date(ms).toLocaleDateString("en-GB", { day: "numeric", month: "short" });
}

// One line describing the sync state, and its tone for the status dot.
function describeSync(status, now) {
  const last = status.lastSyncMs ? `last synced ${ago(status.lastSyncMs, now)}` : "";
  if (status.running) return { tone: "busy", text: "Syncing…" };
  if (!status.available) {
    return { tone: "problem", text: ["Folder not available", last].filter(Boolean).join(" · ") };
  }
  if (status.error) {
    return { tone: "problem", text: [`Couldn't sync: ${status.error}`, last].filter(Boolean).join(" · ") };
  }
  if (status.lastSyncMs) return { tone: "ok", text: `Synced ${ago(status.lastSyncMs, now)}` };
  return { tone: "busy", text: "Not synced yet" };
}

function SyncSection({ status, message, onChooseFolder, onSyncNow, onStop }) {
  // Re-render every 30 s so "5 min ago" stays true while Settings is open.
  const [now, setNow] = React.useState(Date.now());
  React.useEffect(() => {
    const t = setInterval(() => setNow(Date.now()), 30_000);
    return () => clearInterval(t);
  }, []);

  const folder = status?.folder;
  const state = folder ? describeSync(status, now) : null;

  return (
    <>
      <div className="settings-section">Sync</div>
      <p className="settings-hint">
        Folioo keeps a copy of your notes in a folder you choose. Sync that folder between machines
        with a tool such as rclone, Syncthing, Nextcloud or Dropbox.
      </p>
      {folder ? (
        <>
          <div className="sync-folder" title={folder}>{folder}</div>
          <div className={`sync-state sync-${state.tone}`}>
            <span className="sync-dot" aria-hidden="true" />
            <span>{state.text}</span>
          </div>
          <div className="settings-actions">
            <button className="btn-ghost" onClick={onSyncNow} disabled={status.running}>
              Sync now
            </button>
            <button className="btn-ghost" onClick={onChooseFolder}>Change folder…</button>
            <button className="btn-ghost btn-ghost-danger" onClick={onStop}>Stop syncing</button>
          </div>
        </>
      ) : (
        <div className="settings-actions">
          <button className="btn-ghost" onClick={onChooseFolder}>Choose folder…</button>
        </div>
      )}
      {message && <div className="sync-message">{message}</div>}
    </>
  );
}

export function SettingsModal({
  open,
  onClose,
  settings,
  onChange,
  onExport,
  onImport,
  importMsg,
  sync,
}) {
  return (
    <Modal open={open} title="Settings" onClose={onClose} width={400}>
      <div className="settings-section">Appearance</div>

      <Field label="Style">
        <Segmented
          value={settings.variant}
          options={[
            { value: "paper", label: "Paper" },
            { value: "slate", label: "Slate" },
            { value: "forest", label: "Forest" },
          ]}
          onChange={(v) => onChange("variant", v)}
        />
      </Field>

      <Field label="Mode">
        <Segmented
          value={settings.theme}
          options={[
            { value: "light", label: "Light" },
            { value: "dark", label: "Dark" },
            { value: "auto", label: "Auto" },
          ]}
          onChange={(v) => onChange("theme", v)}
        />
      </Field>

      <Field label="Translucent background">
        <Segmented
          value={settings.translucid}
          options={[
            { value: false, label: "Off" },
            { value: true, label: "On" },
          ]}
          onChange={(v) => onChange("translucid", v)}
        />
        <p className="settings-hint">Recommended for KDE Plasma with a compositor.</p>
      </Field>

      <div className="settings-section">Typography</div>

      <Field label="Text size" value={`${settings.fontSize}px`}>
        <Slider
          value={settings.fontSize}
          min={13}
          max={19}
          step={1}
          onChange={(v) => onChange("fontSize", v)}
        />
      </Field>

      <div className="settings-section">Data</div>
      <p className="settings-hint">
        Export a backup of all your notes and tags, or import one on another machine.
      </p>
      <div className="settings-actions">
        <button className="btn-ghost" onClick={onExport}>Export backup…</button>
        <button className="btn-ghost" onClick={onImport}>Import backup…</button>
      </div>
      {importMsg && <div className="settings-note">{importMsg}</div>}

      <SyncSection {...sync} />
    </Modal>
  );
}
