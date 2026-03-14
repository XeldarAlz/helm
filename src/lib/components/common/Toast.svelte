<script lang="ts">
  import { slide } from "$lib/animations";

  interface Props {
    variant?: "info" | "success" | "warning" | "error";
    message: string;
    action?: { label: string; onclick: () => void };
    ondismiss?: () => void;
  }

  let {
    variant = "info",
    message,
    action,
    ondismiss,
  }: Props = $props();

  const variantClasses: Record<string, string> = {
    info: "border-l-[var(--color-status-info)]",
    success: "border-l-[var(--color-status-success)]",
    warning: "border-l-[var(--color-status-warning)]",
    error: "border-l-[var(--color-status-error)]",
  };

  const iconPaths: Record<string, string> = {
    info: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
    success: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
    warning: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.34 16.5c-.77.833.192 2.5 1.732 2.5z",
    error: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
  };
</script>

<div
  class="flex items-center gap-3 px-4 py-3 bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)] border-l-4 {variantClasses[variant]} rounded-[var(--radius-md)] shadow-[var(--shadow-lg)]"
  transition:slide={{ direction: "right", duration: 200 }}
>
  <svg class="w-5 h-5 shrink-0" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
    <path d={iconPaths[variant]} />
  </svg>

  <span class="flex-1 text-[13px] text-[var(--color-text-primary)]">{message}</span>

  {#if action}
    <button
      class="text-[12px] font-medium text-[var(--color-accent)] hover:text-[var(--color-accent-hover)] cursor-pointer"
      onclick={action.onclick}
    >
      {action.label}
    </button>
  {/if}

  {#if ondismiss}
    <button
      class="text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)] cursor-pointer"
      onclick={ondismiss}
      aria-label="Dismiss"
    >
      <svg class="w-4 h-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M18 6L6 18M6 6l12 12" />
      </svg>
    </button>
  {/if}
</div>
