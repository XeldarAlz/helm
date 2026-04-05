<script lang="ts">
  import { _ } from "svelte-i18n";
  import { activeView } from "$lib/stores/ui";
  import { currentSession } from "$lib/stores/session";
  import { phases, nextPhase, phaseArtifact, type PipelinePhase } from "$lib/stores/pipeline";
  import { createSession, endSession } from "$lib/utils/ipc";
  import { messages, suggestedAnswers } from "$lib/stores/session";
  import { addToast } from "$lib/stores/toasts";
  import Spinner from "$lib/components/common/Spinner.svelte";
  import {
    Check,
    Lightbulb,
    Compass,
    CalendarClock,
    Hammer,
    PartyPopper,
    ArrowRight,
    Play,
  } from "lucide-svelte";

  let startingPhase = $state<string | null>(null);

  const phaseIcons: Record<string, typeof Check> = {
    game_idea: Lightbulb,
    architect: Compass,
    plan_workflow: CalendarClock,
    orchestrate: Hammer,
    complete: PartyPopper,
  };

  function handlePhaseClick(phase: PipelinePhase) {
    if (phase.status === "locked") return;

    if (phase.status === "done") {
      // Navigate to documents view to review the artifact
      activeView.set("documents");
      return;
    }

    if (phase.status === "active") {
      // Go to the active chat session
      activeView.set("chat");
      return;
    }

    if (phase.status === "ready") {
      startPhase(phase.id);
    }
  }

  async function startPhase(phaseId: string) {
    if (startingPhase) return;
    startingPhase = phaseId;

    try {
      // End existing session if active
      if ($currentSession?.status === "active") {
        await endSession($currentSession.id);
      }

      // Clear frontend state
      messages.set([]);
      suggestedAnswers.set([]);

      const sessionId = await createSession(phaseId);
      currentSession.set({
        id: sessionId,
        phase: phaseId,
        status: "active",
        startedAt: new Date().toISOString(),
        messageCount: 0,
        contextUsage: 0,
      });
      activeView.set("chat");
    } catch (e) {
      addToast(`Failed to start phase: ${e}`, "error");
    } finally {
      startingPhase = null;
    }
  }

  function statusColor(status: PipelinePhase["status"]): string {
    switch (status) {
      case "done": return "var(--color-status-success)";
      case "active": return "var(--color-accent)";
      case "ready": return "var(--color-status-warning)";
      case "locked": return "var(--color-text-tertiary)";
    }
  }
</script>

<div class="flex items-center px-4 py-2 bg-[var(--color-bg-surface)] border-b border-[var(--color-border-subtle)] gap-0 shrink-0">
  {#each $phases as phase, i}
    {@const Icon = phaseIcons[phase.id] ?? Check}
    {@const isLast = i === $phases.length - 1}
    {@const color = statusColor(phase.status)}
    {@const clickable = phase.status !== "locked"}
    {@const isNext = $nextPhase?.id === phase.id}

    <!-- Phase node -->
    <button
      class="group flex items-center gap-2 px-3 py-1.5 rounded-[var(--radius-md)] transition-all duration-200
        {clickable ? 'cursor-pointer' : 'cursor-default opacity-40'}
        {phase.status === 'active' ? 'bg-[var(--color-accent-muted)]' : ''}
        {clickable && phase.status !== 'active' ? 'hover:bg-[var(--color-bg-elevated)]' : ''}"
      onclick={() => handlePhaseClick(phase)}
      disabled={phase.status === "locked"}
    >
      <!-- Status circle -->
      <div
        class="relative flex items-center justify-center w-6 h-6 rounded-full border-2 transition-all duration-300 shrink-0"
        style="border-color: {color}; {phase.status === 'done' ? `background: ${color}` : ''}"
      >
        {#if phase.status === "done"}
          <Check size={12} class="text-[var(--color-text-inverse)]" />
        {:else if startingPhase === phase.id}
          <Spinner size="sm" />
        {:else if phase.status === "active"}
          <div class="w-2 h-2 rounded-full animate-pulse" style="background: {color}"></div>
        {:else}
          <Icon size={10} style="color: {color}" />
        {/if}

        <!-- Active glow -->
        {#if phase.status === "active"}
          <div class="absolute inset-0 rounded-full animate-pulse" style="box-shadow: 0 0 8px {color}40"></div>
        {/if}
      </div>

      <!-- Label -->
      <span
        class="text-[12px] font-medium transition-colors duration-200 whitespace-nowrap"
        style="color: {phase.status === 'locked' ? 'var(--color-text-tertiary)' : 'var(--color-text-primary)'}"
      >
        {phase.label}
      </span>

      <!-- "Start" badge for ready phase -->
      {#if isNext && !startingPhase}
        <button
          class="flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-semibold cursor-pointer
            bg-[var(--color-accent)] text-[var(--color-text-inverse)]
            hover:brightness-110 active:scale-95 transition-all duration-150"
          onclick={(e: MouseEvent) => { e.stopPropagation(); startPhase(phase.id); }}
        >
          <Play size={8} />
          Start
        </button>
      {/if}
    </button>

    <!-- Connector -->
    {#if !isLast}
      <div class="flex-1 flex items-center px-1 min-w-[16px]">
        <div
          class="w-full h-[2px] rounded-full transition-colors duration-500"
          style="background: {phase.status === 'done' ? 'var(--color-status-success)' : 'var(--color-border-subtle)'}"
        ></div>
        {#if phase.status === "done" && $phases[i + 1]?.status === "ready"}
          <ArrowRight size={12} class="text-[var(--color-status-warning)] shrink-0 -ml-1" />
        {/if}
      </div>
    {/if}
  {/each}
</div>
