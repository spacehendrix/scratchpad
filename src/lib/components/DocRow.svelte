<script lang="ts">
  import type { DocMeta } from "../bindings";
  import { displayName, relativeAge } from "../format";

  let {
    meta,
    selected,
    now,
    onopen,
  }: {
    meta: DocMeta;
    selected: boolean;
    now: number;
    onopen: () => void;
  } = $props();
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<div class="row" class:selected onclick={onopen} role="button" tabindex="-1">
  <span class="marker">{selected ? "❯" : " "}</span>
  <span class="pin">{meta.pinned ? "◆" : " "}</span>
  <span class="name" class:untitled={!meta.title && !meta.preview}>{displayName(meta)}</span>
  {#if meta.checklist}
    <span class="badge">[{meta.checklist.done}/{meta.checklist.total}]</span>
  {/if}
  <span class="age">{relativeAge(meta.updatedAt, now)}</span>
</div>

<style>
  .row {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.15rem 0.5rem;
    white-space: nowrap;
  }
  .row.selected {
    background: var(--sel-bg);
  }
  .marker {
    width: 1ch;
    color: var(--accent, var(--fg));
  }
  .pin {
    width: 1ch;
    color: var(--accent, var(--fg));
  }
  .name {
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .untitled {
    opacity: 0.4;
    font-style: italic;
  }
  .badge {
    opacity: 0.7;
  }
  .age {
    opacity: 0.45;
    min-width: 3ch;
    text-align: right;
  }
</style>
