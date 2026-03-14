<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { getFileTree, readProjectFile, type FileNode } from "$lib/utils/ipc";
  import { fadeScale } from "$lib/animations";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import SplitPane from "$lib/components/layout/SplitPane.svelte";
  import FileTree from "$lib/components/code/FileTree.svelte";
  import CodeViewer from "$lib/components/code/CodeViewer.svelte";
  import Spinner from "$lib/components/common/Spinner.svelte";
  import { Search, FolderTree, FileCode } from "lucide-svelte";

  let fileTree = $state<FileNode[]>([]);
  let selectedFile = $state<FileNode | null>(null);
  let fileContent = $state("");
  let loading = $state(false);
  let treeLoading = $state(true);
  let searchFilter = $state("");

  let unlistenCodeChanged: (() => void) | null = null;

  async function loadTree() {
    treeLoading = true;
    try {
      fileTree = await getFileTree();
    } catch {
      fileTree = [];
    } finally {
      treeLoading = false;
    }
  }

  async function openFile(node: FileNode) {
    if (node.is_dir) return;
    selectedFile = node;
    loading = true;
    try {
      fileContent = await readProjectFile(node.path);
    } catch (e) {
      fileContent = `Error reading file: ${e}`;
    } finally {
      loading = false;
    }
  }

  onMount(async () => {
    await loadTree();

    // Listen for code changes to refresh tree
    unlistenCodeChanged = await listen("code-changed", () => {
      loadTree();
      // If the changed file is currently open, reload it
      if (selectedFile) {
        openFile(selectedFile);
      }
    });
  });

  onDestroy(() => {
    unlistenCodeChanged?.();
  });
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("code.title")} />

  <div class="flex-1 overflow-hidden">
    {#snippet left()}
      <div class="h-full flex flex-col bg-[var(--color-bg-surface)] border-r border-[var(--color-border-subtle)]">
        <!-- Search bar -->
        <div class="px-2 pt-2 pb-1">
          <div class="relative flex items-center">
            <Search size={12} class="absolute left-2 text-[var(--color-text-tertiary)] pointer-events-none" />
            <input
              type="text"
              bind:value={searchFilter}
              placeholder={$_("code.searchFiles")}
              class="w-full h-7 pl-7 pr-2 text-[11px] bg-[var(--color-bg-deep)] text-[var(--color-text-primary)] placeholder-[var(--color-text-tertiary)]
                border border-[var(--color-border-subtle)] rounded-[var(--radius-sm)]
                focus:outline-none focus:border-[var(--color-border-active)]
                transition-colors duration-150"
            />
          </div>
        </div>

        <!-- File tree -->
        <div class="flex-1 overflow-y-auto px-1 py-1">
          {#if treeLoading}
            <div class="flex items-center justify-center py-8">
              <Spinner size="sm" />
            </div>
          {:else if fileTree.length === 0}
            <div class="flex flex-col items-center gap-2 py-8 px-4 text-center">
              <FolderTree size={20} class="text-[var(--color-text-tertiary)]" />
              <p class="text-[11px] text-[var(--color-text-tertiary)]">
                {$_("code.emptyTree")}
              </p>
            </div>
          {:else}
            <FileTree
              nodes={fileTree}
              selectedPath={selectedFile?.path ?? ""}
              onSelect={openFile}
              {searchFilter}
            />
          {/if}
        </div>
      </div>
    {/snippet}

    {#snippet right()}
      <div class="h-full">
        {#if loading}
          <div class="flex items-center justify-center h-full">
            <Spinner size="lg" />
          </div>
        {:else if selectedFile}
          <CodeViewer
            content={fileContent}
            filePath={selectedFile.path}
            fileSize={selectedFile.size}
          />
        {:else}
          <!-- Empty state -->
          <div class="flex flex-col items-center justify-center h-full gap-4" in:fadeScale={{ duration: 200 }}>
            <div class="flex items-center justify-center w-16 h-16 rounded-[var(--radius-xl)] bg-[var(--color-bg-elevated)]">
              <FileCode size={28} class="text-[var(--color-text-tertiary)]" />
            </div>
            <p class="text-[var(--text-body)] text-[var(--color-text-tertiary)]">
              {$_("code.noFile")}
            </p>
          </div>
        {/if}
      </div>
    {/snippet}

    <SplitPane direction="horizontal" initialSplit={240} minSize={180} {left} {right} />
  </div>
</div>
