// UI session state (Svelte 5 runes). Purely presentational — the source of
// truth for locked/unlocked lives in the Rust core.
export type View = "browse" | "editor" | "search" | "settings";

class SessionStore {
  locked = $state(true);
  view = $state<View>("browse");
}

export const session = new SessionStore();
