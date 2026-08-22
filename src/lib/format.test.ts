import { describe, expect, it } from "vitest";
import { displayName, relativeAge } from "./format";

describe("displayName", () => {
  it("prefers title, then preview, then untitled", () => {
    expect(displayName({ title: "t", preview: "p" })).toBe("t");
    expect(displayName({ title: null, preview: "p" })).toBe("p");
    expect(displayName({ title: null, preview: "" })).toBe("untitled");
  });
});

describe("relativeAge", () => {
  const now = 1_000_000_000_000;
  const min = 60_000;
  it("buckets", () => {
    expect(relativeAge(now - 10_000, now)).toBe("now");
    expect(relativeAge(now - 5 * min, now)).toBe("5m");
    expect(relativeAge(now - 3 * 60 * min, now)).toBe("3h");
    expect(relativeAge(now - 2 * 24 * 60 * min, now)).toBe("2d");
    expect(relativeAge(now - 3 * 7 * 24 * 60 * min, now)).toBe("3w");
    expect(relativeAge(now - 4 * 30 * 24 * 60 * min, now)).toBe("4mo");
  });
  it("never goes negative on clock skew", () => {
    expect(relativeAge(now + 60_000, now)).toBe("now");
  });
});
