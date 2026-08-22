import { describe, expect, it } from "vitest";
import { displayName, fmtBytes, relativeAge } from "./format";

describe("fmtBytes", () => {
  it("buckets with sensible precision", () => {
    expect(fmtBytes(0)).toBe("0 B");
    expect(fmtBytes(512)).toBe("512 B");
    expect(fmtBytes(1400)).toBe("1.4 kB");
    expect(fmtBytes(890_000)).toBe("890 kB");
    expect(fmtBytes(1_400_000)).toBe("1.4 MB");
    expect(fmtBytes(142_000_000)).toBe("142 MB");
    expect(fmtBytes(5_000_000_000)).toBe("5.0 GB");
  });
});

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
