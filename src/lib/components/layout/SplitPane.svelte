<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    direction?: "horizontal" | "vertical";
    initialSplit?: number;
    minSize?: number;
    left: Snippet;
    right: Snippet;
    class?: string;
  }

  let {
    direction = "horizontal",
    initialSplit = 250,
    minSize = 150,
    left,
    right,
    class: className = "",
  }: Props = $props();

  let splitSize = $state(initialSplit);
  let dragging = $state(false);

  function startDrag(e: MouseEvent) {
    dragging = true;
    const startPos = direction === "horizontal" ? e.clientX : e.clientY;
    const startSize = splitSize;

    function onMove(ev: MouseEvent) {
      const delta =
        (direction === "horizontal" ? ev.clientX : ev.clientY) - startPos;
      splitSize = Math.max(minSize, startSize + delta);
    }

    function onUp() {
      dragging = false;
      window.removeEventListener("mousemove", onMove);
      window.removeEventListener("mouseup", onUp);
    }

    window.addEventListener("mousemove", onMove);
    window.addEventListener("mouseup", onUp);
  }
</script>

<div
  class="flex {direction === 'vertical' ? 'flex-col' : 'flex-row'} h-full {className}"
>
  <div
    class="overflow-auto shrink-0"
    style="{direction === 'horizontal' ? 'width' : 'height'}: {splitSize}px"
  >
    {@render left()}
  </div>

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="shrink-0 {direction === 'horizontal' ? 'w-1 cursor-col-resize' : 'h-1 cursor-row-resize'}
      bg-[var(--color-border-subtle)] hover:bg-[var(--color-accent)] transition-colors duration-150
      {dragging ? 'bg-[var(--color-accent)]' : ''}"
    onmousedown={startDrag}
  ></div>

  <div class="flex-1 overflow-auto">
    {@render right()}
  </div>
</div>
