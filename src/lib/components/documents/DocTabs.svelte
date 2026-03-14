<script lang="ts">
  import { _ } from "svelte-i18n";
  import { pipelineState } from "$lib/stores/pipeline";
  import { activeDocTab, type DocName } from "$lib/stores/documents";
  import Tooltip from "$lib/components/common/Tooltip.svelte";

  const tabs: { key: DocName; existsKey: keyof typeof $pipelineState }[] = [
    { key: "GDD", existsKey: "gddExists" },
    { key: "TDD", existsKey: "tddExists" },
    { key: "WORKFLOW", existsKey: "workflowExists" },
    { key: "PROGRESS", existsKey: "progressExists" },
    { key: "ACTIVITY_LOG", existsKey: "progressExists" },
  ];

  function exists(tab: typeof tabs[number]): boolean {
    if (tab.key === "ACTIVITY_LOG") return true; // Activity log may always exist
    return ($pipelineState as any)[tab.existsKey] ?? false;
  }

  function selectTab(key: DocName) {
    activeDocTab.set(key);
  }
</script>

<div class="flex items-center gap-1 px-4 border-b border-[var(--color-border-subtle)] bg-[var(--color-bg-base)] overflow-x-auto">
  {#each tabs as tab}
    {@const isActive = $activeDocTab === tab.key}
    {@const tabExists = exists(tab)}
    <button
      class="relative px-3 py-2.5 text-[12px] font-medium whitespace-nowrap transition-colors duration-150 cursor-pointer
        {isActive
          ? 'text-[var(--color-accent)]'
          : tabExists
            ? 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]'
            : 'text-[var(--color-text-tertiary)] cursor-default'}"
      onclick={() => selectTab(tab.key)}
    >
      {$_(`documents.tabs.${tab.key}`)}

      <!-- Active indicator -->
      {#if isActive}
        <div class="absolute bottom-0 left-1 right-1 h-0.5 bg-[var(--color-accent)] rounded-full"></div>
      {/if}

      <!-- Not created badge -->
      {#if !tabExists}
        <span class="ml-1.5 inline-block w-1.5 h-1.5 rounded-full bg-[var(--color-text-tertiary)] opacity-50"></span>
      {/if}
    </button>
  {/each}
</div>
