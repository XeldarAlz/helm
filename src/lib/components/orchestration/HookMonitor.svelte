<script lang="ts">
  import type { HookResultInfo } from "$lib/utils/ipc";
  import { Shield, ShieldAlert, ShieldCheck, ShieldQuestion, Loader2 } from "lucide-svelte";
  import { timeAgo } from "$lib/utils/format";

  interface Props {
    hooks: HookResultInfo[];
  }

  let { hooks }: Props = $props();

  const statusConfig = {
    passed: { icon: ShieldCheck, color: "var(--color-status-success)", label: "Passed" },
    warning: { icon: ShieldAlert, color: "var(--color-status-warning)", label: "Warning" },
    failed: { icon: ShieldAlert, color: "var(--color-status-error)", label: "Failed" },
    running: { icon: Loader2, color: "var(--color-status-info)", label: "Running" },
  };
</script>

{#if hooks.length > 0}
  <div class="space-y-1.5">
    {#each hooks as hook}
      {@const config = statusConfig[hook.status] ?? statusConfig.passed}
      {@const Icon = config.icon}
      <div
        class="flex items-center gap-2.5 px-3 py-2 rounded-[var(--radius-md)]
          border border-[var(--color-border-subtle)] bg-[var(--color-bg-surface)]"
      >
        <div style="color: {config.color}">
          <Icon size={14} class={hook.status === "running" ? "animate-spin" : ""} />
        </div>
        <span class="text-[var(--text-caption)] font-medium text-[var(--color-text-primary)] flex-1 font-mono">
          {hook.name}
        </span>
        <span class="text-[10px] font-medium" style="color: {config.color}">
          {config.label}
        </span>
        {#if hook.last_run}
          <span class="text-[9px] text-[var(--color-text-tertiary)] w-14 text-right">
            {timeAgo(hook.last_run)}
          </span>
        {/if}
      </div>
    {/each}
  </div>
{:else}
  <div class="flex items-center justify-center gap-2 py-4 text-[var(--color-text-tertiary)]">
    <ShieldQuestion size={16} />
    <span class="text-[var(--text-caption)]">No hook data</span>
  </div>
{/if}
