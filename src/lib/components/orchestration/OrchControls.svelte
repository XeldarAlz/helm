<script lang="ts">
  import { _ } from "svelte-i18n";
  import Button from "$lib/components/common/Button.svelte";
  import Badge from "$lib/components/common/Badge.svelte";
  import { Pause, Play, Square, Zap } from "lucide-svelte";
  import { sendOrchestrationCommand } from "$lib/utils/ipc";
  import { addToast } from "$lib/stores/toasts";

  interface Props {
    status: "idle" | "running" | "paused" | "stopped" | "completed" | "failed";
  }

  let { status }: Props = $props();

  let sending = $state<string | null>(null);

  async function handleCommand(command: "pause" | "resume" | "stop") {
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
  {/if}
</div>
