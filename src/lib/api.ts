// Thin typed facade over the generated tauri-specta bindings.
// The UI imports only from here, never from bindings.ts directly,
// so the transport can be swapped without touching components.
import { commands } from "./bindings";

export async function isUnlocked(): Promise<boolean> {
  return commands.isUnlocked();
}
