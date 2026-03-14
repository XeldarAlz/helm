<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { staggeredItem, fadeScale } from "$lib/animations";
  import { activeView } from "$lib/stores/ui";
  import { currentSession, type SessionSummary } from "$lib/stores/session";
  import { addToast } from "$lib/stores/toasts";
  import { timeAgo } from "$lib/utils/format";
  import { listSessions, getSessionMessages, saveTranscript } from "$lib/utils/ipc";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Panel from "$lib/components/layout/Panel.svelte";
  import Badge from "$lib/components/common/Badge.svelte";
  import TextInput from "$lib/components/common/TextInput.svelte";
  import Dropdown from "$lib/components/common/Dropdown.svelte";
  import Spinner from "$lib/components/common/Spinner.svelte";
  import {
    Search,
    MessageSquare,
    Clock,
    Download,
    History,
    Filter,
    ChevronDown,
    ChevronRight,
  } from "lucide-svelte";

  let sessions = $state<SessionSummary[]>([]);
  let searchTerm = $state("");
  let phaseFilter = $state("all");
  let statusFilter = $state("all");
  let loading = $state(true);
  let expandedSession = $state<string | null>(null);
  let sessionMessages = $state<Record<string, { id: string; role: string; content: string; timestamp: string }[]>>({});
  let loadingMessages = $state<string | null>(null);
  let mounted = $state(false);

  const phaseOptions = [
    { value: "all", label: "All Phases" },
    { value: "game_idea", label: "Game Idea" },
    { value: "architect", label: "Architecture" },
    { value: "plan_workflow", label: "Planning" },
    { value: "orchestrate", label: "Orchestration" },
    { value: "custom", label: "Custom" },
  ];

  const statusOptions = [
    { value: "all", label: "All Statuses" },
    { value: "active", label: "Active" },
    { value: "ended", label: "Ended" },
    { value: "error", label: "Error" },
  ];

  const filtered = $derived(
    sessions.filter((s) => {
      if (phaseFilter !== "all" && s.phase !== phaseFilter) return false;
      if (statusFilter !== "all" && s.status !== statusFilter) return false;
      if (searchTerm) {
        const term = searchTerm.toLowerCase();
        return (
          s.phase.toLowerCase().includes(term) ||
          s.id.toLowerCase().includes(term)
        );
      }
      return true;
    })
  );

  function phaseLabel(phase: string): string {
    const map: Record<string, string> = {
      game_idea: "Game Idea",
      architect: "Architecture",
      plan_workflow: "Planning",
      orchestrate: "Orchestration",
      custom: "Custom",
    };
    return map[phase] ?? phase;
  }

  function phaseColor(phase: string): string {
    const map: Record<string, string> = {
      game_idea: "var(--color-accent)",
      architect: "var(--color-agent-coder)",
      plan_workflow: "var(--color-agent-tester)",
      orchestrate: "var(--color-agent-reviewer)",
      custom: "var(--color-text-secondary)",
    };
    return map[phase] ?? "var(--color-text-secondary)";
  }

  function statusVariant(status: string): "success" | "error" | "default" {
    if (status === "active") return "success";
    if (status === "error") return "error";
    return "default";
  }

  function formatDuration(startedAt: string, endedAt: string | null): string {
    const start = new Date(startedAt).getTime();
    const end = endedAt ? new Date(endedAt).getTime() : Date.now();
    const diff = end - start;
    const mins = Math.floor(diff / 60000);
    const hours = Math.floor(diff / 3600000);
    if (hours > 0) return `${hours}h ${mins % 60}m`;
    if (mins > 0) return `${mins}m`;
    return "<1m";
  }

  async function toggleSession(id: string) {
    if (expandedSession === id) {
      expandedSession = null;
      return;
    }
    expandedSession = id;

    if (!sessionMessages[id]) {
      loadingMessages = id;
      try {
        const msgs = await getSessionMessages(id);
        sessionMessages = { ...sessionMessages, [id]: msgs };
      } catch (e) {
        addToast(`Failed to load messages: ${e}`, "error");
      } finally {
        loadingMessages = null;
      }
    }
  }

  async function handleSaveTranscript(id: string) {
    try {
      const path = await saveTranscript(id);
      addToast(`Transcript saved: ${path}`, "success");
    } catch (e) {
      addToast(`Failed to save: ${e}`, "error");
    }
  }

  onMount(async () => {
    mounted = true;
    try {
      sessions = await listSessions();
    } catch (e) {
      addToast(`Failed to load sessions: ${e}`, "error");
    } finally {
      loading = false;
    }
  });
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("history.title")}>
    {#snippet actions()}
      <div class="flex items-center gap-2">
        <span class="text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
          {filtered.length} session{filtered.length !== 1 ? "s" : ""}
        </span>
      </div>
    {/snippet}
  </TopBar>

  <div class="flex-1 overflow-hidden flex flex-col">
    <!-- Search & Filter Bar -->
    <div class="flex items-center gap-3 px-6 py-3 border-b border-[var(--color-border-subtle)]">
      <div class="relative flex-1 max-w-xs">
        <Search size={14} class="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-tertiary)]" />
        <input
          type="text"
          bind:value={searchTerm}
          placeholder={$_("history.search")}
          class="w-full h-8 pl-8 pr-3 text-[12px] bg-[var(--color-bg-surface)] text-[var(--color-text-primary)] placeholder-[var(--color-text-tertiary)]
            border border-[var(--color-border-default)] rounded-[var(--radius-md)]
            focus:outline-none focus:border-[var(--color-border-active)] transition-all duration-150"
        />
      </div>

      <Dropdown
        options={phaseOptions}
        bind:value={phaseFilter}
        class="w-36 !h-8 !text-[12px]"
      />

      <Dropdown
        options={statusOptions}
        bind:value={statusFilter}
        class="w-32 !h-8 !text-[12px]"
      />
    </div>

    <!-- Session List -->
    <div class="flex-1 overflow-y-auto">
      {#if loading}
        <div class="flex items-center justify-center h-full">
          <Spinner size="lg" />
        </div>
      {:else if filtered.length === 0}
        <div class="flex items-center justify-center h-full p-8">
          <div class="text-center max-w-sm">
            <History size={40} class="text-[var(--color-text-tertiary)] mx-auto mb-3" />
            <h3 class="text-[var(--text-subtitle)] font-semibold text-[var(--color-text-primary)] mb-1">
              {searchTerm || phaseFilter !== "all" || statusFilter !== "all" ? "No matching sessions" : $_("history.noHistory")}
            </h3>
            <p class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">
              {$_("history.noHistoryDesc")}
            </p>
          </div>
        </div>
      {:else}
        <div class="px-6 py-4 space-y-2">
          {#each filtered as session, i}
            <div
              in:staggeredItem={{ index: Math.min(i, 10), staggerDelay: 30, duration: 200, distance: 6 }}
            >
              <Panel class="overflow-hidden">
                <!-- Session Header -->
                <div
                  class="w-full text-left px-4 py-3 flex items-center gap-3 cursor-pointer
                    hover:bg-[var(--color-bg-elevated)] transition-colors duration-150"
                  role="button"
                  tabindex="0"
                  onclick={() => toggleSession(session.id)}
                  onkeydown={(e: KeyboardEvent) => { if (e.key === "Enter" || e.key === " ") toggleSession(session.id); }}
                >
                  <!-- Phase indicator -->
                  <div
                    class="w-1 h-10 rounded-full shrink-0"
                    style="background: {phaseColor(session.phase)}"
                  ></div>

                  <!-- Info -->
                  <div class="flex-1 min-w-0">
                    <div class="flex items-center gap-2 mb-0.5">
                      <span class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
                        {phaseLabel(session.phase)}
                      </span>
                      <Badge variant={statusVariant(session.status)} size="sm">
                        {session.status}
                      </Badge>
                    </div>
                    <div class="flex items-center gap-3 text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
                      <span class="flex items-center gap-1">
                        <MessageSquare size={11} />
                        {session.message_count} msgs
                      </span>
                      <span class="flex items-center gap-1">
                        <Clock size={11} />
                        {formatDuration(session.started_at, session.ended_at)}
                      </span>
                      <span>{timeAgo(session.started_at)}</span>
                    </div>
                  </div>

                  <!-- Actions -->
                  <div class="flex items-center gap-2 shrink-0">
                    <button
                      class="p-1.5 rounded-[var(--radius-sm)] text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-overlay)] transition-colors cursor-pointer"
                      onclick={(e: MouseEvent) => { e.stopPropagation(); handleSaveTranscript(session.id); }}
                      title="Save transcript"
                    >
                      <Download size={14} />
                    </button>
                    {#if expandedSession === session.id}
                      <ChevronDown size={16} class="text-[var(--color-text-tertiary)]" />
                    {:else}
                      <ChevronRight size={16} class="text-[var(--color-text-tertiary)]" />
                    {/if}
                  </div>
                </div>

                <!-- Expanded: Message preview -->
                {#if expandedSession === session.id}
                  <div class="border-t border-[var(--color-border-subtle)] bg-[var(--color-bg-base)]">
                    {#if loadingMessages === session.id}
                      <div class="flex items-center justify-center py-6">
                        <Spinner size="sm" />
                      </div>
                    {:else if sessionMessages[session.id]}
                      <div class="max-h-64 overflow-y-auto px-4 py-3 space-y-2">
                        {#each sessionMessages[session.id].slice(0, 20) as msg}
                          <div class="flex gap-2 text-[var(--text-caption)]">
                            <span
                              class="shrink-0 w-12 font-semibold"
                              class:text-[var(--color-accent)]={msg.role === "user"}
                              class:text-[var(--color-agent-coder)]={msg.role === "agent"}
                              class:text-[var(--color-text-tertiary)]={msg.role === "system"}
                            >
                              {msg.role === "user" ? "You" : msg.role === "agent" ? "Claude" : "System"}
                            </span>
                            <p class="text-[var(--color-text-primary)] line-clamp-3 flex-1">
                              {msg.content}
                            </p>
                          </div>
                        {/each}
                        {#if sessionMessages[session.id].length > 20}
                          <p class="text-[10px] text-[var(--color-text-tertiary)] text-center pt-1">
                            +{sessionMessages[session.id].length - 20} more messages
                          </p>
                        {/if}
                      </div>
                    {:else}
                      <p class="px-4 py-3 text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
                        No messages
                      </p>
                    {/if}
                  </div>
                {/if}
              </Panel>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  </div>
</div>
