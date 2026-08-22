// Theme registry. Each id has a matching src/themes/<id>.css defining the
// full token set under :root[data-theme="<id>"] (all loaded eagerly via
// import.meta.glob in main.ts — adding a css file + a row here is all it
// takes; a vitest guard keeps the two in sync).
export interface Theme {
  id: string;
  label: string;
}

export const themes: Theme[] = [
  // true black
  { id: "void", label: "void" },
  { id: "oled-neon", label: "oled neon" },
  { id: "amber-term", label: "amber term" },
  { id: "green-term", label: "green term" },
  { id: "matte-black", label: "matte black" },
  // dark
  { id: "tokyo-night", label: "tokyo night" },
  { id: "catppuccin", label: "catppuccin" },
  { id: "gruvbox", label: "gruvbox" },
  { id: "nord", label: "nord" },
  { id: "everforest", label: "everforest" },
  { id: "rose-pine", label: "rosé pine" },
  { id: "dracula", label: "dracula" },
  { id: "one-dark", label: "one dark" },
  { id: "monokai", label: "monokai" },
  { id: "kanagawa", label: "kanagawa" },
  { id: "ayu-dark", label: "ayu dark" },
  { id: "night-owl", label: "night owl" },
  { id: "github-dark", label: "github dark" },
  { id: "solarized-dark", label: "solarized dark" },
  { id: "synthwave", label: "synthwave" },
  // light
  { id: "solarized-light", label: "solarized light" },
];

export const defaultTheme = "tokyo-night";

export function applyTheme(id: string) {
  document.documentElement.dataset.theme = id;
}
