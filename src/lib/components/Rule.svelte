<script lang="ts">
  // Labeled horizontal rule: "── label ────────────────". The fill is a long
  // run of box-drawing glyphs clipped by flexbox — always pixel-aligned with
  // the monospace grid, no measuring needed.
  //
  // With `intro`, the whole line dissolves rather than sweeping: it mounts
  // as a heavy bar, then churns through ascii noise while characters freeze
  // — first to "═" (every char reaches it before any moves on), then to "─"
  // — and settles on "─".
  import { untrack } from "svelte";

  let {
    label = "",
    right = "",
    intro = false,
  }: { label?: string; right?: string; intro?: boolean } = $props();

  const N = 500;
  const NOISE = "█▓▒░#*+~×≡";
  const T = 32; // dissolve frames (~1.4s at 45ms)

  let cap = $state(untrack(() => (intro ? "██" : "──")));
  let fill = $state(untrack(() => (intro ? "█".repeat(N) : "─".repeat(N))));
  let endCap = $state(untrack(() => (intro ? "██" : "──")));

  function settle() {
    cap = "──";
    fill = "─".repeat(N);
    endCap = "──";
  }

  $effect(() => {
    if (!intro) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      settle();
      return;
    }
    let timer: ReturnType<typeof setInterval> | undefined;
    const start = setTimeout(() => {
      // Per-char random freeze times: everything reaches "═" (thresholds up
      // to 0.6) before anything flips to "─" (from 0.72), so the line
      // visibly ends as all "═", then dissolves once more into "─".
      const toEquals = Array.from({ length: N + 4 }, () => 0.1 + Math.random() * 0.5);
      const toDash = Array.from({ length: N + 4 }, () => 0.72 + Math.random() * 0.23);
      let f = 0;
      timer = setInterval(() => {
        f++;
        const t = f / T;
        const charAt = (i: number) =>
          t >= toDash[i]
            ? "─"
            : t >= toEquals[i]
              ? "═"
              : NOISE[Math.floor(Math.random() * NOISE.length)];
        cap = charAt(0) + charAt(1);
        let s = "";
        for (let j = 0; j < N; j++) s += charAt(2 + j);
        fill = s;
        endCap = charAt(N + 2) + charAt(N + 3);
        if (f >= T) {
          clearInterval(timer);
          settle();
        }
      }, 45);
    }, 1000);
    return () => {
      clearTimeout(start);
      clearInterval(timer);
    };
  });
</script>

<div class="rule">
  <span class="cap">{cap}</span>
  {#if label}<span class="label">{label}</span>{/if}
  <span class="fill" aria-hidden="true">{fill}</span>
  {#if right}
    <span class="right">{right}</span>
    <span class="cap">{endCap}</span>
  {/if}
</div>

<style>
  .rule {
    display: flex;
    align-items: baseline;
    gap: 1ch;
    color: var(--fg-dim);
    white-space: nowrap;
    overflow: hidden;
  }
  .label {
    color: var(--fg);
  }
  .fill {
    flex: 1;
    overflow: hidden;
  }
  .right {
    color: var(--fg-dim);
  }
</style>
