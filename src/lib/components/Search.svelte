<script lang="ts">
  import { search, type SearchHit } from "../api";
  import { documents } from "../stores/documents.svelte";
  import { session } from "../stores/session.svelte";
  import { dispatch, type Keymap } from "../keyboard";
  import { displayName, relativeAge } from "../format";

  let query = $state("");
  let scopeArchived = $state(false);
  let hits = $state<SearchHit[]>([]);
  let selectedIndex = $state(0);
  let busy = $state(false);
  let inputEl = $state<HTMLInputElement | undefined>(undefined);
  let generation = 0;
  let debounceTimer: ReturnType<typeof setTimeout> | undefined;
  const now = Date.now();

  $effect(() => {
    inputEl?.focus();
  });

  // Re-run on query/scope changes, debounced; stale responses are dropped
  // via the generation counter.
  $effect(() => {
    const q = query;
    const scope = scopeArchived;
    const gen = ++generation;
    clearTimeout(debounceTimer);
    if (q.trim() === "") {
      hits = [];
      busy = false;
      return;
    }
    busy = true;
    debounceTimer = setTimeout(async () => {
      try {
        const result = await search(q, scope);
        if (gen === generation) {
          hits = result;
          selectedIndex = 0;
          busy = false;
        }
      } catch {
        if (gen === generation) busy = false;
      }
    }, 150);
  });

  function open(hit: SearchHit) {
    documents.openId = hit.meta.id;
    session.view = "editor";
  }

  const keymap: Keymap = {
    "!escape": () => (session.view = "browse"),
    "!tab": () => (scopeArchived = !scopeArchived),
    "!arrowdown": () => (selectedIndex = Math.min(selectedIndex + 1, hits.length - 1)),
    "!arrowup": () => (selectedIndex = Math.max(selectedIndex - 1, 0)),
    "!enter": () => {
      const hit = hits[selectedIndex];
      if (hit) open(hit);
    },
  };
</script>

<svelte:window onkeydown={(e) => dispatch(keymap, e)} />

<div class="searchview">
  <div class="bar">
    <span class="prompt">/</span>
    <input
      bind:this={inputEl}
      bind:value={query}
      placeholder={scopeArchived ? "search the archive…" : "search…"}
      spellcheck="false"
      autocomplete="off"
    />
    <span class="scope" class:archived={scopeArchived}>
      {scopeArchived ? "[archive]" : "[active]"}
    </span>
    {#if busy}<span class="busy">…</span>{/if}
  </div>

  <div class="results">
    {#each hits as hit, i (hit.meta.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="hit"
        class:selected={i === selectedIndex}
        onclick={() => open(hit)}
        role="button"
        tabindex="-1"
      >
        <span class="marker">{i === selectedIndex ? "❯" : " "}</span>
        <span class="name">{displayName(hit.meta)}</span>
        {#if hit.inBody}
          <span class="snippet">{hit.snippet}</span>
        {/if}
        <span class="age">{relativeAge(hit.meta.updatedAt, now)}</span>
      </div>
    {:else}
      {#if query.trim() !== "" && !busy}
        <p class="empty">no matches</p>
      {/if}
    {/each}
  </div>

  <p class="hints">[⇥] scope · [↑↓] move · [⏎] open · [esc] back</p>
</div>

<style>
  .searchview {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 2rem 1.5rem 0.75rem;
  }
  .bar {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    padding-bottom: 0.4rem;
    margin-bottom: 0.6rem;
  }
  .prompt {
    color: var(--accent, var(--fg));
  }
  input {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--fg);
    font: inherit;
    outline: none;
    user-select: text;
  }
  input::placeholder {
    opacity: 0.3;
  }
  .scope {
    opacity: 0.5;
  }
  .scope.archived {
    color: var(--accent, var(--fg));
    opacity: 1;
  }
  .busy {
    opacity: 0.5;
  }
  .results {
    flex: 1;
    overflow-y: auto;
  }
  .hit {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.15rem 0.5rem;
    white-space: nowrap;
  }
  .hit.selected {
    background: var(--sel-bg, rgba(255, 255, 255, 0.08));
  }
  .marker {
    width: 1ch;
    color: var(--accent, var(--fg));
  }
  .name {
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 40%;
  }
  .snippet {
    flex: 1;
    opacity: 0.5;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .age {
    opacity: 0.45;
    margin-left: auto;
  }
  .empty {
    opacity: 0.4;
    padding: 1rem 0.5rem;
  }
  .hints {
    opacity: 0.35;
    padding-top: 0.5rem;
    text-align: center;
  }
</style>
