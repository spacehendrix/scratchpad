// Theme/settings state. Loaded before unlock so the unlock screen is themed.
import { getSettings, setSettings } from "../api";
import { applyTheme, defaultTheme } from "../themes";

class SettingsStore {
  theme = $state(defaultTheme);

  async load() {
    try {
      const s = await getSettings();
      this.theme = s.theme;
    } catch {
      this.theme = defaultTheme;
    }
    applyTheme(this.theme);
  }

  /** Apply without persisting (live preview in the picker). */
  preview(id: string) {
    applyTheme(id);
  }

  /** Apply and persist. */
  async commit(id: string) {
    this.theme = id;
    applyTheme(id);
    try {
      await setSettings({ theme: id });
    } catch {
      // Persisting is best-effort; the theme still applies for this run.
    }
  }
}

export const settings = new SettingsStore();
