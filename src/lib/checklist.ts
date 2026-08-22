// Pure text logic for the editor: line classification for the styled
// overlay, and checkbox toggling. No DOM, fully unit-tested.

export type LineKind = "h1" | "h2" | "bullet" | "todo" | "done" | "plain";

export interface LineInfo {
  kind: LineKind;
  /** For todo/done lines: text before / the box itself / text after. */
  head: string;
  box: string;
  tail: string;
  /** Full line text (always exact — the overlay must mirror it 1:1). */
  text: string;
}

const BOX_RE = /^(\s*(?:[-*] )?)(\[[ xX]\])( ?)/;

export function classifyLine(text: string): LineInfo {
  const m = BOX_RE.exec(text);
  if (m) {
    const box = m[2];
    const head = m[1];
    const tail = text.slice(head.length + box.length);
    const kind: LineKind = box === "[ ]" ? "todo" : "done";
    return { kind, head, box, tail, text };
  }
  const t = text.trimStart();
  if (t.startsWith("# ")) return plain("h1", text);
  if (t.startsWith("## ")) return plain("h2", text);
  if (t.startsWith("- ") || t.startsWith("* ")) return plain("bullet", text);
  return plain("plain", text);
}

function plain(kind: LineKind, text: string): LineInfo {
  return { kind, head: "", box: "", tail: "", text };
}

/** Toggle the checkbox on one line; null when the line has none. */
export function toggleLineCheckbox(line: string): string | null {
  const info = classifyLine(line);
  if (info.kind === "todo") return info.head + "[x]" + info.tail;
  if (info.kind === "done") return info.head + "[ ]" + info.tail;
  return null;
}

/** Toggle the checkbox on the 0-based `lineNo`; null when not a checkbox. */
export function toggleAtLine(text: string, lineNo: number): string | null {
  const lines = text.split("\n");
  if (lineNo < 0 || lineNo >= lines.length) return null;
  const toggled = toggleLineCheckbox(lines[lineNo]);
  if (toggled === null) return null;
  lines[lineNo] = toggled;
  return lines.join("\n");
}

/** Toggle the checkbox on the line containing char `index` (caret position). */
export function toggleAtIndex(text: string, index: number): string | null {
  const upTo = text.slice(0, Math.max(0, Math.min(index, text.length)));
  const lineNo = upTo.split("\n").length - 1;
  return toggleAtLine(text, lineNo);
}
