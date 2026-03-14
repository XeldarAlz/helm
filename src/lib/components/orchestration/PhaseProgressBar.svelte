<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { PhaseInfo } from "$lib/utils/ipc";
  import { Check, Circle, Loader2 } from "lucide-svelte";

  interface Props {
    phases: PhaseInfo[];
    currentPhase: number;
    totalPhases: number;
    phaseName: string;
  }

  let { phases, currentPhase, totalPhases, phaseName }: Props = $props();

  const progress = $derived(
    totalPhases > 0
      ? Math.round((phases.filter((p) => p.status === "done").length / totalPhases) * 100)
      : 0,
  );
</script>

<div class="space-y-4">
  <!-- Header with phase counter -->
  <div class="flex items-center justify-between">
    <div class="flex items-center gap-3">
      <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
        {$_("orchestration.phase", { values: { current: currentPhase, total: totalPhases } })}
      </h3>
      {#if phaseName}
        <span class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">
          {phaseName}
        </span>
      {/if}
    </div>
    <span class="text-[var(--text-caption)] font-medium text-[var(--color-accent)]">
      {progress}%
    </span>
  </div>

  <!-- Phase pills -->
  <div class="flex items-center gap-1">
    {#each phases as phase, i}
      <div class="flex items-center flex-1" title="{phase.name}">
        <!-- Phase segment -->
        <div
          class="h-2 w-full rounded-full transition-all duration-500 relative overflow-hidden"
          class:bg-[var(--color-status-success)]={phase.status === "done"}
          class:bg-[var(--color-accent)]={phase.status === "active"}
          class:bg-[var(--color-bg-overlay)]={phase.status === "pending"}
        >
          {#if phase.status === "active"}
            <div
              class="absolute inset-0 bg-gradient-to-r from-transparent via-white/20 to-transparent animate-[shimmer-slide_2s_ease-in-out_infinite]"
            ></div>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <!-- Phase labels (compact) -->
  <div class="flex gap-1">
    {#each phases as phase}
      <div class="flex-1 flex items-center gap-1 min-w-0">
        {#if phase.status === "done"}
          <Check size={10} class="text-[var(--color-status-success)] shrink-0" />
        {:else if phase.status === "active"}
          <Loader2 size={10} class="text-[var(--color-accent)] shrink-0 animate-spin" />
        {:else}
          <Circle size={8} class="text-[var(--color-text-tertiary)] shrink-0" />
        {/if}
        <span
          class="text-[9px] truncate leading-none"
          class:text-[var(--color-text-primary)]={phase.status === "done" || phase.status === "active"}
          class:font-medium={phase.status === "active"}
          class:text-[var(--color-text-tertiary)]={phase.status === "pending"}
        >
          {phase.name}
        </span>
      </div>
    {/each}
  </div>
</div>
