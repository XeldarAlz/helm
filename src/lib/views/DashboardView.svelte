<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { activeView } from "$lib/stores/ui";
  import { pipelineState, phases, nextPhase, type AssetCounts } from "$lib/stores/pipeline";
  import { currentSession, messages, suggestedAnswers, type SessionSummary } from "$lib/stores/session";
  import { staggeredItem, fadeScale } from "$lib/animations";
  import { timeAgo } from "$lib/utils/format";
  import { createSession, endSession, getAssetCounts, listSessions, getPipelineState } from "$lib/utils/ipc";
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
    FileText,
    Bot,
    ArrowRight,
  } from "lucide-svelte";

  let assetCounts = $state<AssetCounts>({ scripts: 0, tests: 0, prefabs: 0, configs: 0 });
  let recentSessions = $state<SessionSummary[]>([]);
  let mounted = $state(false);
  let startingPhase = $state<string | null>(null);

  // Quick actions — context-aware based on pipeline state
  const quickActions = $derived(() => {
    const next = $nextPhase;
    const hasActive = $currentSession?.status === "active";
    const actions: { id: string; icon: typeof Gamepad2; label: string; desc: string; color: string }[] = [];

    // Primary: start next phase or new game
    if (next) {
      actions.push({
        id: `start-${next.id}`,
        icon: next.id === "game_idea" ? Gamepad2 : ArrowRight,
        label: next.id === "game_idea" ? $_("dashboard.quickActions.newGame") : `Start ${next.label}`,
        desc: next.id === "game_idea" ? $_("dashboard.quickActions.newGameDesc") : `Continue to ${next.label} phase`,
        color: "var(--color-accent)",
      });
    }

    // Resume active session
    if (hasActive) {
      actions.push({
        id: "resume",
        icon: Play,
        label: $_("dashboard.quickActions.resume"),
        desc: $_("dashboard.quickActions.resumeDesc"),
        color: "var(--color-status-success)",
      });
    }

    // View docs if any exist
    if ($pipelineState.gddExists) {
      actions.push({
        id: "documents",
        icon: FileText,
        label: "View Documents",
        desc: "Review GDD, TDD, and Workflow",
        color: "var(--color-status-info)",
      });
    }

    // Orchestration status
    if ($pipelineState.progressExists) {
      actions.push({
        id: "status",
        icon: Bot,
        label: $_("dashboard.quickActions.status"),
        desc: $_("dashboard.quickActions.statusDesc"),
        color: "var(--color-agent-unity)",
      });
    }

    return actions;
  });

  async function handleQuickAction(actionId: string) {
    if (actionId.startsWith("start-")) {
      const phaseId = actionId.replace("start-", "");
      startingPhase = phaseId;
      try {
        if ($currentSession?.status === "active") {
          await endSession($currentSession.id);
        }
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
        addToast(`Failed to start session: ${e}`, "error");
      } finally {
        startingPhase = null;
      }
    } else if (actionId === "resume") {
      activeView.set("chat");
    } else if (actionId === "status") {
      activeView.set("orchestration");
    } else if (actionId === "documents") {
      activeView.set("documents");
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

      <!-- Quick actions — dynamic based on pipeline state -->
      {#if mounted}
        {@const actions = quickActions()}
        {#if actions.length > 0}
          <div class="grid gap-4" style="grid-template-columns: repeat({Math.min(actions.length, 4)}, 1fr)">
            {#each actions as action, i}
              {@const Icon = action.icon}
              <button
                class="group relative flex flex-col items-center gap-3 p-6 rounded-[var(--radius-xl)] border border-[var(--color-border-subtle)]
                  bg-[var(--color-bg-surface)] cursor-pointer text-left
                  hover:border-[var(--color-border-default)] hover:bg-[var(--color-bg-elevated)]
                  hover:-translate-y-0.5 hover:shadow-[var(--shadow-lg)]
                  active:translate-y-0 active:shadow-[var(--shadow-sm)]
                  transition-all duration-200 ease-out
                  disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:translate-y-0"
                onclick={() => handleQuickAction(action.id)}
                disabled={startingPhase !== null}
                in:staggeredItem={{ index: i, staggerDelay: 80, duration: 300, distance: 12 }}
              >
                <div
                  class="flex items-center justify-center w-12 h-12 rounded-[var(--radius-lg)] transition-all duration-200"
                  style="background: color-mix(in srgb, {action.color} 12%, transparent); color: {action.color}"
                >
                  {#if startingPhase && action.id.startsWith("start-")}
                    <Spinner size="md" />
                  {:else}
                    <Icon size={24} />
                  {/if}
                </div>
                <div class="text-center">
                  <span class="block text-[var(--text-body)] font-semibold text-[var(--color-text-primary)] mb-0.5">
                    {action.label}
                  </span>
                  <span class="block text-[var(--text-caption)] text-[var(--color-text-secondary)] leading-snug">
                    {action.desc}
                  </span>
                </div>
                <div
                  class="absolute inset-0 rounded-[var(--radius-xl)] opacity-0 group-hover:opacity-100 transition-opacity duration-300 pointer-events-none"
                  style="box-shadow: inset 0 0 24px color-mix(in srgb, {action.color} 6%, transparent)"
                ></div>
              </button>
            {/each}
          </div>
        {:else}
          <Panel class="p-6 text-center">
            <p class="text-[var(--text-body)] text-[var(--color-text-secondary)]">
              Pipeline complete! All phases finished.
            </p>
          </Panel>
        {/if}
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
