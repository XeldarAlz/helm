<script lang="ts">
  import { _ } from "svelte-i18n";
  import { AlertTriangle, RefreshCw } from "lucide-svelte";

  interface Props {
    title?: string;
    message?: string;
    onretry?: () => void;
    class?: string;
  }

  let {
    title = "Something went wrong",
    message = "",
    onretry,
    class: className = "",
  }: Props = $props();
</script>

<div class="flex items-center justify-center p-8 {className}">
  <div class="text-center max-w-sm">
    <div class="flex items-center justify-center w-12 h-12 mx-auto mb-3 rounded-[var(--radius-lg)] bg-[var(--color-status-error)]/10">
      <AlertTriangle size={24} class="text-[var(--color-status-error)]" />
    </div>
    <h3 class="text-[var(--text-subtitle)] font-semibold text-[var(--color-text-primary)] mb-1">
      {title}
    </h3>
    {#if message}
      <p class="text-[var(--text-caption)] text-[var(--color-text-secondary)] mb-3 break-words">
        {message}
      </p>
    {/if}
    {#if onretry}
      <button
        class="inline-flex items-center gap-2 px-3 py-1.5 text-[var(--text-caption)] font-medium
          text-[var(--color-accent)] hover:bg-[var(--color-accent)]/10
          rounded-[var(--radius-md)] transition-colors duration-150 cursor-pointer"
        onclick={onretry}
      >
        <RefreshCw size={13} />
        {$_("common.retry")}
      </button>
    {/if}
  </div>
</div>
