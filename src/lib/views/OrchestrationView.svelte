<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount, onDestroy } from "svelte";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { fadeScale } from "$lib/animations";
  import { getOrchestrationState, startWatching, listSessions } from "$lib/utils/ipc";
  import {
    orchestrationState,
    orchStatus,
    orchPhases,
    orchAgents,
    orchTasks,
    orchHooks,
    filteredLog,
    orchProgress,
    startOrchestrationListeners,
    stopOrchestrationListeners,
  } from "$lib/stores/agents";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Panel from "$lib/components/layout/Panel.svelte";
  import PhaseProgressBar from "$lib/components/orchestration/PhaseProgressBar.svelte";
  import Timeline from "$lib/components/orchestration/Timeline.svelte";
  import OrchControls from "$lib/components/orchestration/OrchControls.svelte";
  import HookMonitor from "$lib/components/orchestration/HookMonitor.svelte";
  import {
    Activity,
    Loader2,
    Clock,
    ListTodo,
    Users,
    Shield,
    Code,
    TestTube2,
    Eye,
    Box,
    GitCommitHorizontal,
    Zap,
    CheckCircle2,
    Circle,
  } from "lucide-svelte";

  let mounted = $state(false);
  let hasActiveOrchSession = $state(false);
  let progressListener: UnlistenFn | undefined;
  let pollInterval: ReturnType<typeof setInterval> | undefined;
  let now = $state(Date.now());
  let tickInterval: ReturnType<typeof setInterval> | undefined;

  async function loadState() {
    try {
      const state = await getOrchestrationState();
      orchestrationState.set(state);
    } catch {
      // PROGRESS.md may not exist yet
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

    startWatching().catch(() => {});

    progressListener = await listen("progress-updated", () => {
      loadState();
    });

    pollInterval = setInterval(() => {
      loadState();
      checkActiveOrchSession();
    }, 5000);

    tickInterval = setInterval(() => {
      now = Date.now();
    }, 1000);
  });

  onDestroy(() => {
    progressListener?.();
    stopOrchestrationListeners();
    if (pollInterval) clearInterval(pollInterval);
    if (tickInterval) clearInterval(tickInterval);
  });

  const hasData = $derived(
    $orchPhases.length > 0 || $orchAgents.length > 0 || $orchTasks.length > 0,
  );

  // ── Derived stats ─────────────────────────────────────────────────────────

  const elapsed = $derived(formatElapsed($orchestrationState.started_at, now));

  function formatElapsed(startedAt: string | null, _now: number): string {
    if (!startedAt) return "--:--:--";
    const diff = _now - new Date(startedAt).getTime();
    if (diff < 0) return "00:00:00";
    const h = Math.floor(diff / 3_600_000);
    const m = Math.floor((diff % 3_600_000) / 60_000);
    const s = Math.floor((diff % 60_000) / 1_000);
    return `${String(h).padStart(2, "0")}:${String(m).padStart(2, "0")}:${String(s).padStart(2, "0")}`;
  }

  const taskCounts = $derived({
    done: $orchTasks.filter((t) => t.status === "done").length,
    working: $orchTasks.filter((t) => t.status === "working").length,
    review: $orchTasks.filter((t) => t.status === "review").length,
    pending: $orchTasks.filter((t) => t.status === "pending").length,
    failed: $orchTasks.filter((t) => t.status === "failed").length,
    total: $orchTasks.length,
  });

  const activeAgents = $derived($orchAgents.filter((a) => a.status === "running" || a.status === "reviewing"));

  // ── Agent visual config ───────────────────────────────────────────────────

  const agentIcons: Record<string, typeof Code> = {
    coder: Code,
    tester: TestTube2,
    reviewer: Eye,
    unity_setup: Box,
    committer: GitCommitHorizontal,
  };

  const agentColors: Record<string, string> = {
    coder: "var(--color-agent-coder)",
    tester: "var(--color-agent-tester)",
    reviewer: "var(--color-agent-reviewer)",
    unity_setup: "var(--color-agent-unity)",
    committer: "var(--color-agent-commit)",
  };

  const agentLabels: Record<string, string> = {
    coder: "Coder",
    tester: "Tester",
    reviewer: "Reviewer",
    unity_setup: "Unity",
    committer: "Committer",
  };
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("orchestration.title")}>
    {#snippet actions()}
      <OrchControls status={$orchStatus} />
    {/snippet}
  </TopBar>

  <div class="flex-1 overflow-hidden min-h-0">
    {#if !hasData && $orchStatus === "idle" && hasActiveOrchSession}
      <!-- Initializing state -->
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
              Start the Building phase from the pipeline bar to monitor multi-agent execution in real time.
            </p>
          </div>
        {/if}
      </div>
    {:else}
      <!-- ═══════════════════════════════════════════════════════════════════
           Main orchestration dashboard — timeline + status panels
           ═══════════════════════════════════════════════════════════════════ -->
      <div class="flex flex-col h-full">
        <!-- Phase progress (top strip) -->
        {#if $orchPhases.length > 0}
          <div class="px-6 pt-4 pb-3 border-b border-[var(--color-border-subtle)]">
            <PhaseProgressBar
              phases={$orchPhases}
              currentPhase={$orchestrationState.current_phase}
              totalPhases={$orchestrationState.total_phases}
              phaseName={$orchestrationState.phase_name}
            />
          </div>
        {/if}

        <!-- Two-column layout: Timeline | Status panels -->
        <div class="flex-1 flex min-h-0">
          <!-- ── LEFT: Timeline ──────────────────────────────────────────── -->
          <div class="flex-1 min-w-0 min-h-0 border-r border-[var(--color-border-subtle)]">
            <Timeline entries={$filteredLog} />
          </div>

          <!-- ── RIGHT: Status panels ────────────────────────────────────── -->
          <div class="w-[300px] shrink-0 overflow-y-auto p-4 space-y-4">

            <!-- Elapsed time -->
            <div class="flex items-center gap-3 px-3 py-2.5 rounded-[var(--radius-lg)] bg-[var(--color-bg-surface)] border border-[var(--color-border-subtle)]">
              <div class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)] bg-[var(--color-accent)]/10">
                <Clock size={16} class="text-[var(--color-accent)]" />
              </div>
              <div>
                <div class="text-[10px] text-[var(--color-text-tertiary)] uppercase tracking-wider font-medium">
                  Elapsed
                </div>
                <div class="text-[var(--text-subtitle)] font-bold text-[var(--color-text-primary)] tabular-nums font-mono">
                  {elapsed}
                </div>
              </div>
              {#if $orchStatus === "running"}
                <div class="ml-auto w-2 h-2 rounded-full bg-[var(--color-status-success)] animate-pulse"></div>
              {/if}
            </div>

            <!-- Task progress -->
            {#if taskCounts.total > 0}
              <Panel class="p-3.5">
                <div class="flex items-center gap-2 mb-3">
                  <ListTodo size={13} class="text-[var(--color-text-secondary)]" />
                  <span class="text-[var(--text-caption)] font-semibold text-[var(--color-text-primary)]">
                    Tasks
                  </span>
                  <span class="text-[10px] text-[var(--color-text-tertiary)] ml-auto tabular-nums">
                    {taskCounts.done}/{taskCounts.total}
                  </span>
                </div>

                <!-- Stacked progress bar -->
                <div class="h-2 flex rounded-full overflow-hidden bg-[var(--color-bg-overlay)] mb-3">
                  {#if taskCounts.done > 0}
                    <div
                      class="transition-all duration-500"
                      style="width: {(taskCounts.done / taskCounts.total) * 100}%; background: var(--color-status-success)"
                    ></div>
                  {/if}
                  {#if taskCounts.working > 0}
                    <div
                      class="transition-all duration-500 relative overflow-hidden"
                      style="width: {(taskCounts.working / taskCounts.total) * 100}%; background: var(--color-status-info)"
                    >
                      <div class="absolute inset-0 bg-gradient-to-r from-transparent via-white/15 to-transparent animate-[shimmer-slide_2s_ease-in-out_infinite]"></div>
                    </div>
                  {/if}
                  {#if taskCounts.review > 0}
                    <div
                      class="transition-all duration-500"
                      style="width: {(taskCounts.review / taskCounts.total) * 100}%; background: var(--color-status-warning)"
                    ></div>
                  {/if}
                  {#if taskCounts.failed > 0}
                    <div
                      class="transition-all duration-500"
                      style="width: {(taskCounts.failed / taskCounts.total) * 100}%; background: var(--color-status-error)"
                    ></div>
                  {/if}
                </div>

                <!-- Status counts -->
                <div class="grid grid-cols-2 gap-x-4 gap-y-1.5">
                  <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" style="background: var(--color-status-success)"></div>
                    <span class="text-[10px] text-[var(--color-text-secondary)]">Done</span>
                    <span class="text-[10px] font-semibold text-[var(--color-text-primary)] ml-auto tabular-nums">{taskCounts.done}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" style="background: var(--color-status-info)"></div>
                    <span class="text-[10px] text-[var(--color-text-secondary)]">Working</span>
                    <span class="text-[10px] font-semibold text-[var(--color-text-primary)] ml-auto tabular-nums">{taskCounts.working}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" style="background: var(--color-status-warning)"></div>
                    <span class="text-[10px] text-[var(--color-text-secondary)]">Review</span>
                    <span class="text-[10px] font-semibold text-[var(--color-text-primary)] ml-auto tabular-nums">{taskCounts.review}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <div class="w-2 h-2 rounded-full" style="background: var(--color-status-pending)"></div>
                    <span class="text-[10px] text-[var(--color-text-secondary)]">Pending</span>
                    <span class="text-[10px] font-semibold text-[var(--color-text-primary)] ml-auto tabular-nums">{taskCounts.pending}</span>
                  </div>
                  {#if taskCounts.failed > 0}
                    <div class="flex items-center gap-2 col-span-2">
                      <div class="w-2 h-2 rounded-full" style="background: var(--color-status-error)"></div>
                      <span class="text-[10px] text-[var(--color-status-error)]">Failed</span>
                      <span class="text-[10px] font-semibold text-[var(--color-status-error)] ml-auto tabular-nums">{taskCounts.failed}</span>
                    </div>
                  {/if}
                </div>
              </Panel>
            {/if}

            <!-- Active agents -->
            <Panel class="p-3.5">
              <div class="flex items-center gap-2 mb-3">
                <Users size={13} class="text-[var(--color-text-secondary)]" />
                <span class="text-[var(--text-caption)] font-semibold text-[var(--color-text-primary)]">
                  Agents
                </span>
                {#if activeAgents.length > 0}
                  <span class="text-[10px] font-medium text-[var(--color-status-success)] ml-auto">
                    {activeAgents.length} active
                  </span>
                {/if}
              </div>

              {#if $orchAgents.length > 0}
                <div class="space-y-2">
                  {#each $orchAgents as agent (agent.id)}
                    {@const Icon = agentIcons[agent.agent_type] ?? Code}
                    {@const color = agentColors[agent.agent_type] ?? "var(--color-text-secondary)"}
                    {@const label = agentLabels[agent.agent_type] ?? agent.agent_type}
                    {@const isActive = agent.status === "running" || agent.status === "reviewing"}

                    <div
                      class="flex items-center gap-2.5 px-2.5 py-2 rounded-[var(--radius-md)] border transition-all duration-200
                        {isActive
                          ? 'border-[var(--color-border-active)]/30 bg-[var(--color-bg-elevated)]'
                          : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-surface)]'}"
                    >
                      <!-- Agent icon -->
                      <div
                        class="flex items-center justify-center w-7 h-7 rounded-[var(--radius-sm)] shrink-0"
                        style="background: color-mix(in srgb, {color} 12%, transparent); color: {color}"
                      >
                        <Icon size={14} />
                      </div>

                      <!-- Name + task -->
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-1.5">
                          <span class="text-[11px] font-semibold text-[var(--color-text-primary)]">
                            {agent.id}
                          </span>
                          {#if isActive}
                            <div class="w-1.5 h-1.5 rounded-full animate-pulse" style="background: {color}"></div>
                          {:else if agent.status === "passed"}
                            <CheckCircle2 size={10} class="text-[var(--color-status-success)]" />
                          {/if}
                        </div>
                        {#if agent.current_task}
                          <p class="text-[10px] text-[var(--color-text-tertiary)] truncate leading-tight">
                            {agent.current_task}
                          </p>
                        {:else}
                          <p class="text-[10px] text-[var(--color-text-tertiary)] italic leading-tight">
                            {label} - {agent.status}
                          </p>
                        {/if}
                      </div>
                    </div>
                  {/each}
                </div>
              {:else}
                <div class="flex items-center justify-center gap-2 py-3 text-[var(--color-text-tertiary)]">
                  <Circle size={14} />
                  <span class="text-[var(--text-caption)]">No agents active</span>
                </div>
              {/if}
            </Panel>

            <!-- Hooks -->
            {#if $orchHooks.length > 0}
              <Panel class="p-3.5">
                <div class="flex items-center gap-2 mb-3">
                  <Shield size={13} class="text-[var(--color-text-secondary)]" />
                  <span class="text-[var(--text-caption)] font-semibold text-[var(--color-text-primary)]">
                    Hooks
                  </span>
                </div>
                <HookMonitor hooks={$orchHooks} />
              </Panel>
            {/if}
          </div>
        </div>
      </div>
    {/if}
  </div>
</div>
