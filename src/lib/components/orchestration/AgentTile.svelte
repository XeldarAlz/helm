<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { AgentInfo } from "$lib/utils/ipc";
  import Badge from "$lib/components/common/Badge.svelte";
  import ProgressBar from "$lib/components/common/ProgressBar.svelte";
  import { Code, TestTube2, Eye, Box, GitCommitHorizontal } from "lucide-svelte";

  interface Props {
    agent: AgentInfo;
  }

  let { agent }: Props = $props();

  const agentIcon = $derived(
    ({
      coder: Code,
      tester: TestTube2,
      reviewer: Eye,
      unity_setup: Box,
      committer: GitCommitHorizontal,
    })[agent.agent_type] ?? Code,
  );

  const agentLabel = $derived(
    ({
      coder: "Coder",
      tester: "Tester",
      reviewer: "Reviewer",
      unity_setup: "Unity",
      committer: "Committer",
    })[agent.agent_type] ?? agent.agent_type,
  );

  const badgeVariant = $derived(
    ({
      idle: "default" as const,
      running: "info" as const,
      reviewing: "warning" as const,
      passed: "success" as const,
      failed: "error" as const,
    })[agent.status] ?? ("default" as const),
  );

  const agentColor = $derived(
    ({
      coder: "var(--color-agent-coder)",
      tester: "var(--color-agent-tester)",
      reviewer: "var(--color-agent-reviewer)",
      unity_setup: "var(--color-agent-unity)",
      committer: "var(--color-agent-commit)",
    })[agent.agent_type] ?? "var(--color-text-secondary)",
  );

  const progressVariant = $derived(
    agent.status === "failed" ? "error" : agent.status === "passed" ? "success" : "default",
  );
</script>

<div
  class="relative flex flex-col gap-2.5 p-3.5 rounded-[var(--radius-lg)] border transition-all duration-200
    {agent.status === 'running' ? 'border-[var(--color-border-active)] bg-[var(--color-bg-elevated)]' : 'border-[var(--color-border-subtle)] bg-[var(--color-bg-surface)]'}"
>
  <!-- Top: icon + name + status badge -->
  <div class="flex items-center gap-2.5">
    <div
      class="flex items-center justify-center w-8 h-8 rounded-[var(--radius-md)]"
      style="background: color-mix(in srgb, {agentColor} 15%, transparent); color: {agentColor}"
    >
      <svelte:component this={agentIcon} size={16} />
    </div>

    <div class="flex-1 min-w-0">
      <div class="flex items-center gap-2">
        <span class="text-[var(--text-body)] font-semibold text-[var(--color-text-primary)]">
          {agent.id}
        </span>
        <Badge variant={badgeVariant} size="sm" pulse={agent.status === "running"}>
          {$_(`orchestration.agents.${agent.status}`)}
        </Badge>
      </div>
      <span class="text-[10px] text-[var(--color-text-tertiary)]">{agentLabel}</span>
    </div>
  </div>

  <!-- Task -->
  {#if agent.current_task}
    <div class="text-[var(--text-caption)] text-[var(--color-text-secondary)] truncate" title={agent.current_task}>
      {agent.current_task}
    </div>
  {:else}
    <div class="text-[var(--text-caption)] text-[var(--color-text-tertiary)] italic">
      No active task
    </div>
  {/if}

  <!-- Progress bar -->
  {#if agent.status === "running" && agent.progress > 0}
    <ProgressBar value={agent.progress} variant={progressVariant} size="sm" />
  {/if}

  <!-- Running glow -->
  {#if agent.status === "running"}
    <div
      class="absolute inset-0 rounded-[var(--radius-lg)] pointer-events-none"
      style="box-shadow: inset 0 0 16px color-mix(in srgb, {agentColor} 8%, transparent)"
    ></div>
  {/if}
</div>
