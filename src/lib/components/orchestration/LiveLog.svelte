<script lang="ts">
  import { _ } from "svelte-i18n";
  import { tick } from "svelte";
  import type { LogEntry } from "$lib/utils/ipc";
  import { logFilter, type LogFilter } from "$lib/stores/agents";

  interface Props {
    entries: LogEntry[];
  }

  let { entries }: Props = $props();

  let scrollContainer: HTMLDivElement | undefined = $state();
  let autoScroll = $state(true);

  const ITEM_HEIGHT = 24; // px per log row
  const OVERSCAN = 10;
  const VIRTUAL_THRESHOLD = 200; // Use virtual scrolling above this count

  let scrollTop = $state(0);
  let containerHeight = $state(400);

  // Virtual scrolling derived values
  const useVirtual = $derived(entries.length > VIRTUAL_THRESHOLD);
  const totalHeight = $derived(entries.length * ITEM_HEIGHT);
  const startIndex = $derived(
    useVirtual ? Math.max(0, Math.floor(scrollTop / ITEM_HEIGHT) - OVERSCAN) : 0
  );
  const endIndex = $derived(
    useVirtual ? Math.min(entries.length, Math.ceil((scrollTop + containerHeight) / ITEM_HEIGHT) + OVERSCAN) : entries.length
  );
  const visibleEntries = $derived(entries.slice(startIndex, endIndex));
  const offsetY = $derived(useVirtual ? startIndex * ITEM_HEIGHT : 0);

  const filters: { key: LogFilter; label: string }[] = [
    { key: "all", label: "orchestration.log.filterAll" },
    { key: "agents", label: "orchestration.log.filterAgents" },
    { key: "hooks", label: "orchestration.log.filterHooks" },
    { key: "errors", label: "orchestration.log.filterErrors" },
  ];

  const levelColors: Record<string, string> = {
    system: "var(--color-text-secondary)",
    agent: "var(--color-agent-coder)",
    hook: "var(--color-agent-commit)",
    error: "var(--color-status-error)",
  };

  // Auto-scroll on new entries
  $effect(() => {
    if (entries.length && autoScroll && scrollContainer) {
      tick().then(() => {
        if (scrollContainer) {
          scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }
      });
    }
  });

  function handleScroll() {
    if (!scrollContainer) return;
    const { scrollTop: st, scrollHeight, clientHeight } = scrollContainer;
    scrollTop = st;
    containerHeight = clientHeight;
    autoScroll = scrollHeight - st - clientHeight < 40;
  }

  function formatTime(timestamp: string): string {
    try {
      const d = new Date(timestamp);
      return d.toLocaleTimeString("en-US", { hour12: false, hour: "2-digit", minute: "2-digit", second: "2-digit" });
    } catch {
      return timestamp.slice(11, 19);
    }
  }
</script>

<div class="flex flex-col h-full min-h-0">
  <!-- Filter tabs -->
  <div class="flex items-center gap-1 px-1 pb-2.5 border-b border-[var(--color-border-subtle)]">
    <span class="text-[var(--text-caption)] font-semibold text-[var(--color-text-primary)] mr-2">
      {$_("orchestration.log.title")}
    </span>
    {#each filters as f}
      <button
        class="px-2 py-0.5 text-[10px] rounded-[var(--radius-sm)] font-medium transition-colors duration-150 cursor-pointer
          {$logFilter === f.key
            ? 'bg-[var(--color-accent)]/15 text-[var(--color-accent)]'
            : 'text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-overlay)]'}"
        onclick={() => logFilter.set(f.key)}
      >
        {$_(f.label)}
      </button>
    {/each}

    <!-- Entry count -->
    {#if entries.length > 0}
      <span class="text-[10px] text-[var(--color-text-tertiary)] ml-1">
        ({entries.length})
      </span>
    {/if}

    {#if !autoScroll}
      <button
        class="ml-auto text-[10px] text-[var(--color-accent)] hover:underline cursor-pointer"
        onclick={() => {
          autoScroll = true;
          if (scrollContainer) scrollContainer.scrollTop = scrollContainer.scrollHeight;
        }}
      >
        Jump to latest
      </button>
    {/if}
  </div>

  <!-- Log entries (virtualized for large lists) -->
  <div
    bind:this={scrollContainer}
    onscroll={handleScroll}
    class="flex-1 overflow-y-auto min-h-0 pt-2 font-mono text-[11px] leading-relaxed"
  >
    {#if entries.length === 0}
      <div class="text-center py-6 text-[var(--color-text-tertiary)]">
        No log entries
      </div>
    {:else}
      <div style={useVirtual ? `height: ${totalHeight}px; position: relative;` : ''}>
        <div style={useVirtual ? `transform: translateY(${offsetY}px);` : ''}>
          {#each visibleEntries as entry, i}
            <div
              class="flex gap-2 px-1 py-0.5 hover:bg-[var(--color-bg-overlay)]/50 rounded-sm"
              style={useVirtual ? `height: ${ITEM_HEIGHT}px;` : ''}
            >
              <span class="text-[var(--color-text-tertiary)] shrink-0 w-[60px]">
                {formatTime(entry.timestamp)}
              </span>
              <span
                class="shrink-0 w-[90px] truncate font-medium"
                style="color: {levelColors[entry.level] ?? 'var(--color-text-secondary)'}"
              >
                [{entry.source}]
              </span>
              <span
                class="flex-1 break-words truncate"
                class:text-[var(--color-text-primary)]={entry.level !== "error"}
                class:text-[var(--color-status-error)]={entry.level === "error"}
              >
                {entry.message}
              </span>
            </div>
          {/each}
        </div>
      </div>
    {/if}
  </div>
</div>
