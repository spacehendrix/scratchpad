<script lang="ts">
  import { storageStats, type StorageStats } from "../api";
  import { session } from "../stores/session.svelte";
  import { documents } from "../stores/documents.svelte";
  import { fmtBytes } from "../format";

  let stats = $state<StorageStats | null>(null);

  async function refresh() {
    try {
      stats = await storageStats();
    } catch {
      stats = null;
    }
  }

  $effect(() => {
    // Re-fetch when the view changes (saves/deletes happen on transitions)
    // and every minute while open.
    void session.view;
    void documents.catalog;
    refresh();
    const tick = setInterval(refresh, 60_000);
    return () => clearInterval(tick);
  });

  const HINTS: Record<string, string> = {
    browse: "[n] new · [⏎] open · [j/k] move · [p] pin · [/] search · [,] theme",
    editor: "[esc] back · [⌘⏎] toggle",
    search: "[⇥] scope · [↑↓] move · [⏎] open · [esc] back",
    settings: "[j/k] preview · [⏎] apply · [esc] cancel",
  };
</script>

<div class="statusbar">
  <span class="cap">──</span>
  {#if stats}
    <span class="stats">
      {stats.docCount - stats.archivedCount} notes
      {#if stats.pinnedCount > 0}· {stats.pinnedCount} ◆{/if}
      {#if stats.archivedCount > 0}· {stats.archivedCount} archived{/if}
      · {fmtBytes(Number(stats.dbBytes))} / {fmtBytes(Number(stats.limitBytes))}
    </span>
    {#if stats.overCapacity}
      <span class="warn">▲ storage full — unpin or delete</span>
    {/if}
  {/if}
  <span class="fill" aria-hidden="true">{"─".repeat(500)}</span>
  <span class="hints">{HINTS[session.view] ?? ""}</span>
  <span class="cap">──</span>
</div>

<style>
  .statusbar {
    display: flex;
    align-items: baseline;
    gap: 1ch;
    color: var(--fg-dim);
    white-space: nowrap;
    overflow: hidden;
  }
  .stats {
    color: var(--fg-dim);
  }
  .warn {
    color: var(--warn);
  }
  .fill {
    flex: 1;
    overflow: hidden;
    min-width: 2ch;
  }
  .hints {
    color: var(--fg-dim);
  }
</style>
