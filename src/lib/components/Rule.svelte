<script lang="ts">
  // Labeled horizontal rule: "── label ────────────────". The fill is a long
  // run of box-drawing glyphs clipped by flexbox — always pixel-aligned with
  // the monospace grid, no measuring needed.
  //
  // With `intro`, the line mounts drawn in "═" and, after 1s, converts to
  // "─" in a left-to-right sweep (cap first, then the fill).
  import { untrack } from "svelte";

  let {
    label = "",
    right = "",
    intro = false,
  }: { label?: string; right?: string; intro?: boolean } = $props();

  const N = 500;
  const DONE = N + 2;
  // Intro is a mount-time decision by design.
  let k = $state(untrack(() => (intro ? 0 : DONE)));
  let fillEl = $state<HTMLElement | undefined>(undefined);

  const cap = $derived(k >= 2 ? "──" : k === 1 ? "─═" : "══");
  // The far-right cap converts when the sweep completes.
  const endCap = $derived(k >= DONE ? "──" : "══");
  const fill = $derived.by(() => {
    const converted = Math.max(0, Math.min(N, k - 2));
    return "─".repeat(converted) + "═".repeat(N - converted);
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
      const target = Math.min(DONE, visible + 6);
      sweep = setInterval(() => {
        k += 3;
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
