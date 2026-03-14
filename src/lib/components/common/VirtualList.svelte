<script lang="ts">
  import type { Snippet } from "svelte";
  import { tick } from "svelte";

  interface Props {
    items: any[];
    itemHeight: number;
    overscan?: number;
    class?: string;
    children: Snippet<[{ item: any; index: number }]>;
  }

  let {
    items,
    itemHeight,
    overscan = 5,
    class: className = "",
    children,
  }: Props = $props();

  let container: HTMLDivElement | undefined = $state();
  let scrollTop = $state(0);
  let containerHeight = $state(0);

  const totalHeight = $derived(items.length * itemHeight);

  const startIndex = $derived(
    Math.max(0, Math.floor(scrollTop / itemHeight) - overscan)
  );

  const endIndex = $derived(
    Math.min(items.length, Math.ceil((scrollTop + containerHeight) / itemHeight) + overscan)
  );

  const visibleItems = $derived(
    items.slice(startIndex, endIndex).map((item, i) => ({
      item,
      index: startIndex + i,
    }))
  );

  const offsetY = $derived(startIndex * itemHeight);

  function handleScroll() {
    if (container) {
      scrollTop = container.scrollTop;
      containerHeight = container.clientHeight;
    }
  }

  $effect(() => {
    if (container) {
      containerHeight = container.clientHeight;
    }
  });
</script>

<div
  bind:this={container}
  onscroll={handleScroll}
  class="overflow-y-auto {className}"
>
  <div style="height: {totalHeight}px; position: relative;">
    <div style="transform: translateY({offsetY}px);">
      {#each visibleItems as entry (entry.index)}
        {@render children(entry)}
      {/each}
    </div>
  </div>
</div>
