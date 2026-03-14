<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    title?: string;
    children?: Snippet;
    actions?: Snippet;
  }

  let { title, children, actions }: Props = $props();
</script>

<header
  class="flex items-center justify-between h-12 px-6 border-b border-[var(--color-border-subtle)] bg-[var(--color-bg-base)]"
  data-tauri-drag-region
>
  <div class="flex items-center gap-3">
    {#if title}
      <h1 class="text-[15px] font-semibold text-[var(--color-text-primary)]">{title}</h1>
    {/if}
    {#if children}
      {@render children()}
    {/if}
  </div>

  {#if actions}
    <div class="flex items-center gap-2">
      {@render actions()}
    </div>
  {/if}
</header>
