<script lang="ts">
  import type { TaskInfo } from "$lib/utils/ipc";
  import Badge from "$lib/components/common/Badge.svelte";

  interface Props {
    task: TaskInfo;
  }

  let { task }: Props = $props();

  const complexityColor = $derived(
    ({
      S: "var(--color-status-success)",
      M: "var(--color-status-warning)",
      L: "var(--color-status-error)",
    })[task.complexity] ?? "var(--color-text-secondary)",
  );
</script>

<div
  class="p-2.5 rounded-[var(--radius-md)] border border-[var(--color-border-subtle)] bg-[var(--color-bg-surface)]
    hover:bg-[var(--color-bg-elevated)] transition-colors duration-150"
>
  <!-- Title -->
  <div class="text-[var(--text-caption)] font-medium text-[var(--color-text-primary)] mb-1.5 line-clamp-2">
    {task.title}
  </div>

  <!-- Bottom row: ID, complexity, agent -->
  <div class="flex items-center gap-1.5">
    <span class="text-[9px] text-[var(--color-text-tertiary)] font-mono">{task.id}</span>

    <span
      class="inline-flex items-center justify-center w-4 h-4 rounded text-[9px] font-bold"
      style="background: color-mix(in srgb, {complexityColor} 15%, transparent); color: {complexityColor}"
    >
      {task.complexity}
    </span>

    {#if task.agent}
      <span class="text-[9px] text-[var(--color-text-secondary)] ml-auto truncate max-w-[80px]">
        {task.agent}
      </span>
    {/if}
  </div>
</div>
