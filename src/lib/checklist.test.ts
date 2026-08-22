import { describe, expect, it } from "vitest";
import { classifyLine, toggleAtIndex, toggleAtLine, toggleLineCheckbox } from "./checklist";

describe("classifyLine", () => {
  it("recognizes checkbox variants", () => {
    expect(classifyLine("[ ] milk").kind).toBe("todo");
    expect(classifyLine("[x] milk").kind).toBe("done");
    expect(classifyLine("[X] milk").kind).toBe("done");
    expect(classifyLine("- [ ] milk").kind).toBe("todo");
    expect(classifyLine("* [x] milk").kind).toBe("done");
    expect(classifyLine("  - [ ] indented").kind).toBe("todo");
  });
  it("recognizes headings and bullets", () => {
    expect(classifyLine("# big").kind).toBe("h1");
    expect(classifyLine("## small").kind).toBe("h2");
    expect(classifyLine("- item").kind).toBe("bullet");
    expect(classifyLine("* item").kind).toBe("bullet");
    expect(classifyLine("just text").kind).toBe("plain");
  });
  it("head/box/tail reassemble to the exact line", () => {
    for (const line of ["- [ ] milk", "  * [x] done thing", "[ ]", "[ ] x"]) {
      const i = classifyLine(line);
      expect(i.head + i.box + i.tail).toBe(line);
    }
  });
});

describe("toggleLineCheckbox", () => {
  it("toggles both directions, preserving prefix", () => {
    expect(toggleLineCheckbox("- [ ] milk")).toBe("- [x] milk");
    expect(toggleLineCheckbox("- [x] milk")).toBe("- [ ] milk");
    expect(toggleLineCheckbox("[X] milk")).toBe("[ ] milk");
    expect(toggleLineCheckbox("  [ ] indented")).toBe("  [x] indented");
  });
  it("returns null for non-checkbox lines", () => {
    expect(toggleLineCheckbox("plain")).toBeNull();
    expect(toggleLineCheckbox("# heading")).toBeNull();
    expect(toggleLineCheckbox("")).toBeNull();
  });
  it("keeps length identical (caret never shifts)", () => {
    const line = "- [ ] milk";
    expect(toggleLineCheckbox(line)!.length).toBe(line.length);
  });
});

describe("toggleAtLine / toggleAtIndex", () => {
  const text = "title\n[ ] one\n[x] two\nplain";
  it("toggles only the addressed line", () => {
    expect(toggleAtLine(text, 1)).toBe("title\n[x] one\n[x] two\nplain");
    expect(toggleAtLine(text, 2)).toBe("title\n[ ] one\n[ ] two\nplain");
    expect(toggleAtLine(text, 0)).toBeNull();
    expect(toggleAtLine(text, 99)).toBeNull();
  });
  it("maps caret index to its line", () => {
    const caretInOne = text.indexOf("one");
    expect(toggleAtIndex(text, caretInOne)).toBe("title\n[x] one\n[x] two\nplain");
    expect(toggleAtIndex(text, 0)).toBeNull();
    expect(toggleAtIndex(text, text.length)).toBeNull(); // last line is plain
  });
  it("handles unicode content before the caret line", () => {
    const t = "émojis 🎸🎸\n[ ] tâche";
    expect(toggleAtIndex(t, t.length)).toBe("émojis 🎸🎸\n[x] tâche");
  });
});
