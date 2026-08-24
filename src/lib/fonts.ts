// Monospace font registry. Every stack ends in generic monospace, so a font
// that isn't installed on this Mac falls back gracefully.
export interface MonoFont {
  id: string;
  label: string;
  stack: string;
}

export const fonts: MonoFont[] = [
  { id: "sf-mono", label: "sf mono", stack: `"SF Mono", "Menlo", monospace` },
  { id: "menlo", label: "menlo", stack: `"Menlo", monospace` },
  { id: "monaco", label: "monaco", stack: `"Monaco", monospace` },
  { id: "jetbrains-mono", label: "jetbrains mono", stack: `"JetBrains Mono", "SF Mono", "Menlo", monospace` },
  { id: "fira-code", label: "fira code", stack: `"Fira Code", "SF Mono", "Menlo", monospace` },
  { id: "ibm-plex-mono", label: "ibm plex mono", stack: `"IBM Plex Mono", "SF Mono", "Menlo", monospace` },
  { id: "courier", label: "courier", stack: `"Courier New", monospace` },
];

export const defaultFont = "sf-mono";
export const MIN_SIZE = 12;
export const MAX_SIZE = 22;
export const defaultFontSize = 16;

export function applyFont(id: string) {
  const font = fonts.find((f) => f.id === id) ?? fonts[0];
  document.documentElement.style.fontFamily = font.stack;
}

/** Root font size in px — the rem base, so the whole UI scales with it. */
export function applyFontSize(px: number) {
  const clamped = Math.max(MIN_SIZE, Math.min(MAX_SIZE, px));
  document.documentElement.style.fontSize = `${clamped}px`;
}
