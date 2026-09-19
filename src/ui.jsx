// ui.jsx — small theme-aware primitives shared across dialogs.
import React from "react";

// Centered modal with a backdrop. Closes on Esc or backdrop click.
export function Modal({ open, title, onClose, children, width = 380, footer }) {
  React.useEffect(() => {
    if (!open) return;
    const onKey = (e) => e.key === "Escape" && onClose();
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [open, onClose]);

  if (!open) return null;
  return (
    <div className="modal-backdrop" onMouseDown={onClose}>
      <div
        className="modal-card"
        style={{ width }}
        role="dialog"
        aria-modal="true"
        aria-label={title}
        onMouseDown={(e) => e.stopPropagation()}
      >
        <div className="modal-head">
          <span className="modal-title">{title}</span>
          <button className="modal-x" aria-label="Close" onClick={onClose}>
            ✕
          </button>
        </div>
        <div className="modal-body">{children}</div>
        {footer && <div className="modal-foot">{footer}</div>}
      </div>
    </div>
  );
}

// Segmented (radio) control with a sliding thumb and drag support.
export function Segmented({ value, options, onChange }) {
  const trackRef = React.useRef(null);
  const valueRef = React.useRef(value);
  valueRef.current = value;

  const opts = options.map((o) => (typeof o === "object" ? o : { value: o, label: o }));
  const idx = Math.max(0, opts.findIndex((o) => o.value === value));
  const n = opts.length;

  const segAt = (clientX) => {
    const r = trackRef.current.getBoundingClientRect();
    const inner = r.width - 4;
    const i = Math.floor(((clientX - r.left - 2) / inner) * n);
    return opts[Math.max(0, Math.min(n - 1, i))].value;
  };

  const onPointerDown = (e) => {
    const v0 = segAt(e.clientX);
    if (v0 !== valueRef.current) onChange(v0);
    const move = (ev) => {
      if (!trackRef.current) return;
      const v = segAt(ev.clientX);
      if (v !== valueRef.current) onChange(v);
    };
    const up = () => {
      window.removeEventListener("pointermove", move);
      window.removeEventListener("pointerup", up);
    };
    window.addEventListener("pointermove", move);
    window.addEventListener("pointerup", up);
  };

  return (
    <div ref={trackRef} className="seg" role="radiogroup" onPointerDown={onPointerDown}>
      <div
        className="seg-thumb"
        style={{ left: `calc(2px + ${idx} * (100% - 4px) / ${n})`, width: `calc((100% - 4px) / ${n})` }}
      />
      {opts.map((o) => (
        <button key={o.value} type="button" role="radio" aria-checked={o.value === value}>
          {o.label}
        </button>
      ))}
    </div>
  );
}

export function Slider({ value, min = 0, max = 100, step = 1, onChange }) {
  return (
    <input
      type="range"
      className="ui-slider"
      min={min}
      max={max}
      step={step}
      value={value}
      onChange={(e) => onChange(Number(e.target.value))}
    />
  );
}

// Confirmation dialog with an optional destructive accent.
export function ConfirmModal({ open, title, message, confirmLabel = "OK", danger, onConfirm, onClose }) {
  return (
    <Modal
      open={open}
      title={title}
      onClose={onClose}
      width={360}
      footer={
        <>
          <button className="btn-ghost" onClick={onClose}>Cancel</button>
          <button className={danger ? "btn-danger" : "btn-primary"} onClick={onConfirm}>
            {confirmLabel}
          </button>
        </>
      }
    >
      <p className="confirm-text">{message}</p>
    </Modal>
  );
}

// A labelled settings row.
export function Field({ label, value, children }) {
  return (
    <div className="field">
      <div className="field-head">
        <span className="field-label">{label}</span>
        {value != null && <span className="field-value">{value}</span>}
      </div>
      {children}
    </div>
  );
}
