// data.js — presentational helpers. Tags/folders now live in SQLite and are
// passed in as data; this module only holds UI-only utilities.

// Palette offered when creating a new tag.
export const TAG_PALETTE = [
  "#4B85E8",
  "#52B46B",
  "#E8A23A",
  "#E85252",
  "#9B6BE8",
  "#E86BB0",
  "#2BB0A6",
  "#8A7A66",
];

export function formatDate(dateStr) {
  if (!dateStr) return "";
  const d = new Date(dateStr + "T12:00:00");
  const now = new Date();
  const diff = Math.floor((now - d) / 86400000);
  if (diff === 0) return "Today";
  if (diff === 1) return "Yesterday";
  return d.toLocaleDateString("en-GB", { day: "numeric", month: "short" });
}

// Resolve a tag's colour from the loaded folder list.
export function colorOf(folders, name) {
  return folders.find((f) => f.name === name)?.color || "#999";
}

// Days remaining before a trashed note is purged (mirrors the backend window).
export const TRASH_RETENTION_DAYS = 30;

export function trashDaysLeft(deletedAt) {
  if (!deletedAt) return TRASH_RETENTION_DAYS;
  const deleted = new Date(deletedAt + "T12:00:00");
  const elapsed = Math.floor((Date.now() - deleted) / 86400000);
  return Math.max(0, TRASH_RETENTION_DAYS - elapsed);
}
