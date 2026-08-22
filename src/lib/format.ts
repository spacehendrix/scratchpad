import type { DocMeta } from "./bindings";

/** Display name: title, else first-line preview, else "untitled". */
export function displayName(meta: Pick<DocMeta, "title" | "preview">): string {
  return meta.title || meta.preview || "untitled";
}

/** Compact byte size: "0 B", "890 kB", "1.4 MB", "2.1 GB". */
export function fmtBytes(bytes: number): string {
  if (bytes < 1000) return `${bytes} B`;
  const units = ["kB", "MB", "GB"];
  let value = bytes;
  let unit = "";
  for (const u of units) {
    value /= 1000;
    unit = u;
    if (value < 1000) break;
  }
  return `${value < 10 ? value.toFixed(1) : Math.round(value)} ${unit}`;
}

/** Compact relative age: "now", "5m", "3h", "2d", "5w", "4mo". */
export function relativeAge(thenMs: number, nowMs: number): string {
  const s = Math.max(0, Math.floor((nowMs - thenMs) / 1000));
  if (s < 60) return "now";
  const m = Math.floor(s / 60);
  if (m < 60) return `${m}m`;
  const h = Math.floor(m / 60);
  if (h < 24) return `${h}h`;
  const d = Math.floor(h / 24);
  if (d < 7) return `${d}d`;
  const w = Math.floor(d / 7);
  if (w < 9) return `${w}w`;
  const mo = Math.floor(d / 30);
  return `${mo}mo`;
}
