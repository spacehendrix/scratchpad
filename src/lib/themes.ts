// Theme registry. Each id has a matching src/themes/<id>.css defining the
// full token set under :root[data-theme="<id>"].
export interface Theme {
  id: string;
  label: string;
}

export const themes: Theme[] = [
  { id: "tokyo-night", label: "tokyo night" },
  { id: "catppuccin", label: "catppuccin" },
  { id: "gruvbox", label: "gruvbox" },
  { id: "nord", label: "nord" },
  { id: "everforest", label: "everforest" },
  { id: "rose-pine", label: "rosé pine" },
  { id: "matte-black", label: "matte black" },
];

export const defaultTheme = "tokyo-night";

export function applyTheme(id: string) {
  document.documentElement.dataset.theme = id;
}
