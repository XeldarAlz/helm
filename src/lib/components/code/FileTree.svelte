<script lang="ts">
  import { ChevronRight, ChevronDown, File, Folder, FolderOpen } from "lucide-svelte";
  import type { FileNode } from "$lib/utils/ipc";
  import FileTree from "./FileTree.svelte";

  interface Props {
    nodes: FileNode[];
    selectedPath?: string;
    onSelect: (node: FileNode) => void;
    depth?: number;
    searchFilter?: string;
  }

  let {
    nodes,
    selectedPath = "",
    onSelect,
    depth = 0,
    searchFilter = "",
  }: Props = $props();

  let expandedDirs = $state<Set<string>>(new Set());

  function toggleDir(path: string) {
    const next = new Set(expandedDirs);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
    }
    expandedDirs = next;
  }

  function handleClick(node: FileNode) {
    if (node.is_dir) {
      toggleDir(node.path);
    } else {
      onSelect(node);
    }
  }

  function getFileIcon(ext: string | null): string {
    switch (ext) {
      case "cs": return "text-[var(--color-agent-coder)]";
      case "rs": return "text-[var(--color-agent-commit)]";
      case "ts":
      case "js": return "text-[var(--color-status-warning)]";
      case "svelte": return "text-[var(--color-status-error)]";
      case "json": return "text-[var(--color-status-success)]";
      case "md": return "text-[var(--color-status-info)]";
      case "css":
      case "scss": return "text-[var(--color-agent-reviewer)]";
      case "toml":
      case "yaml":
      case "yml": return "text-[var(--color-agent-tester)]";
      default: return "text-[var(--color-text-tertiary)]";
    }
  }

  function matchesSearch(node: FileNode, filter: string): boolean {
    if (!filter) return true;
    const lower = filter.toLowerCase();
    if (node.name.toLowerCase().includes(lower)) return true;
    if (node.is_dir) {
      return node.children.some((c) => matchesSearch(c, filter));
    }
    return false;
  }

  const filteredNodes = $derived(
    searchFilter ? nodes.filter((n) => matchesSearch(n, searchFilter)) : nodes,
  );

  // Auto-expand dirs that match search
  $effect(() => {
    if (searchFilter && searchFilter.length >= 2) {
      const toExpand = new Set(expandedDirs);
      for (const node of filteredNodes) {
        if (node.is_dir && node.children.some((c) => matchesSearch(c, searchFilter))) {
          toExpand.add(node.path);
        }
      }
      expandedDirs = toExpand;
    }
  });
</script>

<div class="text-[12px]">
  {#each filteredNodes as node}
    {@const isExpanded = expandedDirs.has(node.path)}
    {@const isSelected = selectedPath === node.path}

    <button
      class="flex items-center gap-1 w-full px-1 py-0.5 rounded-[var(--radius-sm)] text-left transition-colors duration-75 cursor-pointer
        {isSelected
          ? 'bg-[var(--color-accent-muted)] text-[var(--color-accent)]'
          : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)] hover:text-[var(--color-text-primary)]'}"
      style="padding-left: {depth * 14 + 4}px"
      onclick={() => handleClick(node)}
    >
      {#if node.is_dir}
        <!-- Directory chevron -->
        <span class="w-3.5 h-3.5 flex items-center justify-center shrink-0 text-[var(--color-text-tertiary)]">
          {#if isExpanded}
            <ChevronDown size={12} />
          {:else}
            <ChevronRight size={12} />
          {/if}
        </span>
        <!-- Folder icon -->
        <span class="shrink-0 text-[var(--color-status-warning)]">
          {#if isExpanded}
            <FolderOpen size={14} />
          {:else}
            <Folder size={14} />
          {/if}
        </span>
      {:else}
        <!-- File spacer + icon -->
        <span class="w-3.5 shrink-0"></span>
        <span class="shrink-0 {getFileIcon(node.extension)}">
          <File size={14} />
        </span>
      {/if}

      <span class="truncate">{node.name}</span>
    </button>

    <!-- Children (expanded directory) -->
    {#if node.is_dir && isExpanded && node.children.length > 0}
      <FileTree
        nodes={node.children}
        {selectedPath}
        {onSelect}
        depth={depth + 1}
        {searchFilter}
      />
    {/if}
  {/each}
</div>
