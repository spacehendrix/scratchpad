// Theme/font settings state. Loaded before unlock so the unlock screen is
// already themed and sized.
import { getSettings, setSettings, type Settings } from "../api";
import { applyTheme, defaultTheme } from "../themes";
import { applyFont, applyFontSize, defaultFont, defaultFontSize } from "../fonts";

class SettingsStore {
  theme = $state(defaultTheme);
  font = $state(defaultFont);
  fontSize = $state(defaultFontSize);

  async load() {
    try {
      const s = await getSettings();
      this.theme = s.theme;
      this.font = s.font ?? defaultFont;
      this.fontSize = s.fontSize ?? defaultFontSize;
    } catch {
      // Defaults stand.
    }
    this.applyAll();
  }

  applyAll() {
    applyTheme(this.theme);
    applyFont(this.font);
    applyFontSize(this.fontSize);
  }

  /** Apply and persist. */
  async commit(next: Required<Settings>) {
    this.theme = next.theme;
    this.font = next.font;
    this.fontSize = next.fontSize;
    this.applyAll();
    try {
      await setSettings(next);
    } catch {
      // Persisting is best-effort; the settings still apply for this run.
    }
  }
}

export const settings = new SettingsStore();
