<script lang="ts">
  import { tick } from "svelte";
  import type { LogEntry } from "$lib/utils/ipc";
  import {
    Layers,
    Code,
    TestTube2,
    Eye,
    Box,
    GitCommitHorizontal,
    CheckCircle2,
    XCircle,
    AlertTriangle,
    ShieldCheck,
    Zap,
    Info,
    ChevronDown,
  } from "lucide-svelte";

  interface Props {
    entries: LogEntry[];
  }

  let { entries }: Props = $props();

  let scrollContainer: HTMLDivElement | undefined = $state();
  let autoScroll = $state(true);

  // ── Event classification ──────────────────────────────────────────────────

  type EventType =
    | "phase"
    | "agent_spawn"
    | "agent_complete"
    | "hook_pass"
    | "hook_warn"
    | "hook_fail"
    | "error"
    | "info";

  interface TimelineEvent {
    id: number;
    timestamp: string;
    type: EventType;
    title: string;
    detail: string;
    agentType?: string;
  }

  const events = $derived<TimelineEvent[]>(entries.map((e, i) => classify(e, i)));

  function classify(entry: LogEntry, i: number): TimelineEvent {
    const base = { id: i, timestamp: entry.timestamp };

    // Phase change
    const phaseMatch = entry.message.match(/^Phase (\d+):\s*(.+)/);
    if (entry.source === "system" && phaseMatch) {
      return {
        ...base,
        type: "phase",
        title: `Phase ${phaseMatch[1]}`,
        detail: phaseMatch[2],
      };
    }

    // Agent spawn
    if (entry.source.startsWith("agent:") && entry.message.startsWith("Spawned:")) {
      const agentType = entry.source.split(":")[1];
      return {
        ...base,
        type: "agent_spawn",
        title: agentLabel(agentType),
        detail: entry.message.slice(9).trim(),
        agentType,
      };
    }

    // Agent complete
    if (entry.source.startsWith("agent:") && entry.message.startsWith("Completed:")) {
      const agentType = entry.source.split(":")[1];
      return {
        ...base,
        type: "agent_complete",
        title: agentLabel(agentType),
        detail: entry.message.slice(11).trim(),
        agentType,
      };
    }

    // Hook
    if (entry.level === "hook") {
      const msg = entry.message.toLowerCase();
      const type: EventType = msg.includes("fail")
        ? "hook_fail"
        : msg.includes("warn")
          ? "hook_warn"
          : "hook_pass";
      return { ...base, type, title: entry.source, detail: entry.message };
    }

    // Error
    if (entry.level === "error") {
      return { ...base, type: "error", title: "Error", detail: entry.message };
    }

    // Default
    return { ...base, type: "info", title: entry.source || "System", detail: entry.message };
  }

  function agentLabel(type: string): string {
    const labels: Record<string, string> = {
      coder: "Coder",
      tester: "Tester",
      reviewer: "Reviewer",
      unity_setup: "Unity Setup",
      committer: "Committer",
    };
    return labels[type] ?? type;
  }

  // ── Visual mappings ───────────────────────────────────────────────────────

  const agentColors: Record<string, string> = {
    coder: "var(--color-agent-coder)",
    tester: "var(--color-agent-tester)",
    reviewer: "var(--color-agent-reviewer)",
    unity_setup: "var(--color-agent-unity)",
    committer: "var(--color-agent-commit)",
  };

  const typeColors: Record<EventType, string> = {
    phase: "var(--color-accent)",
    agent_spawn: "var(--color-status-info)",
    agent_complete: "var(--color-status-success)",
    hook_pass: "var(--color-status-success)",
    hook_warn: "var(--color-status-warning)",
    hook_fail: "var(--color-status-error)",
    error: "var(--color-status-error)",
    info: "var(--color-text-tertiary)",
  };

  function dotColor(ev: TimelineEvent): string {
    if (ev.agentType) return agentColors[ev.agentType] ?? typeColors[ev.type];
    return typeColors[ev.type];
  }

  const agentIcons: Record<string, typeof Code> = {
    coder: Code,
    tester: TestTube2,
    reviewer: Eye,
    unity_setup: Box,
    committer: GitCommitHorizontal,
  };

  function eventIcon(ev: TimelineEvent): typeof Code {
    if (ev.agentType && agentIcons[ev.agentType]) return agentIcons[ev.agentType];
    const icons: Record<EventType, typeof Code> = {
      phase: Layers,
      agent_spawn: Zap,
      agent_complete: CheckCircle2,
      hook_pass: ShieldCheck,
      hook_warn: AlertTriangle,
      hook_fail: XCircle,
      error: XCircle,
      info: Info,
    };
    return icons[ev.type];
  }

  function formatTime(ts: string): string {
    try {
      return new Date(ts).toLocaleTimeString("en-US", {
        hour12: false,
        hour: "2-digit",
        minute: "2-digit",
        second: "2-digit",
      });
    } catch {
      return ts.slice(11, 19);
    }
  }

  // ── Auto-scroll ───────────────────────────────────────────────────────────

  $effect(() => {
    if (entries.length && autoScroll && scrollContainer) {
      tick().then(() => {
        if (scrollContainer) scrollContainer.scrollTop = scrollContainer.scrollHeight;
      });
    }
  });

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainer;
    autoScroll = scrollHeight - scrollTop - clientHeight < 40;
  }
