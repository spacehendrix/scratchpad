<script lang="ts">
  // Three ascii-art metric panels above the browse list:
  //  activity — 14-day sparkline of notes touched per day
  //  tasks    — checkbox completion across active notes
  //  storage  — database footprint against the 5 GB cap
  import { documents } from "../stores/documents.svelte";
  import { settings } from "../stores/settings.svelte";
  import { storageStats, type StorageStats } from "../api";
  import { fmtBytes } from "../format";
  import { sizeById, styleById } from "../dashboard";

  // Dashed rule, deliberately distinct from the app header/footer's solid
  // "─" chrome.
  const DASH_FILL = "┄".repeat(300);

  const style = $derived(styleById(settings.dashboardStyle));
  const size = $derived(sizeById(settings.dashboardSize));
  const enabled = $derived(new Set(settings.dashboardPanels));

  let stats = $state<StorageStats | null>(null);

  $effect(() => {
    void documents.catalog;
    storageStats()
      .then((s) => (stats = s))
      .catch(() => (stats = null));
  });

  const DAY_MS = 86_400_000;

  const activity = $derived.by(() => {
    const days = size.days;
    const counts = Array(days).fill(0) as number[];
    const now = Date.now();
    for (const m of documents.catalog) {
      const age = Math.floor((now - m.updatedAt) / DAY_MS);
      if (age >= 0 && age < days) counts[days - 1 - age]++;
    }
    const max = Math.max(1, ...counts);
    const spark = counts
      .map((c) => (c === 0 ? style.levels[0] : style.levels[Math.min(7, Math.round((c / max) * 7))]))
      .join("");
    return { spark, total: counts.reduce((a, b) => a + b, 0), days };
  });

  const tasks = $derived.by(() => {
    let done = 0;
    let total = 0;
    for (const m of documents.catalog) {
      if (m.archivedAt === null && m.checklist) {
        done += m.checklist.done;
        total += m.checklist.total;
      }
    }
    return { done, total };
  });

  function bar(fraction: number): { filled: string; empty: string } {
    const width = size.bar;
    const k = Math.max(0, Math.min(width, Math.round(fraction * width)));
    return { filled: style.gauge[0].repeat(k), empty: style.gauge[1].repeat(width - k) };
  }

  const taskBar = $derived(bar(tasks.total > 0 ? tasks.done / tasks.total : 0));
  const storageBar = $derived(bar(stats ? Number(stats.dbBytes) / Number(stats.limitBytes) : 0));
</script>

{#if enabled.size > 0}
  <div class="dash">
    {#if enabled.has("activity")}
      <div class="panel">
        <div class="phead">
          <span>┄┄</span><span class="plabel">activity</span><span class="pfill" aria-hidden="true"
            >{DASH_FILL}</span
          >
        </div>
        <pre class="art"><span class="graph">{activity.spark}</span> <span class="dim"
            >{activity.total} · {activity.days}d</span
          ></pre>
      </div>
    {/if}
    {#if enabled.has("tasks")}
      <div class="panel">
        <div class="phead">
          <span>┄┄</span><span class="plabel">tasks</span><span class="pfill" aria-hidden="true"
            >{DASH_FILL}</span
          >
        </div>
        <pre class="art"><span class="graph">{taskBar.filled}</span><span class="dim"
            >{taskBar.empty}</span
          > <span class="dim"
            >{tasks.total > 0 ? `${tasks.done}/${tasks.total}` : "none"}</span
          ></pre>
      </div>
    {/if}
    {#if enabled.has("storage")}
      <div class="panel">
        <div class="phead">
          <span>┄┄</span><span class="plabel">storage</span><span class="pfill" aria-hidden="true"
            >{DASH_FILL}</span
          >
        </div>
        <pre class="art"><span class="graph">{storageBar.filled}</span><span class="dim"
            >{storageBar.empty}</span
          > <span class="dim">{stats ? fmtBytes(Number(stats.dbBytes)) : "…"}</span
          ></pre>
      </div>
    {/if}
  </div>
{/if}

<style>
  .dash {
    display: flex;
    gap: 3ch;
    margin: 0.3rem 0 1.8rem;
  }
  .panel {
    flex: 1;
    min-width: 0;
    overflow: hidden;
  }
  .phead {
    display: flex;
    align-items: baseline;
    gap: 1ch;
    color: var(--fg-dim);
    white-space: nowrap;
    overflow: hidden;
    opacity: 0.75;
  }
  .plabel {
    color: var(--fg-dim);
  }
  .pfill {
    flex: 1;
    overflow: hidden;
  }
  .art {
    margin: 0.35rem 0 0;
    white-space: nowrap;
  }
  .graph {
    color: var(--accent);
  }
  .dim {
    color: var(--fg-dim);
  }
</style>
