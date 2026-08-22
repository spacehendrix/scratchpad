import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import { defaultTheme, themes } from "./themes";

const THEMES_DIR = join(fileURLToPath(new URL(".", import.meta.url)), "..", "themes");
const TOKENS = [
  "--bg",
  "--bg-alt",
  "--fg",
  "--fg-dim",
  "--accent",
  "--accent2",
  "--ok",
  "--warn",
  "--err",
  "--border",
  "--sel-bg",
  "--cursor",
];

describe("theme registry", () => {
  it("has unique ids and includes the default", () => {
    const ids = themes.map((t) => t.id);
    expect(new Set(ids).size).toBe(ids.length);
    expect(ids).toContain(defaultTheme);
  });

  it("matches the css files on disk exactly", () => {
    const files = readdirSync(THEMES_DIR)
      .filter((f) => f.endsWith(".css"))
      .map((f) => f.replace(/\.css$/, ""))
      .sort();
    const ids = themes.map((t) => t.id).sort();
    expect(ids).toEqual(files);
  });

  it("every theme defines the full token set under its own selector", () => {
    for (const theme of themes) {
      const css = readFileSync(join(THEMES_DIR, `${theme.id}.css`), "utf8");
      expect(css).toContain(`:root[data-theme="${theme.id}"]`);
      for (const token of TOKENS) {
        expect(css, `${theme.id} is missing ${token}`).toContain(`${token}:`);
      }
    }
  });
});
