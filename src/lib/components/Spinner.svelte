<script lang="ts">
  import { ticker } from "../stores/ticker.svelte";

  const FRAMES = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
</script>

<span class="spinner">{FRAMES[ticker.frame % FRAMES.length]}</span>

<style>
  .spinner {
    color: var(--accent);
  }
</style>
