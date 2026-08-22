<script lang="ts">
  import Unlock from "./lib/components/Unlock.svelte";
  import Browse from "./lib/components/Browse.svelte";
  import Editor from "./lib/components/Editor.svelte";
  import { session } from "./lib/stores/session.svelte";
</script>

{#if session.locked}
  <Unlock onunlocked={() => (session.locked = false)} />
{:else if session.view === "editor"}
  <Editor />
{:else}
  <Browse />
{/if}
