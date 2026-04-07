<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { staggeredItem, fadeScale } from "$lib/animations";
  import { addToast } from "$lib/stores/toasts";
  import { timeAgo } from "$lib/utils/format";
  import {
    getGitLog,
    getGitBranches,
    getGitStatus,
    getGitDiff,
    getCommitTrailers,
    type GitCommit,
    type GitBranch,
    type GitStatus,
    type GitDiffFile,
  } from "$lib/utils/ipc";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import Panel from "$lib/components/layout/Panel.svelte";
  import Badge from "$lib/components/common/Badge.svelte";
  import Spinner from "$lib/components/common/Spinner.svelte";
  import IconButton from "$lib/components/common/IconButton.svelte";
  import {
    GitBranch as GitBranchIcon,
    GitCommit as GitCommitIcon,
    RefreshCw,
    Plus,
    Minus,
    FileCode,
    ChevronDown,
    ChevronRight,
    ArrowUp,
    ArrowDown,
    Circle,
  } from "lucide-svelte";

  let commits = $state<GitCommit[]>([]);
  let branches = $state<GitBranch[]>([]);
  let gitStatus = $state<GitStatus | null>(null);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let expandedCommit = $state<string | null>(null);
  let commitFiles = $state<Record<string, GitDiffFile[]>>({});
  let commitTrailers = $state<Record<string, Record<string, string>>>({});
  let mounted = $state(false);

  async function loadData() {
    loading = true;
    error = null;
    try {
      const [logResult, branchResult, statusResult] = await Promise.allSettled([
        getGitLog(80),
        getGitBranches(),
        getGitStatus(),
      ]);

      if (logResult.status === "fulfilled") commits = logResult.value;
      else error = logResult.reason?.toString() ?? "Failed to load git log";

      if (branchResult.status === "fulfilled") branches = branchResult.value;
      if (statusResult.status === "fulfilled") gitStatus = statusResult.value;
    } catch (e) {
      error = `${e}`;
    } finally {
      loading = false;
    }
  }

  async function toggleCommit(hash: string) {
    if (expandedCommit === hash) {
      expandedCommit = null;
      return;
    }
    expandedCommit = hash;

    if (!commitFiles[hash]) {
      try {
        const [files, trailers] = await Promise.all([
          getGitDiff(hash),
          getCommitTrailers(hash),
        ]);
        commitFiles = { ...commitFiles, [hash]: files };
        commitTrailers = { ...commitTrailers, [hash]: trailers };
      } catch (e) {
        addToast(`Failed to load diff: ${e}`, "error");
      }
    }
  }

  function statusIcon(status: string): string {
    switch (status) {
      case "A": return "+";
      case "D": return "-";
      default: return "~";
    }
  }

  function statusColor(status: string): string {
    switch (status) {
      case "A": return "var(--color-status-success)";
      case "D": return "var(--color-status-error)";
      default: return "var(--color-status-warning)";
    }
  }

  onMount(async () => {
    mounted = true;
    await loadData();
  });
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("git.title")}>
    {#snippet actions()}
      <IconButton onclick={loadData} tooltip={$_("git.refresh")}>
        <RefreshCw size={15} class={loading ? "animate-spin" : ""} />
      </IconButton>
    {/snippet}
  </TopBar>

  <div class="flex-1 overflow-hidden flex">
    {#if loading && commits.length === 0}
      <div class="flex-1 flex items-center justify-center">
        <div class="text-center">
          <Spinner size="lg" />
          <p class="text-[var(--text-caption)] text-[var(--color-text-tertiary)] mt-3">{$_("git.loading")}</p>
        </div>
      </div>
    {:else if error}
      <div class="flex-1 flex items-center justify-center p-8">
        <div class="text-center max-w-sm">
          <GitBranchIcon size={40} class="text-[var(--color-text-tertiary)] mx-auto mb-3" />
          <h3 class="text-[var(--text-subtitle)] font-semibold text-[var(--color-text-primary)] mb-1">
            {$_("git.noRepo")}
          </h3>
          <p class="text-[var(--text-caption)] text-[var(--color-text-secondary)]">
            {$_("git.noRepoDesc")}
          </p>
        </div>
      </div>
    {:else}
      <!-- Left: Status + Branches sidebar -->
      <aside class="w-64 shrink-0 border-r border-[var(--color-border-subtle)] overflow-y-auto p-4 space-y-4">
        <!-- Current Branch & Status -->
        {#if gitStatus}
          <div in:fadeScale={{ duration: 200 }}>
            <h4 class="text-[var(--text-caption)] font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider mb-2">
              {$_("git.status")}
            </h4>
            <Panel class="p-3">
              <div class="flex items-center gap-2 mb-2">
                <GitBranchIcon size={14} class="text-[var(--color-accent)]" />
                <span class="text-[var(--text-body)] font-mono font-semibold text-[var(--color-text-primary)]">
                  {gitStatus.branch}
                </span>
              </div>

              {#if gitStatus.ahead > 0 || gitStatus.behind > 0}
                <div class="flex gap-3 text-[var(--text-caption)]">
                  {#if gitStatus.ahead > 0}
                    <span class="flex items-center gap-1 text-[var(--color-status-success)]">
                      <ArrowUp size={11} /> {gitStatus.ahead}
                    </span>
                  {/if}
                  {#if gitStatus.behind > 0}
                    <span class="flex items-center gap-1 text-[var(--color-status-warning)]">
                      <ArrowDown size={11} /> {gitStatus.behind}
                    </span>
                  {/if}
                </div>
              {/if}

              {#if gitStatus.staged.length > 0 || gitStatus.unstaged.length > 0 || gitStatus.untracked.length > 0}
                <div class="mt-2 pt-2 border-t border-[var(--color-border-subtle)] space-y-1">
                  {#if gitStatus.staged.length > 0}
                    <div class="flex items-center gap-2 text-[var(--text-caption)]">
                      <span class="text-[var(--color-status-success)]">{$_("git.staged")}</span>
                      <span class="text-[var(--color-text-tertiary)]">{gitStatus.staged.length}</span>
                    </div>
                  {/if}
                  {#if gitStatus.unstaged.length > 0}
                    <div class="flex items-center gap-2 text-[var(--text-caption)]">
                      <span class="text-[var(--color-status-warning)]">{$_("git.unstaged")}</span>
                      <span class="text-[var(--color-text-tertiary)]">{gitStatus.unstaged.length}</span>
                    </div>
                  {/if}
                  {#if gitStatus.untracked.length > 0}
                    <div class="flex items-center gap-2 text-[var(--text-caption)]">
                      <span class="text-[var(--color-text-secondary)]">{$_("git.untracked")}</span>
                      <span class="text-[var(--color-text-tertiary)]">{gitStatus.untracked.length}</span>
                    </div>
                  {/if}
                </div>
              {/if}
            </Panel>
          </div>
        {/if}

        <!-- Branches -->
        {#if branches.length > 0}
          <div>
            <h4 class="text-[var(--text-caption)] font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider mb-2">
              {$_("git.branches")}
            </h4>
            <div class="space-y-1">
              {#each branches as branch}
                <div
                  class="flex items-center gap-2 px-2.5 py-1.5 rounded-[var(--radius-md)] text-[var(--text-caption)]
                    {branch.is_current ? 'bg-[var(--color-accent)]/10 text-[var(--color-accent)]' : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)]'}"
                >
                  {#if branch.is_current}
                    <Circle size={6} class="fill-current" />
                  {:else}
                    <GitBranchIcon size={12} />
                  {/if}
                  <span class="font-mono truncate">{branch.name}</span>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </aside>

      <!-- Right: Commit timeline -->
      <div class="flex-1 overflow-y-auto">
        {#if commits.length === 0}
          <div class="flex items-center justify-center h-full">
            <p class="text-[var(--color-text-tertiary)]">{$_("git.noCommits")}</p>
          </div>
        {:else}
          <div class="py-4 px-5">
            <h4 class="text-[var(--text-caption)] font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider mb-3">
              {$_("git.commits")} ({commits.length})
            </h4>

            <!-- Timeline -->
            <div class="relative">
              <!-- Vertical line -->
              <div class="absolute left-[11px] top-2 bottom-2 w-px bg-[var(--color-border-subtle)]"></div>

              {#each commits as commit, i}
                <div
                  class="relative pl-8 pb-1"
                  in:staggeredItem={{ index: Math.min(i, 15), staggerDelay: 20, duration: 200, distance: 6 }}
                >
                  <!-- Timeline dot -->
                  <div class="absolute left-[7px] top-3 w-[9px] h-[9px] rounded-full border-2 border-[var(--color-border-default)] bg-[var(--color-bg-surface)]"
                    class:border-[var(--color-accent)]={i === 0}
                    class:bg-[var(--color-accent)]={i === 0}
                  ></div>

                  <!-- Commit card -->
                  <button
                    class="w-full text-left px-3 py-2 rounded-[var(--radius-md)] transition-all duration-150 cursor-pointer
                      hover:bg-[var(--color-bg-elevated)]
                      {expandedCommit === commit.hash ? 'bg-[var(--color-bg-elevated)] border border-[var(--color-border-default)]' : ''}"
                    onclick={() => toggleCommit(commit.hash)}
                  >
                    <div class="flex items-start gap-2">
                      <div class="flex-1 min-w-0">
                        <p class="text-[var(--text-body)] text-[var(--color-text-primary)] leading-snug truncate">
                          {commit.message}
                        </p>
                        <div class="flex items-center gap-3 mt-1 text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
                          <span class="font-mono">{commit.short_hash}</span>
                          <span>{commit.author}</span>
                          <span>{timeAgo(commit.date)}</span>
                        </div>
                      </div>

                      <!-- Stats -->
                      <div class="flex items-center gap-2 shrink-0">
                        {#if commit.files_changed > 0}
                          <span class="text-[10px] text-[var(--color-text-tertiary)]">
                            {commit.files_changed} files
                          </span>
                        {/if}
                        {#if commit.insertions > 0}
                          <span class="text-[10px] text-[var(--color-status-success)]">+{commit.insertions}</span>
                        {/if}
                        {#if commit.deletions > 0}
                          <span class="text-[10px] text-[var(--color-status-error)]">-{commit.deletions}</span>
                        {/if}
                        {#if expandedCommit === commit.hash}
                          <ChevronDown size={14} class="text-[var(--color-text-tertiary)]" />
                        {:else}
                          <ChevronRight size={14} class="text-[var(--color-text-tertiary)]" />
                        {/if}
                      </div>
                    </div>

                    <!-- Expanded: file list -->
                    {#if expandedCommit === commit.hash}
                      <div class="mt-2 pt-2 border-t border-[var(--color-border-subtle)]">
                        {#if commitFiles[commit.hash]}
                          <div class="space-y-0.5">
                            {#each commitFiles[commit.hash] as file}
                              <div class="flex items-center gap-2 py-0.5 text-[var(--text-caption)]">
                                <span
                                  class="w-4 text-center font-mono font-semibold text-[10px]"
                                  style="color: {statusColor(file.status)}"
                                >
                                  {statusIcon(file.status)}
                                </span>
                                <FileCode size={12} class="text-[var(--color-text-tertiary)] shrink-0" />
                                <span class="font-mono text-[var(--color-text-primary)] truncate flex-1">{file.path}</span>
                                <div class="flex gap-1.5 shrink-0">
                                  {#if file.insertions > 0}
                                    <span class="text-[var(--color-status-success)]">+{file.insertions}</span>
                                  {/if}
                                  {#if file.deletions > 0}
                                    <span class="text-[var(--color-status-error)]">-{file.deletions}</span>
                                  {/if}
                                </div>
                              </div>
                            {/each}
                          </div>
                        {:else}
                          <div class="flex items-center justify-center py-2">
                            <Spinner size="sm" />
                          </div>
                        {/if}

                        {#if commitTrailers[commit.hash] && Object.keys(commitTrailers[commit.hash]).length > 0}
                          <div class="flex flex-wrap gap-1.5 mt-2 pt-2 border-t border-[var(--color-border-subtle)]">
                            {#each Object.entries(commitTrailers[commit.hash]) as [key, value]}
                              {@const color = key === 'Confidence' ? 'var(--color-status-success)' :
                                key === 'Constraint' ? 'var(--color-status-info)' :
                                key === 'Rejected' || key === 'Not-tested' ? 'var(--color-status-error)' :
                                'var(--color-status-warning)'}
                              <span
                                class="text-[8px] font-bold px-1.5 py-0.5 rounded-[3px] tracking-wider"
                                style="background: color-mix(in srgb, {color} 15%, transparent); color: {color}"
                              >
                                {key}: {value}
                              </span>
                            {/each}
                          </div>
                        {/if}
                      </div>
                    {/if}
                  </button>
                </div>
              {/each}
            </div>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>
