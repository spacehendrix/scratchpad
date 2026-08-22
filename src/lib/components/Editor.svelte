<script lang="ts">
  import { untrack } from "svelte";
  import { getDocument, saveDocument } from "../api";
  import { documents } from "../stores/documents.svelte";
  import { session } from "../stores/session.svelte";
  import { dispatch, type Keymap } from "../keyboard";

  // Deliberately captures openId at mount — the editor lives for exactly one
  // document; navigation remounts it.
  let id = $state<string | null>(untrack(() => documents.openId));
  let title = $state("");
  let body = $state("");
  let loaded = $state(untrack(() => id === null));
  let dirty = $state(false);
  let saveTimer: ReturnType<typeof setTimeout> | undefined;
  let bodyEl = $state<HTMLTextAreaElement | undefined>(undefined);
  let saving: Promise<unknown> = Promise.resolve();

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

  const keymap: Keymap = {
    "!escape": () => back(),
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
    <textarea
      class="body"
      bind:this={bodyEl}
      bind:value={body}
      oninput={markDirty}
      spellcheck="false"
    ></textarea>
    <p class="hints">[esc] back{dirty ? " · ~" : ""}</p>
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
  .body {
    flex: 1;
    background: transparent;
    border: none;
    color: var(--fg);
    font: inherit;
    line-height: 1.5;
    resize: none;
    outline: none;
    user-select: text;
    cursor: text;
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
