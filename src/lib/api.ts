// Thin typed facade over the generated tauri-specta bindings.
// The UI imports only from here, never from bindings.ts directly,
// so the transport can be swapped without touching components.
import { commands, type CoreError, type Result } from "./bindings";

export type { CoreError };

/** Unwrap a specta Result, throwing the typed CoreError on failure. */
async function unwrap<T>(p: Promise<Result<T, CoreError>>): Promise<T> {
  const r = await p;
  if (r.status === "error") throw r.error;
  return r.data;
}

export function errorKind(e: unknown): CoreError["kind"] | "unknown" {
  if (e && typeof e === "object" && "kind" in e) {
    return (e as CoreError).kind;
  }
  return "unknown";
}

export const isUnlocked = (): Promise<boolean> => commands.isUnlocked();
export const unlock = (): Promise<null> => unwrap(commands.unlock());
export const lock = (): Promise<void> => commands.lock();
export const startFresh = (): Promise<null> => unwrap(commands.startFresh());
