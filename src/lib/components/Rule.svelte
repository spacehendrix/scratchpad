<script lang="ts">
  // Labeled horizontal rule: "── label ────────────────". The fill is a long
  // run of box-drawing glyphs clipped by flexbox — always pixel-aligned with
  // the monospace grid, no measuring needed.
  let { label = "" }: { label?: string } = $props();
  const FILL = "─".repeat(500);
</script>

<div class="rule">
  <span class="cap">──</span>
  {#if label}<span class="label">{label}</span>{/if}
  <span class="fill" aria-hidden="true">{FILL}</span>
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
</style>
