<script lang="ts">
  // Three ascii-art metric panels above the browse list:
  //  activity — 14-day sparkline of notes touched per day
  //  tasks    — checkbox completion across active notes
  //  storage  — database footprint against the 5 GB cap
  import { documents } from "../stores/documents.svelte";
  import { storageStats, type StorageStats } from "../api";
  import { fmtBytes } from "../format";
  import Rule from "./Rule.svelte";

  let stats = $state<StorageStats | null>(null);

  $effect(() => {
    void documents.catalog;
    storageStats()
      .then((s) => (stats = s))
      .catch(() => (stats = null));
  });

  const DAYS = 14;
  const LEVELS = ["▁", "▂", "▃", "▄", "▅", "▆", "▇", "█"];
  const DAY_MS = 86_400_000;

  const activity = $derived.by(() => {
    const counts = Array(DAYS).fill(0) as number[];
    const now = Date.now();
    for (const m of documents.catalog) {
      const age = Math.floor((now - m.updatedAt) / DAY_MS);
      if (age >= 0 && age < DAYS) counts[DAYS - 1 - age]++;
    }
    const max = Math.max(1, ...counts);
    const spark = counts
      .map((c) => (c === 0 ? "▁" : LEVELS[Math.min(7, Math.round((c / max) * 7))]))
      .join("");
    return { spark, total: counts.reduce((a, b) => a + b, 0) };
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

  const BAR = 12;
  function bar(fraction: number): { filled: string; empty: string } {
    const k = Math.max(0, Math.min(BAR, Math.round(fraction * BAR)));
    return { filled: "▓".repeat(k), empty: "░".repeat(BAR - k) };
  }

  const taskBar = $derived(bar(tasks.total > 0 ? tasks.done / tasks.total : 0));
  const storageBar = $derived(bar(stats ? Number(stats.dbBytes) / Number(stats.limitBytes) : 0));
</script>

<div class="dash">
  <div class="panel">
    <Rule label="activity" />
    <pre class="art"><span class="graph">{activity.spark}</span> <span class="dim"
        >{activity.total} · 14d</span
      ></pre>
  </div>
  <div class="panel">
    <Rule label="tasks" />
    <pre class="art"><span class="graph">{taskBar.filled}</span><span class="dim"
        >{taskBar.empty}</span
      > <span class="dim"
        >{tasks.total > 0 ? `${tasks.done}/${tasks.total}` : "none"}</span
      ></pre>
  </div>
  <div class="panel">
    <Rule label="storage" />
    <pre class="art"><span class="graph">{storageBar.filled}</span><span class="dim"
        >{storageBar.empty}</span
      > <span class="dim">{stats ? fmtBytes(Number(stats.dbBytes)) : "…"}</span
      ></pre>
  </div>
</div>

<style>
  .dash {
    display: flex;
    gap: 3ch;
    margin: 0.3rem 0 0.9rem;
  }
  .panel {
    flex: 1;
    min-width: 0;
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
