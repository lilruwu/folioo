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

export function SettingsModal({ open, onClose, settings, onChange, onExport, onImport, importMsg }) {
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
    </Modal>
  );
}
