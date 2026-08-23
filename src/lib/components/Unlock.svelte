<script lang="ts">
  import { unlock, startFresh, errorKind, type CoreError } from "../api";

  let { onunlocked }: { onunlocked: () => void } = $props();

  // Built programmatically so the borders always match the label width.
  const LABEL = "  s c r a t c h p a d  ";
  let label = $state(LABEL);
  const logo = $derived(
    [
      `┌${"─".repeat(LABEL.length)}┐`,
      `│${label}│`,
      `└${"─".repeat(LABEL.length)}┘`,
    ].join("\n"),
  );

  // Periodic "decode" scramble: letters churn through random glyphs and
  // resolve left-to-right. Spaces are never touched, so the width (and the
  // box) stays fixed.
  const POOL = "abcdefghijklmnopqrstuvwxyz#*+=<>?/\\|~^%$&@!";
  const LETTER_IDXS = [...LABEL].map((c, i) => (c === " " ? -1 : i)).filter((i) => i >= 0);

  function scrambleOnce(): () => void {
    // Full-churn hold first, then resolve left-to-right.
    const HOLD_FRAMES = 22;
    const RESOLVE_FRAMES = 16;
    const FRAMES = HOLD_FRAMES + RESOLVE_FRAMES;
    let frame = 0;
    const timer = setInterval(() => {
      frame++;
      const progress = Math.max(0, frame - HOLD_FRAMES) / RESOLVE_FRAMES;
      label = [...LABEL]
        .map((c, i) => {
          if (c === " ") return " ";
          const resolved = LETTER_IDXS.indexOf(i) / LETTER_IDXS.length < progress;
          return resolved ? c : POOL[Math.floor(Math.random() * POOL.length)];
        })
        .join("");
      if (frame >= FRAMES) {
        clearInterval(timer);
        label = LABEL;
      }
    }, 45);
    return () => clearInterval(timer);
  }

  $effect(() => {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) return;
    let stopRun: (() => void) | undefined;
    const first = setTimeout(() => (stopRun = scrambleOnce()), 900);
    const every = setInterval(() => (stopRun = scrambleOnce()), 4200);
    return () => {
      clearTimeout(first);
      clearInterval(every);
      stopRun?.();
      label = LABEL;
    };
  });

  type Status = "authenticating" | "done" | "denied" | "corrupt" | "error";
  let status = $state<Status>("authenticating");
  let detail = $state("");
  let confirmText = $state("");
  let confirmInput = $state<HTMLInputElement | undefined>(undefined);

  /** Brief "typed" reveal before handing over to the app. */
  function finish() {
    if (window.matchMedia("(prefers-reduced-motion: reduce)").matches) {
      onunlocked();
      return;
    }
    status = "done";
    setTimeout(onunlocked, 420);
  }

  async function attempt() {
    status = "authenticating";
    try {
      await unlock();
      finish();
    } catch (e) {
      const kind = errorKind(e);
      if (kind === "corrupt") {
        status = "corrupt";
        confirmText = "";
      } else if (kind === "keychainDenied") {
        status = "denied";
      } else {
        status = "error";
        detail =
          kind === "io" ? ((e as CoreError & { detail: string }).detail ?? "") : kind;
      }
    }
  }

  async function eraseAndStartFresh() {
    if (confirmText !== "ERASE") return;
    status = "authenticating";
    try {
      await startFresh();
      finish();
    } catch (e) {
      status = "error";
      detail = errorKind(e);
    }
  }

  function onkeydown(e: KeyboardEvent) {
    if ((status === "denied" || status === "error") && e.key === "r") {
      e.preventDefault();
      attempt();
    }
  }

  $effect(() => {
    attempt();
  });

  $effect(() => {
    if (status === "corrupt") confirmInput?.focus();
  });
</script>

<svelte:window {onkeydown} />

<main class="unlock">
  <pre class="logo">{logo}</pre>

  {#if status === "authenticating"}
    <p class="line dim">authenticating …</p>
  {:else if status === "done"}
    <p class="line typed ok">▸ unlocked</p>
  {:else if status === "denied"}
    <p class="line warn">✗ authentication cancelled</p>
    <p class="line dim">[r] retry</p>
  {:else if status === "corrupt"}
    <p class="line err">✗ stored data cannot be decrypted with the current key</p>
    <p class="line dim">
      the keychain entry was likely deleted or replaced. the old database will be
      quarantined (renamed, not deleted).
    </p>
    <p class="line">
      type <strong>ERASE</strong> + ⏎ to start fresh:
      <input
        bind:this={confirmInput}
        bind:value={confirmText}
        onkeydown={(e) => e.key === "Enter" && eraseAndStartFresh()}
        spellcheck="false"
        autocomplete="off"
      />
    </p>
  {:else}
    <p class="line err">✗ unlock failed{detail ? `: ${detail}` : ""}</p>
    <p class="line dim">[r] retry</p>
  {/if}
</main>

<style>
  .unlock {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100vh;
    gap: 0.75rem;
  }
  .logo {
    color: var(--fg);
    margin-bottom: 1rem;
    /* Box-drawing glyphs only connect vertically at line-height 1. */
    line-height: 1;
  }
  .line {
    max-width: 34rem;
    text-align: center;
  }
  .dim {
    opacity: 0.55;
  }
  .warn {
    color: var(--warn);
  }
  .ok {
    color: var(--ok);
  }
  /* Monospace makes a steps() width animation read as typing. */
  .typed {
    overflow: hidden;
    white-space: nowrap;
    width: 10ch;
    animation: typing 0.32s steps(10);
  }
  @keyframes typing {
    from {
      width: 0;
    }
    to {
      width: 10ch;
    }
  }
  .err {
    color: var(--err);
  }
  input {
    background: transparent;
    border: none;
    border-bottom: 0.0625rem solid var(--fg);
    color: var(--fg);
    font: inherit;
    width: 6ch;
    outline: none;
    user-select: text;
  }
</style>
