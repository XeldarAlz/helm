<script lang="ts">
  import type { Snippet } from "svelte";
  import { fade } from "svelte/transition";
  import { fadeScale } from "$lib/animations";

  interface Props {
    open?: boolean;
    title?: string;
    size?: "sm" | "md" | "lg";
    closable?: boolean;
    onclose?: () => void;
    class?: string;
    children: Snippet;
    actions?: Snippet;
  }

  let {
    open = false,
    title,
    size = "md",
    closable = true,
    onclose,
    class: className = "",
    children,
    actions,
  }: Props = $props();

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape" && closable && onclose) onclose();
  }

  function handleBackdrop() {
    if (closable && onclose) onclose();
  }

  const sizeClasses: Record<string, string> = {
    sm: "max-w-sm",
    md: "max-w-md",
    lg: "max-w-lg",
  };
</script>

<svelte:window onkeydown={handleKeydown} />

{#if open}
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="fixed inset-0 z-50 flex items-center justify-center"
    transition:fade={{ duration: 200 }}
  >
    <!-- Backdrop -->
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <div
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      onclick={handleBackdrop}
    ></div>

    <!-- Content -->
    <div
      class="relative w-full {sizeClasses[size]} mx-4 bg-[var(--color-bg-surface)] border border-[var(--color-border-default)] rounded-[var(--radius-lg)] shadow-[var(--shadow-xl)] {className}"
      transition:fadeScale={{ duration: 200 }}
    >
      {#if title}
        <div class="flex items-center justify-between px-6 py-4 border-b border-[var(--color-border-subtle)]">
          <h2 class="text-[15px] font-semibold text-[var(--color-text-primary)]">{title}</h2>
          {#if closable}
            <button
              class="text-[var(--color-text-tertiary)] hover:text-[var(--color-text-primary)] transition-colors cursor-pointer"
              onclick={onclose}
            >
              <svg class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M18 6L6 18M6 6l12 12" />
              </svg>
            </button>
          {/if}
        </div>
      {/if}

      <div class="px-6 py-4">
        {@render children()}
      </div>

      {#if actions}
        <div class="flex items-center justify-end gap-3 px-6 py-4 border-t border-[var(--color-border-subtle)]">
          {@render actions()}
        </div>
      {/if}
    </div>
  </div>
{/if}
