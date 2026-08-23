<script lang="ts">
  import { unlock, startFresh, errorKind, type CoreError } from "../api";

  let { onunlocked }: { onunlocked: () => void } = $props();

  // Built programmatically so the borders always match the label width.
  const LABEL = "  s c r a t c h p a d  ";
  let label = $state(LABEL);

  // Outer ring: a second, wider box drawn one cell outside the logo box,
  // visible only while scrambling. Two gaps, diametrically opposite, orbit
  // the perimeter anticlockwise.
  const INNER_W = LABEL.length + 2;
  const COLS = INNER_W + 2;
  const ROWS = 5;
  const GAP = 8;
  // Perimeter cells in clockwise order; anticlockwise motion walks it backwards.
  const PATH: Array<[number, number]> = [];
  for (let c = 0; c < COLS; c++) PATH.push([0, c]);
  for (let r = 1; r < ROWS; r++) PATH.push([r, COLS - 1]);
  for (let c = COLS - 2; c >= 0; c--) PATH.push([ROWS - 1, c]);
  for (let r = ROWS - 2; r >= 1; r--) PATH.push([r, 0]);
  const P = PATH.length;
  const HALF = Math.floor(P / 2);
  const mod = (n: number, m: number) => ((n % m) + m) % m;

  let ringOffset = $state(0);
  let ringActive = $state(false);

  function ringChar(r: number, c: number): string {
    if (r === 0 && c === 0) return "┌";
    if (r === 0 && c === COLS - 1) return "┐";
    if (r === ROWS - 1 && c === 0) return "└";
    if (r === ROWS - 1 && c === COLS - 1) return "┘";
    return r === 0 || r === ROWS - 1 ? "─" : "│";
  }

  const logo = $derived.by(() => {
    const grid: string[][] = Array.from({ length: ROWS }, () => Array(COLS).fill(" "));
    if (ringActive) {
      const gaps = new Set<number>();
      for (let k = 0; k < GAP; k++) {
        gaps.add(mod(ringOffset + k, P));
        gaps.add(mod(ringOffset + HALF + k, P));
      }
      PATH.forEach(([r, c], idx) => {
        grid[r][c] = gaps.has(idx) ? " " : ringChar(r, c);
      });
    }
    const inner = [
      `┌${"─".repeat(LABEL.length)}┐`,
      `│${label}│`,
      `└${"─".repeat(LABEL.length)}┘`,
    ];
    inner.forEach((line, i) => {
      [...line].forEach((ch, j) => {
        grid[1 + i][1 + j] = ch;
      });
    });
    // Split off the ring columns so CSS can nudge them toward the box.
    return grid.map((row) => ({
      left: row[0],
      mid: row.slice(1, -1).join(""),
      right: row[COLS - 1],
    }));
  });

  // Periodic "decode" scramble: letters churn through random glyphs and
  // resolve left-to-right. Spaces are never touched, so the width (and the
  // box) stays fixed.
  const POOL = "abcdefghijklmnopqrstuvwxyz#*+=<>?/\\|~^%$&@!";
  const LETTER_IDXS = [...LABEL].map((c, i) => (c === " " ? -1 : i)).filter((i) => i >= 0);

  function scrambleOnce(): () => void {
    // Full-churn hold first, then resolve left-to-right.
    const HOLD_FRAMES = 22;
    const RESOLVE_FRAMES = 16;
    const FRAMES = HOLD_FRAMES + RESOLVE_FRAMES;
    let frame = 0;
    ringActive = true;
    const timer = setInterval(() => {
      frame++;
      ringOffset = mod(ringOffset - 1, P); // anticlockwise
      const progress = Math.max(0, frame - HOLD_FRAMES) / RESOLVE_FRAMES;
      label = [...LABEL]
        .map((c, i) => {
          if (c === " ") return " ";
          const resolved = LETTER_IDXS.indexOf(i) / LETTER_IDXS.length < progress;
          return resolved ? c : POOL[Math.floor(Math.random() * POOL.length)];
        })
        .join("");
      if (frame >= FRAMES) {
        clearInterval(timer);
        label = LABEL;
        ringActive = false;
      }
    }, 45);
    return () => {
      clearInterval(timer);
      ringActive = false;
    };
  }

  $effect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let stopRun: (() => void) | undefined;
    const first = setTimeout(() => (stopRun = scrambleOnce()), 900);
    const every = setInterval(() => (stopRun = scrambleOnce()), 4200);
    return () => {
      clearTimeout(first);
      clearInterval(every);
      stopRun?.();
      label = LABEL;
      ringActive = false;
    };
  });

  type Status = "authenticating" | "done" | "denied" | "corrupt" | "error";
  let status = $state<Status>("authenticating");
  let detail = $state("");
  let confirmText = $state("");
  let confirmInput = $state<HTMLInputElement | undefined>(undefined);

  /** Brief "typed" reveal before handing over to the app. */
  function finish() {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      onunlocked();
      return;
    }
    status = "done";
    setTimeout(onunlocked, 420);
  }

  async function attempt() {
    status = "authenticating";
    try {
      await unlock();
      finish();
    } catch (e) {
      const kind = errorKind(e);
      if (kind === "corrupt") {
        status = "corrupt";
        confirmText = "";
      } else if (kind === "keychainDenied") {
        status = "denied";
      } else {
        status = "error";
        detail =
          kind === "io" ? ((e as CoreError & { detail: string }).detail ?? "") : kind;
      }
    }
  }

  async function eraseAndStartFresh() {
    if (confirmText !== "ERASE") return;
    status = "authenticating";
    try {
      await startFresh();
      finish();
    } catch (e) {
      status = "error";
      detail = errorKind(e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if ((status === "denied" || status === "error") && e.key === "r") {
      e.preventDefault();
      attempt();
    }
  }

  $effect(() => {
    attempt();
  });

  $effect(() => {
    if (status === "corrupt") confirmInput?.focus();
  });
</script>

<svelte:window {onkeydown} />

<main class="unlock">
  <div class="logo">
    {#each logo as line, i (i)}
      <pre
        class="row"
        class:tuck-below={i === 0}
        class:tuck-above={i === ROWS - 1}><span class="side-l">{line.left}</span>{line.mid}<span
          class="side-r">{line.right}</span></pre>
    {/each}
  </div>

  {#if status === "authenticating"}
    <p class="line dim">authenticating …</p>
  {:else if status === "done"}
    <p class="line typed ok">▸ unlocked</p>
  {:else if status === "denied"}
    <p class="line warn">✗ authentication cancelled</p>
    <p class="line dim">[r] retry</p>
  {:else if status === "corrupt"}
    <p class="line err">✗ stored data cannot be decrypted with the current key</p>
    <p class="line dim">
      the keychain entry was likely deleted or replaced. the old database will be
      quarantined (renamed, not deleted).
    </p>
    <p class="line">
      type <strong>ERASE</strong> + ⏎ to start fresh:
      <input
        bind:this={confirmInput}
        bind:value={confirmText}
        onkeydown={(e) => e.key === "Enter" && eraseAndStartFresh()}
        spellcheck="false"
        autocomplete="off"
      />
    </p>
  {:else}
    <p class="line err">✗ unlock failed{detail ? `: ${detail}` : ""}</p>
    <p class="line dim">[r] retry</p>
  {/if}
</main>

<style>
  .unlock {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    gap: 0.75rem;
  }
  .logo {
    color: var(--fg);
    margin-bottom: 1rem;
  }
  .row {
    margin: 0;
    /* Box-drawing glyphs only connect vertically at line-height 1. */
    line-height: 1;
  }
  /* Character cells are ~2x taller than wide, so the ring rows are pulled
     toward the box to even out the visual gap. */
  .tuck-below {
    margin-bottom: -0.55em;
  }
  .tuck-above {
    margin-top: -0.55em;
  }
  /* Ring strokes sit centered in their cells; nudge the side columns
     inward so the horizontal gap tightens to match. */
  .side-l {
    position: relative;
    left: 0.4ch;
  }
  .side-r {
    position: relative;
    left: -0.4ch;
  }
  .line {
    max-width: 34rem;
    text-align: center;
  }
  .dim {
    opacity: 0.55;
  }
  .warn {
    color: var(--warn);
  }
  .ok {
    color: var(--ok);
  }
  /* Monospace makes a steps() width animation read as typing. */
  .typed {
    overflow: hidden;
    white-space: nowrap;
    width: 10ch;
    animation: typing 0.32s steps(10);
  }
  @keyframes typing {
    from {
      width: 0;
    }
    to {
      width: 10ch;
    }
  }
  .err {
    color: var(--err);
  }
  input {
    background: transparent;
    border: none;
    border-bottom: 0.0625rem solid var(--fg);
    color: var(--fg);
    font: inherit;
    width: 6ch;
    outline: none;
    user-select: text;
  }
</style>
