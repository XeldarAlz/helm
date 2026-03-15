<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";

  let elapsed = $state(0);
  let intervalId: ReturnType<typeof setInterval>;

  onMount(() => {
    elapsed = 0;
    intervalId = setInterval(() => {
      elapsed += 1;
    }, 1000);
    return () => clearInterval(intervalId);
  });

  const formattedTime = $derived.by(() => {
    if (elapsed < 60) return `${elapsed}s`;
    const mins = Math.floor(elapsed / 60);
    const secs = elapsed % 60;
    return `${mins}m ${secs.toString().padStart(2, "0")}s`;
  });
</script>

<div class="typing-indicator flex items-center gap-2.5 px-4 py-3">
  <div class="dots flex items-center gap-1.5">
    <span class="dot"></span>
    <span class="dot"></span>
    <span class="dot"></span>
  </div>
  <span class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">
    {$_("chat.typing")}
  </span>
  <span class="elapsed text-[var(--text-caption)] text-[var(--color-text-tertiary)] tabular-nums">
    {formattedTime}
  </span>
</div>

<style>
  .typing-indicator {
    animation: fade-in 0.2s ease-out;
  }

  .dot {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background-color: var(--color-accent);
    animation: typing-dot 1.4s ease-in-out infinite;
  }

  .dot:nth-child(2) {
    animation-delay: 0.2s;
  }

  .dot:nth-child(3) {
    animation-delay: 0.4s;
  }

  .elapsed {
    font-variant-numeric: tabular-nums;
    min-width: 3ch;
  }

  @keyframes typing-dot {
    0%,
    60%,
    100% {
      opacity: 0.2;
      transform: scale(0.85) translateY(0);
    }
    30% {
      opacity: 1;
      transform: scale(1.3) translateY(-3px);
    }
  }

  @keyframes fade-in {
    from {
      opacity: 0;
      transform: translateY(4px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
