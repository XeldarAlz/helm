<script lang="ts">
  import { _ } from "svelte-i18n";
  import { Plus, X, Eraser } from "lucide-svelte";
  import IconButton from "$lib/components/common/IconButton.svelte";
  import Badge from "$lib/components/common/Badge.svelte";
  import ContextMeter from "./ContextMeter.svelte";

  interface Props {
    phaseName: string;
    sessionStatus: "active" | "ended" | "error";
    contextUsage?: number;
    onNewSession: () => void;
    onEndSession: () => void;
    onClearContext: () => void;
  }

  let {
    phaseName,
    sessionStatus,
    contextUsage = 0,
    onNewSession,
    onEndSession,
    onClearContext,
  }: Props = $props();

  const isActive = $derived(sessionStatus === "active");
</script>

<div class="flex items-center justify-between h-11 px-4 border-b border-[var(--color-border-subtle)] bg-[var(--color-bg-surface)]">
  <!-- Left: phase name + status -->
  <div class="flex items-center gap-3">
    <h2 class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
      {phaseName}
    </h2>
    <Badge
      variant={isActive ? "success" : sessionStatus === "error" ? "error" : "default"}
      pulse={isActive}
    >
      {sessionStatus}
    </Badge>
  </div>

  <!-- Right: context meter + controls -->
  <div class="flex items-center gap-3">
    {#if isActive}
      <ContextMeter value={contextUsage} />
    {/if}

    <div class="flex items-center gap-1">
      <IconButton
        size="sm"
        tooltip={$_("chat.clearContext")}
        disabled={!isActive}
        onclick={onClearContext}
      >
        <Eraser size={14} />
      </IconButton>
      <IconButton
        size="sm"
        tooltip={$_("chat.endSession")}
        disabled={!isActive}
        onclick={onEndSession}
      >
        <X size={14} />
      </IconButton>
      <IconButton
        size="sm"
        tooltip={$_("chat.newSession")}
        onclick={onNewSession}
      >
        <Plus size={14} />
      </IconButton>
    </div>
  </div>
</div>