</script>

<div class="flex flex-col h-full min-h-0">
  <div
    bind:this={scrollContainer}
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto min-h-0"
  >
    {#if events.length === 0}
      <!-- Empty state -->
      <div class="flex flex-col items-center justify-center h-full gap-3">
        <div class="relative flex items-center justify-center">
          <div class="w-3 h-3 rounded-full bg-[var(--color-accent)] animate-pulse"></div>
          <div class="absolute w-6 h-6 rounded-full animate-ping opacity-20" style="background: var(--color-accent)"></div>
        </div>
        <span class="text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
          Waiting for events...
        </span>
      </div>
    {:else}
      <div class="relative pl-8 pr-3 pt-3 pb-6">
        <!-- Vertical rail -->
        <div
          class="absolute left-[13px] top-0 bottom-0 w-[2px] rounded-full"
          style="background: linear-gradient(to bottom, var(--color-border-subtle), var(--color-border-default), var(--color-border-subtle))"
        ></div>

        {#each events as event, i (event.id)}
          {#if event.type === "phase"}
            <!-- ── Phase marker ────────────────────────────────────────── -->
            <div class="relative {i > 0 ? 'mt-5' : ''} mb-3">
              <!-- Large dot with icon -->
              <div
                class="absolute -left-8 top-[1px] flex items-center justify-center w-[28px] h-[28px] rounded-full border-2 bg-[var(--color-bg-base)] z-10"
                style="border-color: var(--color-accent); box-shadow: 0 0 12px color-mix(in srgb, var(--color-accent) 25%, transparent)"
              >
                <Layers size={13} style="color: var(--color-accent)" />
              </div>

              <!-- Phase card -->
              <div
                class="ml-1 px-4 py-2.5 rounded-[var(--radius-lg)] border"
                style="
                  border-color: color-mix(in srgb, var(--color-accent) 20%, transparent);
                  background: linear-gradient(135deg,
                    color-mix(in srgb, var(--color-accent) 6%, var(--color-bg-surface)),
                    var(--color-bg-surface)
                  );
                "
              >
                <div class="flex items-center gap-2.5">
                  <span class="text-[var(--text-body)] font-bold" style="color: var(--color-accent)">
                    {event.title}
                  </span>
                  <div class="w-[1px] h-3 bg-[var(--color-border-subtle)]"></div>
                  <span class="text-[var(--text-caption)] text-[var(--color-text-secondary)] flex-1">
                    {event.detail}
                  </span>
                  <span class="text-[10px] text-[var(--color-text-tertiary)] tabular-nums shrink-0">
                    {formatTime(event.timestamp)}
                  </span>
                </div>
              </div>
            </div>
          {:else}
            <!-- ── Regular event ───────────────────────────────────────── -->
            {@const Icon = eventIcon(event)}
            {@const color = dotColor(event)}

            <div class="relative mb-0.5 group">
              <!-- Dot on rail -->
              <div class="absolute -left-8 top-[5px] flex items-center justify-center w-[28px] h-[28px]">
                <div
                  class="rounded-full transition-all duration-200
                    {event.type === 'agent_spawn' || event.type === 'agent_complete' || event.type === 'error'
                      ? 'w-[10px] h-[10px]'
                      : 'w-[7px] h-[7px]'}"
                  style="background: {color}; box-shadow: 0 0 6px color-mix(in srgb, {color} 30%, transparent)"
                ></div>
              </div>

              <!-- Content card -->
              <div class="ml-1 py-1.5 px-3 rounded-[var(--radius-md)] transition-colors duration-100 hover:bg-[var(--color-bg-elevated)]/50">
                <div class="flex items-center gap-2">
                  <div class="shrink-0" style="color: {color}">
                    <Icon size={13} />
                  </div>

                  <span class="text-[var(--text-caption)] font-semibold shrink-0" style="color: {color}">
                    {event.title}
                  </span>

                  {#if event.type === "agent_spawn"}
                    <span class="text-[10px] font-medium px-1.5 py-0.5 rounded-[var(--radius-sm)]"
                      style="background: color-mix(in srgb, {color} 12%, transparent); color: {color}"
                    >
                      spawned
                    </span>
                  {:else if event.type === "agent_complete"}
                    <span class="text-[10px] font-medium px-1.5 py-0.5 rounded-[var(--radius-sm)] bg-[var(--color-status-success)]/12 text-[var(--color-status-success)]">
                      completed
                    </span>
                  {:else if event.type === "hook_fail"}
                    <span class="text-[10px] font-medium px-1.5 py-0.5 rounded-[var(--radius-sm)] bg-[var(--color-status-error)]/12 text-[var(--color-status-error)]">
                      failed
                    </span>
                  {/if}

                  <span class="text-[10px] text-[var(--color-text-tertiary)] ml-auto shrink-0 tabular-nums opacity-0 group-hover:opacity-100 transition-opacity duration-150">
                    {formatTime(event.timestamp)}
                  </span>
                </div>

                {#if event.detail}
                  <p class="text-[11px] text-[var(--color-text-secondary)] mt-0.5 ml-[21px] leading-snug">
                    {event.detail}
                  </p>
                {/if}
              </div>
            </div>
          {/if}
        {/each}

        <!-- Live pulse at bottom of rail -->
        <div class="relative mt-2">
          <div class="absolute -left-8 top-0 flex items-center justify-center w-[28px] h-[28px]">
            <div class="w-[7px] h-[7px] rounded-full bg-[var(--color-accent)] animate-pulse"></div>
            <div class="absolute w-3.5 h-3.5 rounded-full bg-[var(--color-accent)] animate-ping opacity-20"></div>
          </div>
        </div>
      </div>
    {/if}
  </div>

  <!-- Jump to latest -->
  {#if !autoScroll}
    <button
      class="flex items-center justify-center gap-1.5 py-2 text-[11px] font-medium text-[var(--color-accent)]
        hover:bg-[var(--color-bg-overlay)]/50 rounded-[var(--radius-md)] transition-colors cursor-pointer
        border-t border-[var(--color-border-subtle)]"
      onclick={() => {
        autoScroll = true;
        if (scrollContainer) scrollContainer.scrollTop = scrollContainer.scrollHeight;
      }}
    >
      <ChevronDown size={14} />
      Jump to latest
    </button>
  {/if}
</div>
