<script lang="ts">
  // Labeled horizontal rule: "── label ────────────────". The fill is a long
  // run of box-drawing glyphs clipped by flexbox — always pixel-aligned with
  // the monospace grid, no measuring needed.
  //
  // With `intro`, the line mounts as a heavy bar and, after 1s, thins out in
  // a left-to-right sweep: each char steps through STAGES as the front
  // passes, so the line tapers from large to thin instead of flipping.
  import { untrack } from "svelte";

  let {
    label = "",
    right = "",
    intro = false,
  }: { label?: string; right?: string; intro?: boolean } = $props();

  const STAGES = ["█", "▓", "━", "═", "─"];
  const LAST = STAGES.length - 1;
  /** Front-steps a char lingers on each weight before thinning further. */
  const DWELL = 5;
  const N = 500;
  const DONE = N + 2 + DWELL * LAST;
  // Intro is a mount-time decision by design.
  let k = $state(untrack(() => (intro ? 0 : DONE)));
  let fillEl = $state<HTMLElement | undefined>(undefined);

  /** Glyph for the char at line-index i: thins as the front (k) passes it. */
  const glyph = (i: number) =>
    STAGES[Math.max(0, Math.min(LAST, Math.floor((k - i) / DWELL)))];

  const cap = $derived(glyph(0) + glyph(1));
  // The far-right cap converts when the sweep completes.
  const endCap = $derived(k >= DONE ? "──" : STAGES[0].repeat(2));
  const fill = $derived.by(() => {
    if (k >= DONE) return "─".repeat(N);
    let s = "";
    for (let j = 0; j < N; j++) s += glyph(2 + j);
    return s;
  });

  $effect(() => {
    if (!intro) return;
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      k = DONE;
      return;
    }
    let sweep: ReturnType<typeof setInterval> | undefined;
    const start = setTimeout(() => {
      // Only the visible width needs sweeping: content is N chars wide, so
      // scrollWidth / N gives the char width without extra measurement.
      const el = fillEl;
      const chPx = el ? el.scrollWidth / N : 8;
      const visible = el ? Math.ceil(el.clientWidth / chPx) : 120;
      const target = Math.min(DONE, visible + 2 + DWELL * LAST + 4);
      sweep = setInterval(() => {
        k += 1;
        if (k >= target) {
          k = DONE;
          clearInterval(sweep);
        }
      }, 12);
    }, 1000);
    return () => {
      clearTimeout(start);
      clearInterval(sweep);
    };
  });
</script>

<div class="rule">
  <span class="cap">{cap}</span>
  {#if label}<span class="label">{label}</span>{/if}
  <span class="fill" aria-hidden="true" bind:this={fillEl}>{fill}</span>
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
