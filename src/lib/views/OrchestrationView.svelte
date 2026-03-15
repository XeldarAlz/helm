<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { staggeredItem, fadeScale } from "$lib/animations";
  import { getOrchestrationState, startWatching } from "$lib/utils/ipc";
  import {
    orchestrationState,
    orchStatus,
    orchPhases,
    orchAgents,
    orchTasks,
    orchHooks,
    filteredLog,
    orchProgress,
    isOrchestrating,
    startOrchestrationListeners,
    stopOrchestrationListeners,
  } from "$lib/stores/agents";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Panel from "$lib/components/layout/Panel.svelte";
  import PhaseProgressBar from "$lib/components/orchestration/PhaseProgressBar.svelte";
  import AgentGrid from "$lib/components/orchestration/AgentGrid.svelte";
  import TaskKanban from "$lib/components/orchestration/TaskKanban.svelte";
  import LiveLog from "$lib/components/orchestration/LiveLog.svelte";
  import HookMonitor from "$lib/components/orchestration/HookMonitor.svelte";
  import OrchControls from "$lib/components/orchestration/OrchControls.svelte";
  import { Activity, Users, ListTodo, Terminal, Shield, Loader2 } from "lucide-svelte";
  import { listSessions } from "$lib/utils/ipc";

  let mounted = $state(false);
  let hasActiveOrchSession = $state(false);
  let progressListener: UnlistenFn | undefined;
  let pollInterval: ReturnType<typeof setInterval> | undefined;

  async function loadState() {
    try {
      const state = await getOrchestrationState();
      orchestrationState.set(state);
    } catch {
      // PROGRESS.md may not exist yet — that's fine
    }
  }

  async function checkActiveOrchSession() {
    try {
      const sessions = await listSessions();
      hasActiveOrchSession = sessions.some(
        (s) => s.phase === "orchestrate" && s.status === "active",
      );
    } catch {
      // ignore
    }
  }

  onMount(async () => {
    mounted = true;
    await loadState();
    await checkActiveOrchSession();
    await startOrchestrationListeners();

    // Ensure file watcher is running so PROGRESS.md changes emit events
    startWatching().catch(() => {});

    // Re-load state when PROGRESS.md changes
    progressListener = await listen("progress-updated", () => {
      loadState();
    });

    // Poll every 5s (always, not just while orchestrating) so we can
    // detect when PROGRESS.md first appears or status transitions to running
    pollInterval = setInterval(() => {
      loadState();
      checkActiveOrchSession();
    }, 5000);
  });

  onDestroy(() => {
    progressListener?.();
    stopOrchestrationListeners();
    if (pollInterval) clearInterval(pollInterval);
  });

  // Section visibility for the grid layout
  const hasData = $derived($orchPhases.length > 0 || $orchAgents.length > 0 || $orchTasks.length > 0);
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("orchestration.title")}>
    {#snippet actions()}
      <OrchControls status={$orchStatus} />
    {/snippet}
  </TopBar>

  <div class="flex-1 overflow-y-auto min-h-0">
    {#if !hasData && $orchStatus === "idle" && hasActiveOrchSession}
      <!-- Initializing state: session is running but PROGRESS.md not yet written -->
      <div class="flex flex-col items-center justify-center h-full gap-4 p-8">
        {#if mounted}
          <div in:fadeScale={{ duration: 300 }}>
            <div
              class="flex items-center justify-center w-16 h-16 rounded-2xl mb-2
                bg-[var(--color-accent)]/10 text-[var(--color-accent)]"
            >
              <Loader2 size={32} class="animate-spin" />
            </div>
          </div>
          <div class="text-center" in:fadeScale={{ duration: 300, delay: 100 }}>
            <h2 class="text-[var(--text-title)] font-semibold text-[var(--color-text-primary)] mb-1">
              Initializing Orchestration
            </h2>
            <p class="text-[var(--text-body)] text-[var(--color-text-secondary)] max-w-md">
              An orchestration session is active. Waiting for the agent to begin writing progress data...
            </p>
          </div>
        {/if}
      </div>
    {:else if !hasData && $orchStatus === "idle"}
      <!-- Empty state -->
      <div class="flex flex-col items-center justify-center h-full gap-4 p-8">
        {#if mounted}
          <div in:fadeScale={{ duration: 300 }}>
            <div
              class="flex items-center justify-center w-16 h-16 rounded-2xl mb-2
                bg-[var(--color-accent)]/10 text-[var(--color-accent)]"
            >
              <Activity size={32} />
            </div>
          </div>
          <div class="text-center" in:fadeScale={{ duration: 300, delay: 100 }}>
            <h2 class="text-[var(--text-title)] font-semibold text-[var(--color-text-primary)] mb-1">
              {$_("orchestration.title")}
            </h2>
            <p class="text-[var(--text-body)] text-[var(--color-text-secondary)] max-w-md">
              Start an orchestration session from the Dashboard to monitor multi-agent execution in real time.
            </p>
          </div>
        {/if}
      </div>
    {:else}
      <!-- Main orchestration dashboard -->
      <div class="p-6 space-y-5">
        <!-- Phase Progress -->
        {#if mounted && $orchPhases.length > 0}
          <Panel class="p-5">
            <div in:staggeredItem={{ index: 0, staggerDelay: 60, duration: 250, distance: 10 }}>
              <PhaseProgressBar
                phases={$orchPhases}
                currentPhase={$orchestrationState.current_phase}
                totalPhases={$orchestrationState.total_phases}
                phaseName={$orchestrationState.phase_name}
              />
            </div>
          </Panel>
        {/if}

        <!-- Middle row: Agents + Hooks side by side -->
        <div class="grid grid-cols-1 xl:grid-cols-3 gap-5">
          <!-- Agent Grid (2/3) -->
          {#if mounted}
            <div class="xl:col-span-2" in:staggeredItem={{ index: 1, staggerDelay: 60, duration: 250, distance: 10 }}>
              <div class="flex items-center gap-2 mb-3">
                <Users size={14} class="text-[var(--color-text-secondary)]" />
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  {$_("orchestration.agents.idle").replace("Idle", "Agents")}
                </h3>
                {#if $orchAgents.length > 0}
                  <span class="text-[10px] text-[var(--color-text-tertiary)]">
                    {$orchAgents.filter((a) => a.status === "running").length} active
                  </span>
                {/if}
              </div>
              <AgentGrid agents={$orchAgents} />
            </div>
          {/if}

          <!-- Hook Monitor (1/3) -->
          {#if mounted}
            <div in:staggeredItem={{ index: 2, staggerDelay: 60, duration: 250, distance: 10 }}>
              <div class="flex items-center gap-2 mb-3">
                <Shield size={14} class="text-[var(--color-text-secondary)]" />
                <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                  Hooks
                </h3>
              </div>
              <HookMonitor hooks={$orchHooks} />
            </div>
          {/if}
        </div>

        <!-- Task Kanban -->
        {#if mounted && $orchTasks.length > 0}
          <div in:staggeredItem={{ index: 3, staggerDelay: 60, duration: 250, distance: 10 }}>
            <div class="flex items-center gap-2 mb-3">
              <ListTodo size={14} class="text-[var(--color-text-secondary)]" />
              <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                Tasks
              </h3>
              <span class="text-[10px] text-[var(--color-text-tertiary)]">
                {$orchTasks.filter((t) => t.status === "done").length}/{$orchTasks.length} complete
              </span>
            </div>
            <Panel class="p-4">
              <div class="h-[240px]">
                <TaskKanban tasks={$orchTasks} />
              </div>
            </Panel>
          </div>
        {/if}

        <!-- Live Log -->
        {#if mounted}
          <div in:staggeredItem={{ index: 4, staggerDelay: 60, duration: 250, distance: 10 }}>
            <div class="flex items-center gap-2 mb-3">
              <Terminal size={14} class="text-[var(--color-text-secondary)]" />
            </div>
            <Panel class="p-4">
              <div class="h-[260px]">
                <LiveLog entries={$filteredLog} />
              </div>
            </Panel>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
