<script lang="ts">
  import { _ } from "svelte-i18n";
  import { tick } from "svelte";
  import { activeView } from "$lib/stores/ui";
  import {
    currentSession,
    messages,
    isAgentTyping,
    suggestedAnswers,
    addUserMessage,
  } from "$lib/stores/session";
  import { phaseJustCompleted, phases, PHASE_ORDER, phaseArtifact } from "$lib/stores/pipeline";
  import { addToast } from "$lib/stores/toasts";
  import { fadeScale, staggeredItem } from "$lib/animations";
  import { createSession, sendMessage, endSession } from "$lib/utils/ipc";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Modal from "$lib/components/common/Modal.svelte";
  import Button from "$lib/components/common/Button.svelte";
  import Spinner from "$lib/components/common/Spinner.svelte";
  import AgentHeader from "$lib/components/chat/AgentHeader.svelte";
  import ChatMessage from "$lib/components/chat/ChatMessage.svelte";
  import ChatInput from "$lib/components/chat/ChatInput.svelte";
  import TypingIndicator from "$lib/components/chat/TypingIndicator.svelte";
  import SuggestedAnswers from "$lib/components/chat/SuggestedAnswers.svelte";
  import ErrorState from "$lib/components/common/ErrorState.svelte";
  import {
    MessageSquare,
    Rocket,
    Gamepad2,
    Compass,
    CalendarClock,
    FolderCog,
    Play,
    Terminal,
    CheckCircle2,
    ArrowRight,
  } from "lucide-svelte";

  let messageListEl = $state<HTMLDivElement | null>(null);
  let chatInputRef = $state<ChatInput | null>(null);
  let showEndModal = $state(false);
  let showClearModal = $state(false);
  let showNewSessionModal = $state(false);
  let startingSession = $state(false);
  let selectedPhase = $state<string | null>(null);

  // Phase display names
  const phaseNames: Record<string, string> = {
    build_game: "chat.phases.build_game",
    game_idea: "chat.phases.game_idea",
    architect: "chat.phases.architect",
    plan_workflow: "chat.phases.plan_workflow",
    init_project: "chat.phases.init_project",
    orchestrate: "chat.phases.orchestrate",
    custom: "chat.phases.custom",
  };

  const phaseName = $derived(
    $currentSession
      ? $_(phaseNames[$currentSession.phase] ?? "chat.phases.custom")
      : "",
  );

  const hasSession = $derived($currentSession !== null);
  const isActive = $derived($currentSession?.status === "active");
  const isErrored = $derived($currentSession?.status === "error");

  // Auto-scroll to bottom when messages change
  $effect(() => {
    // Touch the reactive values to track them
    void $messages.length;
    void $isAgentTyping;
    scrollToBottom();
  });

  async function scrollToBottom() {
    await tick();
    if (messageListEl) {
      messageListEl.scrollTop = messageListEl.scrollHeight;
    }
  }

  // ── Send message ──────────────────────────────────────────────────────────

  async function handleSend(content: string) {
    if (!$currentSession || !isActive) return;

    addUserMessage(content);

    // Estimate context usage (~4 chars per token, ~200k token window)
    currentSession.update((s) => {
      if (!s) return s;
      const totalChars = $messages.reduce(
        (acc, m) => acc + m.content.length,
        0,
      );
      return {
        ...s,
        messageCount: s.messageCount + 1,
        contextUsage: Math.min(1, totalChars / 800000),
      };
    });

    try {
      await sendMessage($currentSession.id, content);
    } catch (e) {
      addToast(`Failed to send: ${e}`, "error");
    }

    chatInputRef?.focus();
  }

  function handleSuggestedAnswer(answer: string) {
    handleSend(answer);
  }

  // ── Session lifecycle ─────────────────────────────────────────────────────

  async function handleNewSession() {
    if (hasSession && isActive) {
      showNewSessionModal = true;
    } else {
      showNewSessionModal = true;
    }
  }

  async function confirmNewSession(phase: string) {
    showNewSessionModal = false;
    startingSession = true;
    selectedPhase = phase;
    console.log("[helm:chat] starting session — phase:", phase);

    try {
      // End existing session if active
      if ($currentSession && isActive) {
        console.log("[helm:chat] ending previous session:", $currentSession.id);
        await endSession($currentSession.id);
      }

      // Clear frontend state
      messages.set([]);
      suggestedAnswers.set([]);

      const sessionId = await createSession(phase);
      console.log("[helm:chat] session created:", sessionId);
      currentSession.set({
        id: sessionId,
        phase,
        status: "active",
        startedAt: new Date().toISOString(),
        messageCount: 0,
        contextUsage: 0,
      });
    } catch (e) {
      console.error("[helm:chat] session creation failed:", e);
      addToast(`Failed to start session: ${e}`, "error");
    } finally {
      startingSession = false;
      selectedPhase = null;
    }
  }

  async function handleEndSession() {
    showEndModal = true;
  }

  async function confirmEndSession() {
    showEndModal = false;
    if (!$currentSession) return;

    try {
      await endSession($currentSession.id);
      currentSession.update((s) =>
        s ? { ...s, status: "ended" as const, endedAt: new Date().toISOString() } : s,
      );
      addToast($_("chat.sessionEnded"), "info");
    } catch (e) {
      addToast(`Failed to end session: ${e}`, "error");
    }
  }

  async function handleClearContext() {
    showClearModal = true;
  }

  async function confirmClearContext() {
    showClearModal = false;
    if (!$currentSession || !isActive) return;

    try {
      await sendMessage($currentSession.id, "/clear");
      messages.update((msgs) => [
        ...msgs,
        {
          id: crypto.randomUUID(),
          role: "system",
          content: $_("chat.contextCleared"),
          timestamp: new Date().toISOString(),
        },
      ]);
      currentSession.update((s) => (s ? { ...s, contextUsage: 0 } : s));
    } catch (e) {
      addToast(`Failed to clear context: ${e}`, "error");
    }
  }

  // ── Restart after crash ──────────────────────────────────────────────────

  async function handleRestartSession() {
    if (!$currentSession) return;
    const phase = $currentSession.phase;
    try {
      await endSession($currentSession.id);
    } catch {
      // Process may already be dead — ignore
    }
    await confirmNewSession(phase);
  }

  // Context usage warning
  $effect(() => {
    if ($currentSession && $currentSession.contextUsage > 0.75) {
      addToast($_("chat.contextWarning"), "warning", 6000);
    }
  });

  // ── Phase completion detection ──────────────────────────────────────────

  let completionBanner = $state<{ phaseLabel: string; artifact: string; nextPhaseId: string; nextPhaseLabel: string } | null>(null);
  let transitioningToNext = $state(false);

  // Watch for phase completions that match our active session
  $effect(() => {
    const completed = $phaseJustCompleted;
    if (!completed || !$currentSession) return;
    if (completed !== $currentSession.phase) return;

    // Find the next phase
    const currentIdx = PHASE_ORDER.findIndex((p) => p.id === completed);
    const next = currentIdx >= 0 && currentIdx < PHASE_ORDER.length - 1 ? PHASE_ORDER[currentIdx + 1] : null;
    const artifact = phaseArtifact(completed);

    if (next && artifact) {
      completionBanner = {
        phaseLabel: PHASE_ORDER[currentIdx].label,
        artifact,
        nextPhaseId: next.id,
        nextPhaseLabel: next.label,
      };
    }

    // Clear the event
    phaseJustCompleted.set(null);
  });

  async function handleContinueToNext() {
    if (!completionBanner || transitioningToNext) return;
    transitioningToNext = true;
    const nextId = completionBanner.nextPhaseId;
    completionBanner = null;

    try {
      await confirmNewSession(nextId);
    } finally {
      transitioningToNext = false;
    }
  }

  function dismissBanner() {
    completionBanner = null;
  }

  // Phase selection options for new session
  const phaseOptions = [
    { id: "build_game", icon: Rocket, color: "var(--color-accent)" },
    { id: "game_idea", icon: Gamepad2, color: "var(--color-accent)" },
    { id: "architect", icon: Compass, color: "var(--color-agent-coder)" },
    { id: "plan_workflow", icon: CalendarClock, color: "var(--color-agent-tester)" },
    { id: "init_project", icon: FolderCog, color: "var(--color-agent-reviewer)" },
    { id: "orchestrate", icon: Play, color: "var(--color-agent-unity)" },
    { id: "custom", icon: Terminal, color: "var(--color-agent-commit)" },
  ];
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("nav.chat")} />

  {#if hasSession}
    <!-- Active/ended session view -->
    <AgentHeader
      {phaseName}
      sessionStatus={$currentSession?.status ?? "ended"}
      contextUsage={$currentSession?.contextUsage ?? 0}
      onNewSession={handleNewSession}
      onEndSession={handleEndSession}
      onClearContext={handleClearContext}
    />

    {#if isErrored}
      <ErrorState
        title={$_("chat.sessionCrashed")}
        message={$_("chat.sessionCrashedDesc")}
        onretry={handleRestartSession}
      />
    {/if}

    <!-- Message list -->
    <div
      bind:this={messageListEl}
      class="flex-1 overflow-y-auto py-4"
    >
      {#if $messages.length === 0}
        <!-- Empty state while waiting for first response -->
        <div class="flex flex-col items-center justify-center h-full gap-3 text-center px-8">
          <div class="w-12 h-12 rounded-full bg-[var(--color-accent-muted)] flex items-center justify-center">
            <MessageSquare size={24} class="text-[var(--color-accent)]" />
          </div>
          <p class="text-[var(--text-body)] text-[var(--color-text-secondary)]">
            {$_("chat.emptyChat")}
          </p>
          {#if $isAgentTyping}
            <Spinner size="md" class="text-[var(--color-accent)]" />
          {/if}
        </div>
      {:else}
        {#each $messages as message (message.id)}
          <ChatMessage {message} />
        {/each}

        {#if $isAgentTyping}
          <TypingIndicator />
        {/if}
      {/if}
    </div>

    <!-- Phase completion banner -->
    {#if completionBanner}
      <div
        class="mx-4 mb-2 p-4 rounded-[var(--radius-lg)] border border-[var(--color-status-success)]
          bg-[color-mix(in_srgb,var(--color-status-success)_8%,var(--color-bg-surface))]
          flex items-center gap-3"
        in:fadeScale={{ duration: 250 }}
      >
        <CheckCircle2 size={20} class="text-[var(--color-status-success)] shrink-0" />
        <div class="flex-1 min-w-0">
          <p class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
            {completionBanner.phaseLabel} complete — {completionBanner.artifact} created
          </p>
          <p class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">
            Ready to continue to the next phase
          </p>
        </div>
        <button
          class="flex items-center gap-1.5 px-4 py-2 rounded-[var(--radius-md)] text-[13px] font-semibold cursor-pointer
            bg-[var(--color-accent)] text-[var(--color-text-inverse)]
            hover:brightness-110 hover:scale-[1.02] active:scale-[0.97]
            transition-all duration-150
            disabled:opacity-50 disabled:cursor-not-allowed"
          onclick={handleContinueToNext}
          disabled={transitioningToNext}
        >
          {#if transitioningToNext}
            <Spinner size="sm" />
          {:else}
            Continue to {completionBanner.nextPhaseLabel}
            <ArrowRight size={14} />
          {/if}
        </button>
        <button
          class="text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] transition-colors cursor-pointer p-1"
          onclick={dismissBanner}
        >
          &times;
        </button>
      </div>
    {/if}

    <!-- Suggested answers -->
    {#if isActive && $suggestedAnswers.length > 0}
      <SuggestedAnswers
        answers={$suggestedAnswers}
        onselect={handleSuggestedAnswer}
        disabled={$isAgentTyping}
      />
    {/if}

    <!-- Chat input -->
    <ChatInput
      bind:this={chatInputRef}
      disabled={!isActive || $isAgentTyping}
      onsend={handleSend}
    />
  {:else}
    <!-- No session — empty state -->
    <div class="flex-1 flex items-center justify-center p-8">
      <div class="text-center max-w-sm" in:fadeScale={{ duration: 300 }}>
        <div class="w-16 h-16 rounded-full bg-[var(--color-accent-muted)] flex items-center justify-center mx-auto mb-4">
          <MessageSquare size={32} class="text-[var(--color-accent)]" />
        </div>
        <h3 class="text-[var(--text-title)] font-semibold text-[var(--color-text-primary)] mb-2">
          {$_("chat.noSession")}
        </h3>
        <p class="text-[var(--text-body)] text-[var(--color-text-secondary)] mb-6">
          {$_("chat.noSessionDesc")}
        </p>
        <Button onclick={handleNewSession} loading={startingSession}>
          {$_("chat.startSession")}
        </Button>
      </div>
    </div>
  {/if}
</div>

<!-- End session confirmation modal -->
<Modal
  open={showEndModal}
  title={$_("chat.endSession")}
  onclose={() => (showEndModal = false)}
>
  <p class="text-[var(--text-body)] text-[var(--color-text-secondary)]">
    {$_("chat.confirmEnd")}
  </p>

  {#snippet actions()}
    <Button variant="ghost" onclick={() => (showEndModal = false)}>
      {$_("common.cancel")}
    </Button>
    <Button variant="danger" onclick={confirmEndSession}>
      {$_("chat.endSession")}
    </Button>
  {/snippet}
</Modal>

<!-- Clear context confirmation modal -->
<Modal
  open={showClearModal}
  title={$_("chat.clearContext")}
  onclose={() => (showClearModal = false)}
>
  <p class="text-[var(--text-body)] text-[var(--color-text-secondary)]">
    {$_("chat.confirmClear")}
  </p>

  {#snippet actions()}
    <Button variant="ghost" onclick={() => (showClearModal = false)}>
      {$_("common.cancel")}
    </Button>
    <Button variant="primary" onclick={confirmClearContext}>
      {$_("common.confirm")}
    </Button>
  {/snippet}
</Modal>

<!-- New session phase selection modal -->
<Modal
  open={showNewSessionModal}
  title={$_("chat.selectPhase")}
  size="md"
  onclose={() => (showNewSessionModal = false)}
>
  <div class="grid grid-cols-1 gap-2">
    {#each phaseOptions as option, i}
      {@const Icon = option.icon}
      <button
        class="flex items-center gap-3 p-3 rounded-[var(--radius-md)] border border-[var(--color-border-subtle)]
          bg-[var(--color-bg-base)] cursor-pointer text-left
          hover:bg-[var(--color-bg-elevated)] hover:border-[var(--color-border-default)]
          active:scale-[0.99] transition-all duration-150
          disabled:opacity-50 disabled:cursor-not-allowed"
        disabled={startingSession}
        onclick={() => confirmNewSession(option.id)}
        in:staggeredItem={{ index: i, staggerDelay: 50, duration: 200, distance: 8 }}
      >
        <div
          class="flex items-center justify-center w-9 h-9 rounded-[var(--radius-md)]"
          style="background: color-mix(in srgb, {option.color} 12%, transparent); color: {option.color}"
        >
          {#if startingSession && selectedPhase === option.id}
            <Spinner size="sm" />
          {:else}
            <Icon size={18} />
          {/if}
        </div>
        <div>
          <span class="block text-[var(--text-body)] font-medium text-[var(--color-text-primary)]">
            {$_(phaseNames[option.id])}
          </span>
        </div>
      </button>
    {/each}
  </div>

  {#snippet actions()}
    <Button variant="ghost" onclick={() => (showNewSessionModal = false)}>
      {$_("common.cancel")}
    </Button>
  {/snippet}
</Modal>
