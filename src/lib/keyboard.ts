// Central keyboard dispatch. Each view owns one Keymap and feeds window
// keydown events through dispatch(); only the mounted view listens, so maps
// never conflict.

export type Keymap = Record<string, (e: KeyboardEvent) => void>;

/** Normalize an event to a keymap key like "j", "cmd+enter", "escape". */
export function keyOf(e: Pick<KeyboardEvent, "key" | "metaKey" | "ctrlKey" | "altKey">): string {
  const parts: string[] = [];
  if (e.metaKey) parts.push("cmd");
  if (e.ctrlKey) parts.push("ctrl");
  if (e.altKey) parts.push("alt");
  parts.push(e.key.toLowerCase());
  return parts.join("+");
}

/** True when the event originates from a text-editing element. */
export function isEditing(e: KeyboardEvent): boolean {
  const t = e.target as HTMLElement | null;
  if (!t) return false;
  return t.tagName === "INPUT" || t.tagName === "TEXTAREA" || t.isContentEditable;
}

/**
 * Run the handler bound to the event's key, if any. Returns true when
 * handled. Plain (unmodified) keys are ignored while typing in an input
 * unless the map opts in with a leading "!" (e.g. "!escape").
 */
export function dispatch(map: Keymap, e: KeyboardEvent): boolean {
  const key = keyOf(e);
  const editing = isEditing(e);
  const handler = (editing ? undefined : map[key]) ?? map[`!${key}`];
  if (!handler) return false;
  e.preventDefault();
  handler(e);
  return true;
}
