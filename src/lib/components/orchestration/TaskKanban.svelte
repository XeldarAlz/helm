<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { TaskInfo } from "$lib/utils/ipc";
  import TaskCard from "./TaskCard.svelte";

  interface Props {
    tasks: TaskInfo[];
  }

  let { tasks }: Props = $props();

  type ColumnKey = "pending" | "working" | "review" | "done";

  const columns: { key: ColumnKey; color: string }[] = [
    { key: "pending", color: "var(--color-status-pending)" },
    { key: "working", color: "var(--color-status-info)" },
    { key: "review", color: "var(--color-status-warning)" },
    { key: "done", color: "var(--color-status-success)" },
  ];

  const grouped = $derived(
    columns.map((col) => ({
      ...col,
      tasks: tasks.filter((t) => {
        if (col.key === "done") return t.status === "done" || t.status === "failed";
        return t.status === col.key;
      }),
    })),
  );
</script>

<div class="grid grid-cols-4 gap-3 h-full min-h-0">
  {#each grouped as column}
    <div class="flex flex-col min-h-0">
      <!-- Column header -->
      <div class="flex items-center gap-2 mb-2.5 px-1">
        <div class="w-2 h-2 rounded-full" style="background: {column.color}"></div>
        <span class="text-[var(--text-caption)] font-semibold text-[var(--color-text-primary)]">
          {$_(`orchestration.tasks.${column.key}`)}
        </span>
        <span class="text-[10px] text-[var(--color-text-tertiary)] ml-auto">
          {column.tasks.length}
        </span>
      </div>

      <!-- Task list -->
      <div class="flex-1 overflow-y-auto space-y-2 pr-0.5 min-h-0">
        {#each column.tasks as task (task.id)}
          <TaskCard {task} />
        {/each}
        {#if column.tasks.length === 0}
          <div class="h-12 rounded-[var(--radius-md)] border border-dashed border-[var(--color-border-subtle)] flex items-center justify-center">
            <span class="text-[10px] text-[var(--color-text-tertiary)]">Empty</span>
          </div>
        {/if}
      </div>
    </div>
  {/each}
</div>
