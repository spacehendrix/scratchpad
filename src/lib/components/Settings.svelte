<script lang="ts">
  import { tick, untrack } from "svelte";
  import { session } from "../stores/session.svelte";
  import { settings } from "../stores/settings.svelte";
  import { themes } from "../themes";
  import { applyTheme } from "../themes";
  import { applyFont, applyFontSize, fonts, MAX_SIZE, MIN_SIZE } from "../fonts";
  import { dispatch, type Keymap } from "../keyboard";

  type Row =
    | { kind: "theme"; id: string; label: string }
    | { kind: "font"; id: string; label: string }
    | { kind: "size" };

  const rows: Row[] = [
    ...themes.map((t) => ({ kind: "theme", id: t.id, label: t.label }) as Row),
    ...fonts.map((f) => ({ kind: "font", id: f.id, label: f.label }) as Row),
    { kind: "size" },
  ];
  const firstFontRow = themes.length;
  const sizeRow = rows.length - 1;

  let draftTheme = $state(untrack(() => settings.theme));
  let draftFont = $state(untrack(() => settings.font));
  let draftSize = $state(untrack(() => settings.fontSize));
  let selectedIndex = $state(
    untrack(() => Math.max(0, themes.findIndex((t) => t.id === settings.theme))),
  );
  let listEl = $state<HTMLElement | undefined>(undefined);

  // Moving only moves — nothing is previewed or changed until chosen.
  function move(delta: number) {
    selectedIndex = Math.min(Math.max(selectedIndex + delta, 0), rows.length - 1);
    tick().then(() =>
      listEl?.querySelector(".selected")?.scrollIntoView({ block: "nearest" }),
    );
  }

  /** Space (or click): adopt the selected row's value and preview it. */
  function choose() {
    const row = rows[selectedIndex];
    if (row.kind === "theme") {
      draftTheme = row.id;
      applyTheme(row.id);
    } else if (row.kind === "font") {
      draftFont = row.id;
      applyFont(row.id);
    }
  }

  function adjustSize(delta: number) {
    if (rows[selectedIndex].kind !== "size") return;
    draftSize = Math.max(MIN_SIZE, Math.min(MAX_SIZE, draftSize + delta));
    applyFontSize(draftSize);
  }

  async function commit() {
    await settings.commit({ theme: draftTheme, font: draftFont, fontSize: draftSize });
    session.view = "browse";
  }

  function cancel() {
    settings.applyAll();
    session.view = "browse";
  }

  const keymap: Keymap = {
    j: () => move(1),
    k: () => move(-1),
    arrowdown: () => move(1),
    arrowup: () => move(-1),
    h: () => adjustSize(-1),
    l: () => adjustSize(1),
    arrowleft: () => adjustSize(-1),
    arrowright: () => adjustSize(1),
    " ": () => choose(),
    enter: () => {
      choose();
      commit();
    },
    "!escape": () => cancel(),
  };
</script>

<svelte:window onkeydown={(e) => dispatch(keymap, e)} />

<div class="settingsview">
  <div class="list" bind:this={listEl}>
    <p class="section">┄┄ theme ┄┄</p>
    {#each rows as row, i (i)}
      {#if i === firstFontRow}
        <p class="section">┄┄ font ┄┄</p>
      {:else if i === sizeRow}
        <p class="section">┄┄ size ┄┄</p>
      {/if}
      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <div
        class="row"
        class:selected={i === selectedIndex}
        onclick={() => {
          selectedIndex = i;
          choose();
        }}
        ondblclick={commit}
        role="button"
        tabindex="-1"
      >
        <span class="marker">{i === selectedIndex ? "❯" : " "}</span>
        {#if row.kind === "theme"}
          <span class="name">{row.label}</span>
          {#if row.id === draftTheme}<span class="current">●</span>{/if}
          <span class="swatch">
            <span style:color="var(--accent)">■</span><span style:color="var(--accent2)">■</span
            ><span style:color="var(--ok)">■</span><span style:color="var(--warn)">■</span><span
              style:color="var(--err)">■</span
            >
          </span>
        {:else if row.kind === "font"}
          <span class="name" style:font-family={fonts.find((f) => f.id === row.id)?.stack}
            >{row.label}</span
          >
          {#if row.id === draftFont}<span class="current">●</span>{/if}
          <span class="sample" style:font-family={fonts.find((f) => f.id === row.id)?.stack}
            >abc [x] 123</span
          >
        {:else}
          <span class="name">◂ {draftSize} px ▸</span>
          <span class="sample">[h/l] adjust</span>
        {/if}
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
  .section {
    color: var(--fg-dim);
    opacity: 0.75;
    padding: 0.6rem 0.5rem 0.2rem;
  }
  .section:first-child {
    padding-top: 0;
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
  .sample {
    color: var(--fg-dim);
  }
</style>
