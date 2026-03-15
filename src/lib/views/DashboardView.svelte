<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { activeView } from "$lib/stores/ui";
  import { pipelineState, type AssetCounts } from "$lib/stores/pipeline";
  import { currentSession, type SessionSummary } from "$lib/stores/session";
  import { staggeredItem, fadeScale } from "$lib/animations";
  import { timeAgo } from "$lib/utils/format";
  import { createSession, getAssetCounts, listSessions, getPipelineState } from "$lib/utils/ipc";
  import { addToast } from "$lib/stores/toasts";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Panel from "$lib/components/layout/Panel.svelte";
  import Badge from "$lib/components/common/Badge.svelte";
  import Spinner from "$lib/components/common/Spinner.svelte";
  import {
    Gamepad2,
    Play,
    BarChart3,
    FileCode,
    TestTube2,
    Box,
    Settings2,
    Check,
    Circle,
    Minus,
  } from "lucide-svelte";

  let assetCounts = $state<AssetCounts>({ scripts: 0, tests: 0, prefabs: 0, configs: 0 });
  let recentSessions = $state<SessionSummary[]>([]);
  let mounted = $state(false);
  let startingSession = $state<string | null>(null);

  // Pipeline steps with their completion checks
  const pipelineSteps = $derived([
    { key: "idea", label: $_("dashboard.pipeline.idea"), done: $pipelineState.gddExists },
    { key: "architecture", label: $_("dashboard.pipeline.architecture"), done: $pipelineState.tddExists },
    { key: "planning", label: $_("dashboard.pipeline.planning"), done: $pipelineState.workflowExists },
    { key: "building", label: $_("dashboard.pipeline.building"), done: $pipelineState.progressExists },
    {
      key: "complete",
      label: $_("dashboard.pipeline.complete"),
      done: $pipelineState.progressExists && $pipelineState.currentPhase === "complete",
    },
  ]);

  // Quick actions
  const quickActions = $derived([
    {
      id: "new-game",
      icon: Gamepad2,
      label: $_("dashboard.quickActions.newGame"),
      desc: $_("dashboard.quickActions.newGameDesc"),
      color: "var(--color-accent)",
      phase: "game_idea",
    },
    {
      id: "resume",
      icon: Play,
      label: $_("dashboard.quickActions.resume"),
      desc: $_("dashboard.quickActions.resumeDesc"),
      color: "var(--color-status-success)",
      phase: null,
    },
    {
      id: "status",
      icon: BarChart3,
      label: $_("dashboard.quickActions.status"),
      desc: $_("dashboard.quickActions.statusDesc"),
      color: "var(--color-status-info)",
      phase: null,
    },
  ]);

  async function handleQuickAction(action: typeof quickActions[number]) {
    if (action.id === "new-game") {
      startingSession = action.id;
      console.log("[helm:dashboard] starting new game_idea session…");
      try {
        const sessionId = await createSession("game_idea");
        console.log("[helm:dashboard] session created:", sessionId);
        currentSession.set({
          id: sessionId,
          phase: "game_idea",
          status: "active",
          startedAt: new Date().toISOString(),
          messageCount: 0,
          contextUsage: 0,
        });
        activeView.set("chat");
      } catch (e) {
        console.error("[helm:dashboard] session creation failed:", e);
        addToast(`Failed to start session: ${e}`, "error");
      } finally {
        startingSession = null;
      }
    } else if (action.id === "resume") {
      activeView.set("chat");
    } else if (action.id === "status") {
      activeView.set("orchestration");
    }
  }

  function phaseLabel(phase: string): string {
    const map: Record<string, string> = {
      game_idea: $_("dashboard.pipeline.idea"),
      architect: $_("dashboard.pipeline.architecture"),
      plan_workflow: $_("dashboard.pipeline.planning"),
      orchestrate: $_("dashboard.pipeline.building"),
      custom: "Custom",
    };
    return map[phase] ?? phase;
  }

  onMount(async () => {
    mounted = true;

    // Fetch asset counts and sessions in parallel
    const [assets, sessions, pipeline] = await Promise.allSettled([
      getAssetCounts(),
      listSessions(),
      getPipelineState(),
    ]);

    if (assets.status === "fulfilled") {
      assetCounts = assets.value;
    }
    if (sessions.status === "fulfilled") {
      recentSessions = sessions.value;
    }
    if (pipeline.status === "fulfilled") {
      const ps = pipeline.value;
      pipelineState.set({
        gddExists: ps.gddExists ?? (ps as any).gdd_exists ?? false,
        tddExists: ps.tddExists ?? (ps as any).tdd_exists ?? false,
        workflowExists: ps.workflowExists ?? (ps as any).workflow_exists ?? false,
        progressExists: ps.progressExists ?? (ps as any).progress_exists ?? false,
        currentPhase: ps.currentPhase ?? (ps as any).current_phase ?? "none",
      });
    }
  });
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("nav.dashboard")} />

  <div class="flex-1 overflow-y-auto">
    <div class="max-w-4xl mx-auto px-8 py-8 space-y-8">

      <!-- Welcome header -->
      {#if mounted}
        <div class="text-center mb-2" in:fadeScale={{ duration: 300 }}>
          <h2 class="text-[var(--text-heading)] font-semibold text-[var(--color-text-primary)] mb-1">
            {$_("dashboard.welcome")}
          </h2>
          <p class="text-[var(--text-subtitle)] text-[var(--color-text-secondary)]">
            {$_("dashboard.subtitle")}
          </p>
        </div>
      {/if}

      <!-- Quick actions -->
      {#if mounted}
        <div class="grid grid-cols-3 gap-4">
          {#each quickActions as action, i}
            <button
              class="group relative flex flex-col items-center gap-3 p-6 rounded-[var(--radius-xl)] border border-[var(--color-border-subtle)]
                bg-[var(--color-bg-surface)] cursor-pointer text-left
                hover:border-[var(--color-border-default)] hover:bg-[var(--color-bg-elevated)]
                hover:-translate-y-0.5 hover:shadow-[var(--shadow-lg)]
                active:translate-y-0 active:shadow-[var(--shadow-sm)]
                transition-all duration-200 ease-out
                disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:translate-y-0"
              onclick={() => handleQuickAction(action)}
              disabled={startingSession !== null}
              in:staggeredItem={{ index: i, staggerDelay: 80, duration: 300, distance: 12 }}
            >
              <!-- Icon -->
              <div
                class="flex items-center justify-center w-12 h-12 rounded-[var(--radius-lg)] transition-all duration-200"
                style="background: color-mix(in srgb, {action.color} 12%, transparent); color: {action.color}"
              >
                {#if startingSession === action.id}
                  <Spinner size="md" />
                {:else}
                  {@const Icon = action.icon}<Icon size={24} />
                {/if}
              </div>

              <!-- Text -->
              <div class="text-center">
                <span class="block text-[var(--text-body)] font-semibold text-[var(--color-text-primary)] mb-0.5">
                  {action.label}
                </span>
                <span class="block text-[var(--text-caption)] text-[var(--color-text-secondary)] leading-snug">
                  {action.desc}
                </span>
              </div>

              <!-- Hover glow -->
              <div
                class="absolute inset-0 rounded-[var(--radius-xl)] opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"
                style="box-shadow: inset 0 0 24px color-mix(in srgb, {action.color} 6%, transparent)"
              ></div>
            </button>
          {/each}
        </div>
      {/if}

      <!-- Pipeline stepper -->
      {#if mounted}
        <Panel class="p-5" >
          <div in:staggeredItem={{ index: 3, staggerDelay: 80, duration: 300, distance: 12 }}>
            <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)] mb-4">
              Pipeline Progress
            </h3>

            <div class="flex items-center gap-0">
              {#each pipelineSteps as step, i}
                <!-- Step node -->
                <div class="flex flex-col items-center gap-2 flex-1">
                  <div class="flex items-center w-full">
                    <!-- Connector before (except first) -->
                    {#if i > 0}
                      <div
                        class="flex-1 h-0.5 transition-colors duration-500"
                        class:bg-[var(--color-accent)]={step.done || pipelineSteps[i - 1].done}
                        class:bg-[var(--color-bg-overlay)]={!step.done && !pipelineSteps[i - 1].done}
                      ></div>
                    {:else}
                      <div class="flex-1"></div>
                    {/if}

                    <!-- Circle -->
                    <div
                      class="relative flex items-center justify-center w-8 h-8 rounded-full border-2 transition-all duration-500 shrink-0"
                      class:border-[var(--color-accent)]={step.done}
                      class:bg-[var(--color-accent)]={step.done}
                      class:border-[var(--color-bg-overlay)]={!step.done}
                      class:bg-[var(--color-bg-surface)]={!step.done}
                    >
                      {#if step.done}
                        <Check size={14} class="text-[var(--color-text-inverse)]" />
                      {:else}
                        <Circle size={8} class="text-[var(--color-text-tertiary)]" />
                      {/if}

                      <!-- Active glow -->
                      {#if step.done}
                        <div class="absolute inset-0 rounded-full" style="box-shadow: var(--glow-accent); opacity: 0.5"></div>
                      {/if}
                    </div>

                    <!-- Connector after (except last) -->
                    {#if i < pipelineSteps.length - 1}
                      <div
                        class="flex-1 h-0.5 transition-colors duration-500"
                        class:bg-[var(--color-accent)]={step.done && pipelineSteps[i + 1]?.done}
                        class:bg-[var(--color-bg-overlay)]={!step.done || !pipelineSteps[i + 1]?.done}
                      ></div>
                    {:else}
                      <div class="flex-1"></div>
                    {/if}
                  </div>

                  <!-- Label -->
                  <span
                    class="text-[var(--text-caption)] font-medium text-center transition-colors duration-300"
                    class:text-[var(--color-text-primary)]={step.done}
                    class:text-[var(--color-text-tertiary)]={!step.done}
                  >
                    {step.label}
                  </span>
                </div>
              {/each}
            </div>
          </div>
        </Panel>
      {/if}

      <!-- Asset counters -->
      {#if mounted}
        <div
          class="grid grid-cols-4 gap-3"
          in:staggeredItem={{ index: 4, staggerDelay: 80, duration: 300, distance: 12 }}
        >
          {#each [
            { icon: FileCode, label: $_("dashboard.assets.scripts"), count: assetCounts.scripts, color: "var(--color-agent-coder)" },
            { icon: TestTube2, label: $_("dashboard.assets.tests"), count: assetCounts.tests, color: "var(--color-agent-tester)" },
            { icon: Box, label: $_("dashboard.assets.prefabs"), count: assetCounts.prefabs, color: "var(--color-agent-unity)" },
            { icon: Settings2, label: $_("dashboard.assets.configs"), count: assetCounts.configs, color: "var(--color-agent-commit)" },
          ] as asset, i}
            {@const AssetIcon = asset.icon}
            <Panel class="p-4 flex items-center gap-3">
              <div
                class="flex items-center justify-center w-9 h-9 rounded-[var(--radius-md)]"
                style="background: color-mix(in srgb, {asset.color} 12%, transparent); color: {asset.color}"
              >
                <AssetIcon size={18} />
              </div>
              <div>
                <div class="text-[var(--text-title)] font-semibold text-[var(--color-text-primary)] leading-none mb-0.5">
                  {asset.count}
                </div>
                <div class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">
                  {asset.label}
                </div>
              </div>
            </Panel>
          {/each}
        </div>
      {/if}

      <!-- Recent sessions -->
      {#if mounted}
        <div in:staggeredItem={{ index: 5, staggerDelay: 80, duration: 300, distance: 12 }}>
          <h3 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)] mb-3">
            {$_("dashboard.recentSessions")}
          </h3>

          {#if recentSessions.length === 0}
            <Panel class="p-6">
              <p class="text-center text-[var(--text-body)] text-[var(--color-text-tertiary)]">
                {$_("dashboard.noSessions")}
              </p>
            </Panel>
          {:else}
            <div class="space-y-2">
              {#each recentSessions.slice(0, 5) as session, i}
                <Panel class="p-3 px-4 flex items-center gap-3 hover:bg-[var(--color-bg-elevated)] transition-colors duration-150 cursor-pointer">
                  <!-- Status dot -->
                  <div
                    class="w-2 h-2 rounded-full shrink-0"
                    class:bg-[var(--color-status-success)]={session.status === "active"}
                    class:animate-pulse={session.status === "active"}
                    class:bg-[var(--color-text-tertiary)]={session.status === "ended"}
                    class:bg-[var(--color-status-error)]={session.status === "error"}
                  ></div>

                  <!-- Phase -->
                  <span class="text-[var(--text-body)] font-medium text-[var(--color-text-primary)] flex-1">
                    {phaseLabel(session.phase)}
                  </span>

                  <!-- Message count -->
                  <span class="text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
                    {session.message_count} msgs
                  </span>

                  <!-- Time -->
                  <span class="text-[var(--text-caption)] text-[var(--color-text-tertiary)] w-16 text-right">
                    {timeAgo(session.started_at)}
                  </span>

                  <!-- Status badge -->
                  <Badge
                    variant={session.status === "active" ? "success" : session.status === "error" ? "error" : "default"}
                    size="sm"
                  >
                    {session.status}
                  </Badge>
                </Panel>
              {/each}
            </div>
          {/if}
        </div>
      {/if}

    </div>
  </div>
</div>
