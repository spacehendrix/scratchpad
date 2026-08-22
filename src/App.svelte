<script lang="ts">
  import { isUnlocked } from "./lib/api";

  let locked = $state<boolean | undefined>(undefined);

  $effect(() => {
    isUnlocked().then((v) => (locked = !v));
  });
</script>

<main class="shell">
  <p>
    scratchpad
    {#if locked === undefined}
      · …
    {:else}
      · {locked ? "locked" : "unlocked"}
    {/if}
    <span class="cursor">▊</span>
  </p>
</main>

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
