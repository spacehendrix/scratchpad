<script lang="ts">
  import { documents } from "../stores/documents.svelte";
  import { session } from "../stores/session.svelte";
  import { togglePin } from "../api";
  import { dispatch, type Keymap } from "../keyboard";
  import DocRow from "./DocRow.svelte";

  let selectedIndex = $state(0);
  let now = $state(Date.now());

  $effect(() => {
    documents.refresh();
    const tick = setInterval(() => (now = Date.now()), 30_000);
    return () => clearInterval(tick);
  });

  const clamp = () => {
    if (selectedIndex >= documents.catalog.length) {
      selectedIndex = Math.max(0, documents.catalog.length - 1);
    }
  };

  function open(id: string | null) {
    documents.openId = id;
    session.view = "editor";
  }

  const keymap: Keymap = {
    n: () => open(null),
    j: () => {
      selectedIndex = Math.min(selectedIndex + 1, documents.catalog.length - 1);
    },
    k: () => {
      selectedIndex = Math.max(selectedIndex - 1, 0);
    },
    arrowdown: () => keymap.j(new KeyboardEvent("keydown")),
    arrowup: () => keymap.k(new KeyboardEvent("keydown")),
    enter: () => {
      const meta = documents.catalog[selectedIndex];
      if (meta) open(meta.id);
    },
    p: async () => {
      const meta = documents.catalog[selectedIndex];
      if (!meta) return;
      await togglePin(meta.id);
      const id = meta.id;
      await documents.refresh();
      selectedIndex = Math.max(0, documents.catalog.findIndex((m) => m.id === id));
    },
  };

  function onkeydown(e: KeyboardEvent) {
    dispatch(keymap, e);
    clamp();
  }
</script>

<svelte:window {onkeydown} />

<div class="browse">
  {#if documents.catalog.length === 0}
    <p class="empty">nothing here · [n] new</p>
  {:else}
    <div class="list">
      {#each documents.catalog as meta, i (meta.id)}
        <DocRow {meta} {now} selected={i === selectedIndex} onopen={() => open(meta.id)} />
      {/each}
    </div>
  {/if}
  <p class="hints">[n] new · [⏎] open · [j/k] move · [p] pin</p>
</div>

<style>
  .browse {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 2rem 1.5rem 0.75rem;
  }
  .list {
    flex: 1;
    overflow-y: auto;
  }
  .empty {
    flex: 1;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0.4;
  }
  .hints {
    opacity: 0.35;
    padding-top: 0.5rem;
    text-align: center;
  }
</style>
