<script lang="ts">
  import { _ } from "svelte-i18n";
  import Button from "$lib/components/common/Button.svelte";
  import Badge from "$lib/components/common/Badge.svelte";
  import { Pause, Play, Square, Zap, Shield, Leaf, Sparkles } from "lucide-svelte";
  import { sendOrchestrationCommand } from "$lib/utils/ipc";
  import { addToast } from "$lib/stores/toasts";

  interface Props {
    status: "idle" | "running" | "paused" | "stopped" | "completed" | "failed";
    ecoMode?: boolean;
    stopProtected?: boolean;
  }

  let { status, ecoMode = false, stopProtected = false }: Props = $props();

  let sending = $state<string | null>(null);

  async function handleCommand(command: "pause" | "resume" | "stop" | "clean-slop") {
    sending = command;
    try {
      await sendOrchestrationCommand(command);
      addToast(`Orchestration: ${command} sent`, "info");
    } catch (e) {
      addToast(`Failed to ${command}: ${e}`, "error");
    } finally {
      sending = null;
    }
  }

  const statusVariant = $derived(
    ({
      idle: "default" as const,
      running: "success" as const,
      paused: "warning" as const,
      stopped: "default" as const,
      completed: "success" as const,
      failed: "error" as const,
    })[status] ?? ("default" as const),
  );

  const statusLabel = $derived(
    ({
      idle: "Idle",
      running: "Running",
      paused: "Paused",
      stopped: "Stopped",
      completed: "Completed",
      failed: "Failed",
    })[status] ?? status,
  );
</script>

<div class="flex items-center gap-3">
  <!-- Status badge -->
  <Badge variant={statusVariant} size="md" pulse={status === "running"}>
    {#if status === "running"}
      <Zap size={10} class="mr-1 inline" />
    {/if}
    {statusLabel}
  </Badge>

  {#if ecoMode}
    <Badge variant="success" size="sm">
      <Leaf size={10} class="mr-0.5 inline" />
      {$_("orchestration.ecoMode")}
    </Badge>
  {/if}

  {#if stopProtected}
    <span
      class="text-[8px] font-bold px-1 py-0.5 rounded-[2px] uppercase tracking-wider"
      style="background: color-mix(in srgb, var(--color-accent) 15%, transparent); color: var(--color-accent)"
      title={$_("orchestration.stopProtectedTooltip")}
    >
      <Shield size={8} class="inline mr-0.5" />{$_("orchestration.stopProtected")}
    </span>
  {/if}

  <!-- Action buttons -->
  {#if status === "running"}
    <Button
      variant="secondary"
      size="sm"
      loading={sending === "pause"}
      disabled={sending !== null}
      onclick={() => handleCommand("pause")}
    >
      {#snippet icon()}<Pause size={12} />{/snippet}
      {$_("orchestration.controls.pause")}
    </Button>
    <Button
      variant="danger"
      size="sm"
      loading={sending === "stop"}
      disabled={sending !== null}
      onclick={() => handleCommand("stop")}
    >
      {#snippet icon()}<Square size={12} />{/snippet}
      {$_("orchestration.controls.stop")}
    </Button>
  {:else if status === "paused"}
    <Button
      variant="primary"
      size="sm"
      loading={sending === "resume"}
      disabled={sending !== null}
      onclick={() => handleCommand("resume")}
    >
      {#snippet icon()}<Play size={12} />{/snippet}
      {$_("orchestration.controls.resume")}
    </Button>
    <Button
      variant="danger"
      size="sm"
      loading={sending === "stop"}
      disabled={sending !== null}
      onclick={() => handleCommand("stop")}
    >
      {#snippet icon()}<Square size={12} />{/snippet}
      {$_("orchestration.controls.stop")}
    </Button>
  {:else if status === "completed"}
    <Button
      variant="secondary"
      size="sm"
      loading={sending === "clean-slop"}
      disabled={sending !== null}
      onclick={() => handleCommand("clean-slop")}
    >
      {#snippet icon()}<Sparkles size={12} />{/snippet}
      {$_("orchestration.controls.cleanSlop")}
    </Button>
  {/if}
</div>
