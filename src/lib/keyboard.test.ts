import { describe, expect, it, vi } from "vitest";
import { dispatch, keyOf, type Keymap } from "./keyboard";

function fakeEvent(
  key: string,
  opts: { meta?: boolean; ctrl?: boolean; alt?: boolean; editing?: boolean } = {},
): KeyboardEvent {
  const target = { tagName: opts.editing ? "TEXTAREA" : "DIV", isContentEditable: false };
  return {
    key,
    metaKey: opts.meta ?? false,
    ctrlKey: opts.ctrl ?? false,
    altKey: opts.alt ?? false,
    target,
    preventDefault: vi.fn(),
  } as unknown as KeyboardEvent;
}

describe("keyOf", () => {
  it("normalizes plain keys to lowercase", () => {
    expect(keyOf(fakeEvent("J"))).toBe("j");
    expect(keyOf(fakeEvent("Escape"))).toBe("escape");
  });
  it("prefixes modifiers in stable order", () => {
    expect(keyOf(fakeEvent("Enter", { meta: true }))).toBe("cmd+enter");
    expect(keyOf(fakeEvent("k", { ctrl: true, alt: true }))).toBe("ctrl+alt+k");
  });
});

describe("dispatch", () => {
  it("runs the bound handler and preventDefaults", () => {
    const fn = vi.fn();
    const e = fakeEvent("n");
    expect(dispatch({ n: fn }, e)).toBe(true);
    expect(fn).toHaveBeenCalledOnce();
    expect(e.preventDefault).toHaveBeenCalledOnce();
  });

  it("ignores unbound keys", () => {
    expect(dispatch({}, fakeEvent("x"))).toBe(false);
  });

  it("suppresses plain keys while typing in an input", () => {
    const fn = vi.fn();
    expect(dispatch({ n: fn }, fakeEvent("n", { editing: true }))).toBe(false);
    expect(fn).not.toHaveBeenCalled();
  });

  it("'!' bindings fire even while typing", () => {
    const fn = vi.fn();
    const map: Keymap = { "!escape": fn };
    expect(dispatch(map, fakeEvent("Escape", { editing: true }))).toBe(true);
    expect(fn).toHaveBeenCalledOnce();
  });
});
