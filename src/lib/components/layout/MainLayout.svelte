<script lang="ts">
  import { activeView } from "$lib/stores/ui";
  import Sidebar from "./Sidebar.svelte";
  import ToastContainer from "$lib/components/common/ToastContainer.svelte";
  import NetworkBanner from "$lib/components/common/NetworkBanner.svelte";
  import DashboardView from "$lib/views/DashboardView.svelte";
  import ChatSessionView from "$lib/views/ChatSessionView.svelte";
  import OrchestrationView from "$lib/views/OrchestrationView.svelte";
  import DocumentsView from "$lib/views/DocumentsView.svelte";
  import CodeBrowserView from "$lib/views/CodeBrowserView.svelte";
  import GitTimelineView from "$lib/views/GitTimelineView.svelte";
  import SessionHistoryView from "$lib/views/SessionHistoryView.svelte";
  import SettingsView from "$lib/views/SettingsView.svelte";

  const viewComponents = {
    dashboard: DashboardView,
    chat: ChatSessionView,
    orchestration: OrchestrationView,
    documents: DocumentsView,
    code: CodeBrowserView,
    git: GitTimelineView,
    history: SessionHistoryView,
    settings: SettingsView,
  } as const;
</script>

<div class="flex h-screen w-screen bg-[var(--color-bg-base)] text-[var(--color-text-primary)] overflow-hidden">
  <Sidebar />

  <main class="flex-1 flex flex-col overflow-hidden">
    <NetworkBanner />
    <div class="flex-1 overflow-hidden">
      <svelte:component this={viewComponents[$activeView]} />
    </div>
  </main>

  <ToastContainer />
</div>
