<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title: string;
    description?: string;
    icon?: Snippet;
    action?: Snippet;
    class?: string;
  }

  let {
    title,
    description = "",
    icon,
    action,
    class: className = "",
  }: Props = $props();
</script>

<div class="flex items-center justify-center p-8 {className}">
  <div class="text-center max-w-sm">
    {#if icon}
      <div class="mb-3 text-[var(--color-text-tertiary)] flex justify-center">
        {@render icon()}
      </div>
    {/if}
    <h3 class="text-[var(--text-subtitle)] font-semibold text-[var(--color-text-primary)] mb-1">
      {title}
    </h3>
    {#if description}
      <p class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">
        {description}
      </p>
    {/if}
    {#if action}
      <div class="mt-3">
        {@render action()}
      </div>
    {/if}
  </div>
</div>
