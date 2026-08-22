<main class="shell">
  <p>scratchpad<span class="cursor">▊</span></p>
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
