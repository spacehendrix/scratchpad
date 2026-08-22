<script lang="ts">
  import { untrack } from "svelte";
  import { session } from "../stores/session.svelte";
  import { settings } from "../stores/settings.svelte";
  import { themes } from "../themes";
  import { dispatch, type Keymap } from "../keyboard";

  let selectedIndex = $state(
    untrack(() => Math.max(0, themes.findIndex((t) => t.id === settings.theme))),
  );

  // Moving the selection live-previews; Enter commits; Esc reverts.
  function move(delta: number) {
    selectedIndex = Math.min(Math.max(selectedIndex + delta, 0), themes.length - 1);
    settings.preview(themes[selectedIndex].id);
  }

  async function commit() {
    await settings.commit(themes[selectedIndex].id);
    session.view = "browse";
  }

  function cancel() {
    settings.preview(settings.theme);
    session.view = "browse";
  }

  const keymap: Keymap = {
    j: () => move(1),
    k: () => move(-1),
    arrowdown: () => move(1),
    arrowup: () => move(-1),
    enter: () => commit(),
    "!escape": () => cancel(),
  };
</script>

<svelte:window onkeydown={(e) => dispatch(keymap, e)} />

<div class="settingsview">
  <div class="list">
    {#each themes as theme, i (theme.id)}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="row"
        class:selected={i === selectedIndex}
        onclick={() => {
          selectedIndex = i;
          settings.preview(theme.id);
        }}
        ondblclick={commit}
        role="button"
        tabindex="-1"
      >
        <span class="marker">{i === selectedIndex ? "❯" : " "}</span>
        <span class="name">{theme.label}</span>
        {#if theme.id === settings.theme}<span class="current">●</span>{/if}
        <span class="swatch">
          <span style:color="var(--accent)">■</span><span style:color="var(--accent2)">■</span><span
            style:color="var(--ok)">■</span
          ><span style:color="var(--warn)">■</span><span style:color="var(--err)">■</span>
        </span>
      </div>
    {/each}
  </div>
</div>

<style>
  .settingsview {
    display: flex;
    flex-direction: column;
    height: 100%;
    padding: 0.4rem 0;
  }
  .list {
    flex: 1;
    overflow-y: auto;
  }
  .row {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
    padding: 0.15rem 0.5rem;
  }
  .row.selected {
    background: var(--sel-bg);
  }
  .marker {
    width: 1ch;
    color: var(--accent);
  }
  .name {
    flex: 1;
  }
  .current {
    color: var(--ok);
  }
  .swatch {
    letter-spacing: 0.15rem;
  }
</style>
