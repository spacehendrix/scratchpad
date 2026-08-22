<script lang="ts">
  import { tick, untrack } from "svelte";
  import { getDocument, saveDocument } from "../api";
  import { documents } from "../stores/documents.svelte";
  import { session } from "../stores/session.svelte";
  import { dispatch, type Keymap } from "../keyboard";
  import { classifyLine, toggleAtIndex, toggleAtLine } from "../checklist";

  // Deliberately captures openId at mount — the editor lives for exactly one
  // document; navigation remounts it.
  let id = $state<string | null>(untrack(() => documents.openId));
  let title = $state("");
  let body = $state("");
  let loaded = $state(untrack(() => id === null));
  let dirty = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let bodyEl = $state<HTMLTextAreaElement | undefined>(undefined);
  let preEl = $state<HTMLPreElement | undefined>(undefined);
  let saving: Promise<unknown> = Promise.resolve();

  const lines = $derived(body.split("\n").map(classifyLine));

  $effect(() => {
    if (id) {
      getDocument(id).then((doc) => {
        title = doc.title ?? "";
        body = doc.body;
        loaded = true;
      });
    }
  });

  $effect(() => {
    if (loaded) bodyEl?.focus();
  });

  function markDirty() {
    dirty = true;
    clearTimeout(saveTimer);
    saveTimer = setTimeout(flush, 800);
  }

  /** Persist if there is anything to persist. Chained so saves never race. */
  function flush(): Promise<unknown> {
    clearTimeout(saveTimer);
    if (!dirty) return saving;
    // Never persist a brand-new, still-empty document.
    if (id === null && title.trim() === "" && body.trim() === "") {
      dirty = false;
      return saving;
    }
    dirty = false;
    saving = saving.then(async () => {
      const meta = await saveDocument(id, title.trim() || null, body);
      id = meta.id;
    });
    return saving;
  }

  async function back() {
    await flush();
    await documents.refresh();
    documents.openId = null;
    session.view = "browse";
  }

  /** Cmd+Enter: toggle the checkbox on the caret line. "[ ]"↔"[x]" have the
   *  same length, so the caret never shifts — but the programmatic value
   *  update still resets it, so it is restored explicitly. */
  async function toggleAtCaret() {
    if (!bodyEl) return;
    const s = bodyEl.selectionStart;
    const e = bodyEl.selectionEnd;
    const t = toggleAtIndex(body, s);
    if (t === null) return;
    body = t;
    markDirty();
    await tick();
    bodyEl.setSelectionRange(s, e);
    bodyEl.focus();
  }

  function clickToggle(lineNo: number) {
    const t = toggleAtLine(body, lineNo);
    if (t === null) return;
    body = t;
    markDirty();
    bodyEl?.focus();
  }

  function syncScroll() {
    if (preEl && bodyEl) {
      preEl.scrollTop = bodyEl.scrollTop;
      preEl.scrollLeft = bodyEl.scrollLeft;
    }
  }

  const keymap: Keymap = {
    "!escape": () => back(),
    "!cmd+enter": () => toggleAtCaret(),
  };
</script>

<svelte:window onkeydown={(e) => dispatch(keymap, e)} />

<div class="editor">
  {#if loaded}
    <input
      class="title"
      bind:value={title}
      oninput={markDirty}
      placeholder="untitled"
      spellcheck="false"
      autocomplete="off"
    />
    <div class="pane">
      <textarea
        class="layer input"
        bind:this={bodyEl}
        bind:value={body}
        oninput={markDirty}
        onscroll={syncScroll}
        spellcheck="false"
      ></textarea>
      <!-- Styled mirror: identical text/metrics, painted over the transparent
           textarea text. pointer-events pass through except on checkboxes. -->
      <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_noninteractive_tabindex -->
      <pre class="layer ghost" bind:this={preEl} aria-hidden="true">{#each lines as ln, i}{#if i > 0}{"\n"}{/if}{#if ln.kind === "todo" || ln.kind === "done"}<span class="line {ln.kind}">{ln.head}<span class="cb" role="checkbox" aria-checked={ln.kind === "done"} tabindex="-1" onclick={() => clickToggle(i)}>{ln.box}</span><span class="tail">{ln.tail}</span></span>{:else}<span class="line {ln.kind}">{ln.text}</span>{/if}{/each}{"\n"}</pre>
    </div>
    <p class="hints">[esc] back · [⌘⏎] toggle{dirty ? " · ~" : ""}</p>
  {:else}
    <p class="loading">…</p>
  {/if}
</div>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    height: 100vh;
    padding: 2rem 1.5rem 0.75rem;
  }
  .title {
    background: transparent;
    border: none;
    border-bottom: 1px solid rgba(255, 255, 255, 0.12);
    color: var(--fg);
    font: inherit;
    padding: 0.25rem 0;
    margin-bottom: 0.75rem;
    outline: none;
    user-select: text;
  }
  .title::placeholder {
    opacity: 0.3;
    font-style: italic;
  }
  .pane {
    position: relative;
    flex: 1;
    overflow: hidden;
  }
  /* Both layers must be metric-identical for the mirror to align. */
  .layer {
    position: absolute;
    inset: 0;
    margin: 0;
    padding: 0;
    border: none;
    font: inherit;
    line-height: 1.5;
    white-space: pre-wrap;
    overflow-wrap: break-word;
    overflow-y: auto;
    background: transparent;
  }
  .input {
    color: transparent;
    caret-color: var(--fg);
    resize: none;
    outline: none;
    user-select: text;
    cursor: text;
    z-index: 0;
  }
  .ghost {
    pointer-events: none;
    z-index: 1;
    color: var(--fg);
    scrollbar-width: none;
  }
  .line.h1 {
    color: var(--accent, var(--fg));
    font-weight: 700;
  }
  .line.h2 {
    color: var(--accent2, var(--fg));
    font-weight: 700;
  }
  .line.bullet {
    color: var(--fg);
  }
  .cb {
    pointer-events: auto;
    cursor: pointer;
    color: var(--accent, var(--fg));
  }
  .line.done .cb {
    opacity: 0.6;
  }
  .line.done .tail {
    opacity: 0.45;
    text-decoration: line-through;
  }
  .loading {
    opacity: 0.4;
  }
  .hints {
    opacity: 0.35;
    padding-top: 0.5rem;
    text-align: center;
  }
</style>
