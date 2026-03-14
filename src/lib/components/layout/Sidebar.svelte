<script lang="ts">
  import { _ } from "svelte-i18n";
  import { activeView, sidebarCollapsed, type ViewName } from "$lib/stores/ui";
  import { currentSession } from "$lib/stores/session";
  import { pipelineState } from "$lib/stores/pipeline";
  import Divider from "$lib/components/common/Divider.svelte";
  import {
    LayoutDashboard,
    MessageSquare,
    Bot,
    FileText,
    FolderCode,
    GitBranch,
    History,
    Settings,
    ChevronLeft,
    ChevronRight,
  } from "lucide-svelte";

  interface NavItem {
    id: ViewName;
    labelKey: string;
    icon: typeof LayoutDashboard;
    badge?: () => string | null;
    disabled?: () => boolean;
  }

  const navItems: NavItem[] = [
    { id: "dashboard", labelKey: "nav.dashboard", icon: LayoutDashboard },
    {
      id: "chat",
      labelKey: "nav.chat",
      icon: MessageSquare,
      badge: () => ($currentSession?.status === "active" ? "" : null),
    },
    {
      id: "orchestration",
      labelKey: "nav.orchestration",
      icon: Bot,
      disabled: () => !$pipelineState.progressExists,
    },
    { id: "documents", labelKey: "nav.documents", icon: FileText },
    { id: "code", labelKey: "nav.code", icon: FolderCode },
    { id: "git", labelKey: "nav.git", icon: GitBranch },
    { id: "history", labelKey: "nav.history", icon: History },
  ];

  function navigate(view: ViewName) {
    activeView.set(view);
  }
</script>

<aside
  class="flex flex-col h-full bg-[var(--color-bg-surface)] border-r border-[var(--color-border-subtle)] transition-[width] duration-200
    {$sidebarCollapsed ? 'w-16' : 'w-52'}"
>
  <!-- Logo area -->
  <div class="flex items-center h-12 px-4 gap-2">
    {#if !$sidebarCollapsed}
      <span class="text-[15px] font-semibold text-[var(--color-accent)]">
        {$_("app.name")}
      </span>
    {/if}
  </div>

  <!-- Navigation -->
  <nav class="flex-1 px-2 py-2 space-y-0.5 overflow-y-auto">
    {#each navItems as item}
      {@const isActive = $activeView === item.id}
      {@const isDisabled = item.disabled?.() ?? false}
      <button
        class="flex items-center gap-3 w-full px-3 py-2 rounded-[var(--radius-md)] transition-all duration-150 cursor-pointer text-left
          {isActive
            ? 'bg-[var(--color-bg-overlay)] text-[var(--color-text-primary)] border-l-2 border-[var(--color-accent)]'
            : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)] hover:text-[var(--color-text-primary)]'}
          {isDisabled ? 'opacity-40 pointer-events-none' : ''}"
        onclick={() => navigate(item.id)}
        disabled={isDisabled}
      >
        <svelte:component this={item.icon} size={18} />
        {#if !$sidebarCollapsed}
          <span class="text-[13px] font-medium truncate">{$_(item.labelKey)}</span>
        {/if}
        {#if item.badge?.() !== undefined && item.badge?.() !== null}
          <span class="w-2 h-2 rounded-full bg-[var(--color-accent)] animate-pulse ml-auto"></span>
        {/if}
      </button>
    {/each}

    <Divider class="my-2" />

    <button
      class="flex items-center gap-3 w-full px-3 py-2 rounded-[var(--radius-md)] transition-all duration-150 cursor-pointer text-left
        {$activeView === 'settings'
          ? 'bg-[var(--color-bg-overlay)] text-[var(--color-text-primary)]'
          : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)] hover:text-[var(--color-text-primary)]'}"
      onclick={() => navigate("settings")}
    >
      <Settings size={18} />
      {#if !$sidebarCollapsed}
        <span class="text-[13px] font-medium">{$_("nav.settings")}</span>
      {/if}
    </button>
  </nav>

  <!-- Collapse toggle -->
  <div class="px-2 py-2">
    <button
      class="flex items-center justify-center w-full py-1.5 rounded-[var(--radius-md)] text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-elevated)] transition-all duration-150 cursor-pointer"
      onclick={() => sidebarCollapsed.update((v) => !v)}
    >
      {#if $sidebarCollapsed}
        <ChevronRight size={16} />
      {:else}
        <ChevronLeft size={16} />
      {/if}
    </button>
  </div>
</aside>
