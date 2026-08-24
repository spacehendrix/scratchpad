// Dashboard configuration registry, shared by the dashboard itself and the
// settings screen.

export const PANELS = [
  { id: "activity", label: "activity" },
  { id: "tasks", label: "tasks" },
  { id: "storage", label: "storage" },
] as const;

export interface DashStyle {
  id: string;
  label: string;
  /** Sparkline levels, low → high (8 steps). */
  levels: string[];
  /** Gauge glyphs: [filled, empty]. */
  gauge: [string, string];
}

export const STYLES: DashStyle[] = [
  {
    id: "blocks",
    label: "blocks",
    levels: ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"],
    gauge: ["▓", "░"],
  },
  {
    id: "braille",
    label: "braille",
    levels: ["⢀", "⣀", "⣄", "⣤", "⣦", "⣶", "⣷", "⣿"],
    gauge: ["⣿", "⣀"],
  },
  {
    id: "shade",
    label: "shade",
    levels: ["░", "░", "▒", "▒", "▓", "▓", "█", "█"],
    gauge: ["█", "░"],
  },
];

export interface DashSize {
  id: string;
  label: string;
  /** Sparkline window in days (one char per day). */
  days: number;
  /** Gauge width in cells. */
  bar: number;
}

export const SIZES: DashSize[] = [
  { id: "small", label: "small", days: 7, bar: 8 },
  { id: "medium", label: "medium", days: 14, bar: 12 },
  { id: "large", label: "large", days: 21, bar: 18 },
];

export const defaultPanels: string[] = PANELS.map((p) => p.id);
export const defaultStyle = "blocks";
export const defaultSize = "medium";

export const styleById = (id: string): DashStyle =>
  STYLES.find((s) => s.id === id) ?? STYLES[0];
export const sizeById = (id: string): DashSize => SIZES.find((s) => s.id === id) ?? SIZES[1];
