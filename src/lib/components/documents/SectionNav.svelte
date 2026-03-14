<script lang="ts">
  import { _ } from "svelte-i18n";
  import type { DocSection } from "$lib/stores/documents";

  interface Props {
    sections: DocSection[];
    activeSection?: string;
    onSelect: (id: string) => void;
  }

  let { sections, activeSection = "", onSelect }: Props = $props();
</script>

<div class="flex flex-col h-full">
  <div class="px-3 py-2 text-[11px] font-semibold uppercase tracking-wider text-[var(--color-text-tertiary)]">
    {$_("documents.sections")}
  </div>

  <div class="flex-1 overflow-y-auto px-1">
    {#if sections.length === 0}
      <p class="px-3 py-2 text-[11px] text-[var(--color-text-tertiary)]">
        {$_("documents.noSections")}
      </p>
    {:else}
      <nav class="space-y-0.5">
        {#each sections as section}
          {@const isActive = activeSection === section.id}
          <button
            class="w-full text-left px-2 py-1 rounded-[var(--radius-sm)] text-[11px] leading-snug transition-colors duration-100 cursor-pointer truncate
              {isActive
                ? 'bg-[var(--color-accent-muted)] text-[var(--color-accent)]'
                : 'text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)] hover:bg-[var(--color-bg-elevated)]'}"
            style="padding-left: {(section.level - 1) * 12 + 8}px"
            onclick={() => onSelect(section.id)}
          >
            {section.text}
          </button>
        {/each}
      </nav>
    {/if}
  </div>

  <!-- Back to top -->
  {#if sections.length > 0}
    <button
      class="mx-2 mb-2 px-2 py-1.5 text-[11px] text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)]
        border border-[var(--color-border-subtle)] rounded-[var(--radius-sm)] transition-colors cursor-pointer"
      onclick={() => onSelect("top")}
    >
      {$_("documents.scrollToTop")}
    </button>
  {/if}
</div>
