<script lang="ts">
  import Unlock from "./lib/components/Unlock.svelte";
  import { session } from "./lib/stores/session.svelte";
</script>

{#if session.locked}
  <Unlock onunlocked={() => (session.locked = false)} />
{:else}
  <main class="shell">
    <p>unlocked · ready<span class="cursor">▊</span></p>
  </main>
{/if}

<style>
  .shell {
    display: flex;
    align-items: center;
    justify-content: center;
    height: 100vh;
  }
  .cursor {
    animation: blink 1s steps(1) infinite;
  }
  @keyframes blink {
    50% {
      opacity: 0;
    }
  }
</style>
