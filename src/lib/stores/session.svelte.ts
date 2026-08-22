// UI session state (Svelte 5 runes). Purely presentational — the source of
// truth for locked/unlocked lives in the Rust core.
import { lock } from "../api";

export type View = "browse" | "editor" | "search" | "settings";

class SessionStore {
  locked = $state(true);
  view = $state<View>("browse");
  /** Registered by the editor so pending edits flush before locking. */
  beforeLock: (() => Promise<unknown>) | null = null;

  /** Cmd+L: flush any pending save, drop the key, show the unlock screen. */
  async lockNow() {
    try {
      await this.beforeLock?.();
    } catch {
      // Never let a failed flush keep the app unlocked.
    }
    await lock();
    this.view = "browse";
    this.locked = true;
  }
}

export const session = new SessionStore();
