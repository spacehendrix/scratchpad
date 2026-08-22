<script lang="ts">
  import Unlock from "./lib/components/Unlock.svelte";
  import Browse from "./lib/components/Browse.svelte";
  import Editor from "./lib/components/Editor.svelte";
  import Search from "./lib/components/Search.svelte";
  import Settings from "./lib/components/Settings.svelte";
  import Rule from "./lib/components/Rule.svelte";
  import StatusBar from "./lib/components/StatusBar.svelte";
  import { session } from "./lib/stores/session.svelte";
  import { dispatch } from "./lib/keyboard";

  function onkeydown(e: KeyboardEvent) {
    if (session.locked) return;
    dispatch({ "!cmd+l": () => session.lockNow() }, e);
  }

  const LABELS: Record<string, string> = {
    browse: "scratchpad",
    editor: "edit",
    search: "search",
    settings: "theme",
  };
</script>

<svelte:window {onkeydown} />

{#if session.locked}
  <Unlock onunlocked={() => (session.locked = false)} />
{:else}
  <div class="app">
    <Rule label={LABELS[session.view] ?? "scratchpad"} />
    <main class="content">
      {#if session.view === "editor"}
        <Editor />
      {:else if session.view === "search"}
        <Search />
      {:else if session.view === "settings"}
        <Settings />
      {:else}
        <Browse />
      {/if}
    </main>
    <StatusBar />
  </div>
{/if}
<div class="scanlines" aria-hidden="true"></div>

<style>
  .app {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 1.75rem 1.25rem 0.6rem;
    gap: 0.4rem;
  }
  .content {
    flex: 1;
    min-height: 0;
    overflow: hidden;
  }
  /* Subtle CRT scanlines; purely decorative. */
  .scanlines {
    position: fixed;
    inset: 0;
    pointer-events: none;
    background: repeating-linear-gradient(
      0deg,
      rgba(0, 0, 0, 0.14) 0,
      rgba(0, 0, 0, 0.14) 1px,
      transparent 1px,
      transparent 3px
    );
    opacity: 0.3;
  }
  @media (prefers-reduced-motion: reduce) {
    .scanlines {
      display: none;
    }
  }
</style>
