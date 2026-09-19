// draw.jsx — a sketch canvas modal. The result is handed back as a PNG data URL
// and embedded into the note as an image (Apple-Notes-style inline sketch).
import React from "react";

const BASE_COLORS = ["#1C1C1C", "#E8483F", "#2F6DF0", "#36A04F", "#E8902E", "#9B5DE5"];
const SIZES = [3, 6, 12];

export function DrawModal({ open, onClose, onSave, initialImage }) {
  const canvasRef = React.useRef(null);
  const wrapRef = React.useRef(null);
  const drawing = React.useRef(false);
  const undoStack = React.useRef([]);

  const [palette, setPalette] = React.useState(BASE_COLORS);
  const [color, setColor] = React.useState("#1C1C1C");
  const [size, setSize] = React.useState(6);
  const [eraser, setEraser] = React.useState(false);

  const ctx = () => canvasRef.current && canvasRef.current.getContext("2d");

  // Initialise the canvas once the modal is laid out.
  React.useEffect(() => {
    if (!open) return;
    const id = requestAnimationFrame(() => {
      const canvas = canvasRef.current;
      const wrap = wrapRef.current;
      if (!canvas || !wrap) return;
      const dpr = window.devicePixelRatio || 1;
      const w = wrap.clientWidth;
      const h = wrap.clientHeight;
      canvas.width = Math.round(w * dpr);
      canvas.height = Math.round(h * dpr);
      canvas.style.width = w + "px";
      canvas.style.height = h + "px";
      const c = canvas.getContext("2d");
      c.setTransform(dpr, 0, 0, dpr, 0, 0);
      c.fillStyle = "#FFFFFF";
      c.fillRect(0, 0, w, h);
      c.lineCap = "round";
      c.lineJoin = "round";
      undoStack.current = [];
      // When re-editing, paint the existing sketch onto the canvas (contained).
      if (initialImage) {
        const im = new Image();
        im.onload = () => {
          const scale = Math.min(w / im.width, h / im.height, 1);
          const dw = im.width * scale;
          const dh = im.height * scale;
          c.drawImage(im, (w - dw) / 2, (h - dh) / 2, dw, dh);
        };
        im.src = initialImage;
      }
      const accent = getComputedStyle(document.documentElement).getPropertyValue("--accent").trim();
      const pal = accent ? [accent, ...BASE_COLORS.filter((x) => x.toLowerCase() !== accent.toLowerCase())] : BASE_COLORS;
      setPalette(pal.slice(0, 7));
      setColor("#1C1C1C");
      setSize(6);
      setEraser(false);
    });
    return () => cancelAnimationFrame(id);
  }, [open]);

  const posOf = (e) => {
    const r = canvasRef.current.getBoundingClientRect();
    return { x: e.clientX - r.left, y: e.clientY - r.top };
  };

  const pushUndo = () => {
    const canvas = canvasRef.current;
    const c = ctx();
    if (!canvas || !c) return;
    try {
      undoStack.current.push(c.getImageData(0, 0, canvas.width, canvas.height));
      if (undoStack.current.length > 25) undoStack.current.shift();
    } catch {
      /* ignore */
    }
  };

  const start = (e) => {
    e.preventDefault();
    pushUndo();
    drawing.current = true;
    const c = ctx();
    const p = posOf(e);
    c.strokeStyle = eraser ? "#FFFFFF" : color;
    c.lineWidth = eraser ? size * 3.2 : size;
    c.beginPath();
    c.moveTo(p.x, p.y);
    c.lineTo(p.x + 0.01, p.y + 0.01); // render a dot on a single tap
    c.stroke();
    try { canvasRef.current.setPointerCapture(e.pointerId); } catch {}
  };

  const move = (e) => {
    if (!drawing.current) return;
    const c = ctx();
    const p = posOf(e);
    c.lineTo(p.x, p.y);
    c.stroke();
  };

  const end = () => { drawing.current = false; };

  const undo = () => {
    const c = ctx();
    const img = undoStack.current.pop();
    if (c && img) c.putImageData(img, 0, 0);
  };

  const clearAll = () => {
    const canvas = canvasRef.current;
    const c = ctx();
    if (!canvas || !c) return;
    pushUndo();
    c.save();
    c.setTransform(1, 0, 0, 1, 0, 0);
    c.fillStyle = "#FFFFFF";
    c.fillRect(0, 0, canvas.width, canvas.height);
    c.restore();
  };

  const save = () => {
    const url = canvasRef.current.toDataURL("image/png");
    onSave(url);
  };

  if (!open) return null;
  return (
    <div
      className="draw-overlay"
      onMouseDown={(e) => { if (e.target.classList.contains("draw-overlay")) onClose(); }}
    >
      <div className="draw-modal">
        <div className="draw-toolbar">
          <div className="draw-colors">
            {palette.map((c) => (
              <button
                key={c}
                className={`draw-swatch${!eraser && color === c ? " sel" : ""}`}
                style={{ background: c }}
                onClick={() => { setColor(c); setEraser(false); }}
                aria-label={c}
              />
            ))}
          </div>
          <div className="draw-sep" />
          {SIZES.map((s) => (
            <button
              key={s}
              className={`draw-size${!eraser && size === s ? " sel" : ""}`}
              onClick={() => { setSize(s); setEraser(false); }}
              title={`Size ${s}`}
            >
              <span style={{ width: s + 2, height: s + 2 }} />
            </button>
          ))}
          <div className="draw-sep" />
          <button className={`draw-tool${eraser ? " sel" : ""}`} onClick={() => setEraser(true)} title="Eraser">Eraser</button>
          <button className="draw-tool" onClick={undo} title="Undo">↶</button>
          <button className="draw-tool" onClick={clearAll} title="Clear all">Clear</button>
          <div className="draw-actions">
            <button className="btn-ghost" onClick={onClose}>Cancel</button>
            <button className="btn-primary" onClick={save}>Insert</button>
          </div>
        </div>
        <div className="draw-canvas-wrap" ref={wrapRef}>
          <canvas
            ref={canvasRef}
            className="draw-canvas"
            onPointerDown={start}
            onPointerMove={move}
            onPointerUp={end}
            onPointerLeave={end}
          />
        </div>
      </div>
    </div>
  );
}
