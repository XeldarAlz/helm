<script lang="ts">
  import type { AgentInfo } from "$lib/utils/ipc";
  import AgentTile from "./AgentTile.svelte";

  interface Props {
    agents: AgentInfo[];
  }

  let { agents }: Props = $props();
</script>

{#if agents.length > 0}
  <div class="grid grid-cols-2 xl:grid-cols-3 gap-3">
    {#each agents as agent (agent.id)}
      <AgentTile {agent} />
    {/each}
  </div>
{:else}
  <div class="text-center py-6 text-[var(--text-body)] text-[var(--color-text-tertiary)]">
    No agents active
  </div>
{/if}
