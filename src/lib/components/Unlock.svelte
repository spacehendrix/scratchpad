<script lang="ts">
  import { unlock, startFresh, errorKind, type CoreError } from "../api";

  let { onunlocked }: { onunlocked: () => void } = $props();

  type Status = "authenticating" | "denied" | "corrupt" | "error";
  let status = $state<Status>("authenticating");
  let detail = $state("");
  let confirmText = $state("");
  let confirmInput = $state<HTMLInputElement | undefined>(undefined);

  async function attempt() {
    status = "authenticating";
    try {
      await unlock();
      onunlocked();
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
      onunlocked();
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
  <pre class="logo">{`┌─────────────────────┐
│  s c r a t c h p a d │
└─────────────────────┘`}</pre>

  {#if status === "authenticating"}
    <p class="line dim">authenticating …</p>
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
  }
  .line {
    max-width: 34rem;
    text-align: center;
  }
  .dim {
    opacity: 0.55;
  }
  .warn {
    color: #e0af68;
  }
  .err {
    color: #f7768e;
  }
  input {
    background: transparent;
    border: none;
    border-bottom: 1px solid var(--fg);
    color: var(--fg);
    font: inherit;
    width: 6ch;
    outline: none;
    user-select: text;
  }
</style>
