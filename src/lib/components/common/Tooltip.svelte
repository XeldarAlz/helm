<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    text: string;
    position?: "top" | "bottom" | "left" | "right";
    delay?: number;
    class?: string;
    children: Snippet;
  }

  let {
    text,
    position = "top",
    delay = 500,
    class: className = "",
    children,
  }: Props = $props();

  let visible = $state(false);
  let timeout: ReturnType<typeof setTimeout>;

  function show() {
    timeout = setTimeout(() => (visible = true), delay);
  }

  function hide() {
    clearTimeout(timeout);
    visible = false;
  }

  const positionClasses: Record<string, string> = {
    top: "bottom-full left-1/2 -translate-x-1/2 mb-2",
    bottom: "top-full left-1/2 -translate-x-1/2 mt-2",
    left: "right-full top-1/2 -translate-y-1/2 mr-2",
    right: "left-full top-1/2 -translate-y-1/2 ml-2",
  };
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="relative inline-flex {className}"
  onmouseenter={show}
  onmouseleave={hide}
>
  {@render children()}
  {#if visible && text}
    <div
      class="absolute z-50 px-2 py-1 text-[11px] font-medium whitespace-nowrap rounded-[var(--radius-sm)] bg-[var(--color-bg-elevated)] text-[var(--color-text-primary)] border border-[var(--color-border-default)] shadow-[var(--shadow-md)] pointer-events-none {positionClasses[position]}"
    >
      {text}
    </div>
  {/if}
</div>
