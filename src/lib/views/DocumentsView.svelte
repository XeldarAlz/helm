<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";
  import { pipelineState } from "$lib/stores/pipeline";
  import {
    documents,
    activeDocTab,
    docSearchTerm,
    parseSections,
    type DocName,
    type DocSection,
  } from "$lib/stores/documents";
  import { readDocument, startWatching } from "$lib/utils/ipc";
  import { addToast } from "$lib/stores/toasts";
  import { fadeScale } from "$lib/animations";
  import TopBar from "$lib/components/layout/TopBar.svelte";
  import SplitPane from "$lib/components/layout/SplitPane.svelte";
  import DocTabs from "$lib/components/documents/DocTabs.svelte";
  import SectionNav from "$lib/components/documents/SectionNav.svelte";
  import DocSearch from "$lib/components/documents/DocSearch.svelte";
  import MarkdownRenderer from "$lib/components/documents/MarkdownRenderer.svelte";
  import Badge from "$lib/components/common/Badge.svelte";
  import Spinner from "$lib/components/common/Spinner.svelte";
  import { FileText, Radio } from "lucide-svelte";

  let content = $state("");
  let sections = $state<DocSection[]>([]);
  let activeSection = $state("");
  let searchTerm = $state("");
  let matchCount = $state(0);
  let loading = $state(false);
  let watching = $state(false);
  let scrollContainer = $state<HTMLElement | null>(null);

  let unlistenDocUpdated: (() => void) | null = null;
  let unlistenProgressUpdated: (() => void) | null = null;

  const docExists = $derived.by(() => {
    const tab = $activeDocTab;
    switch (tab) {
      case "GDD": return $pipelineState.gddExists;
      case "TDD": return $pipelineState.tddExists;
      case "WORKFLOW": return $pipelineState.workflowExists;
      case "PROGRESS": return $pipelineState.progressExists;
      case "ACTIVITY_LOG": return true;
      case "CATCH_UP": return $pipelineState.catchUpExists;
      default: return false;
    }
  });

  async function loadDocument(name: DocName) {
    loading = true;
    try {
      const doc = await readDocument(name);
      content = doc;
      sections = parseSections(doc);
      documents.update((d) => ({ ...d, [name]: doc }));
    } catch {
      content = "";
      sections = [];
    } finally {
      loading = false;
    }
  }

  // Count search matches in content
  function countMatches(text: string, term: string): number {
    if (!term || term.length < 2) return 0;
    const escaped = term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    const regex = new RegExp(escaped, "gi");
    return (text.match(regex) || []).length;
  }

  // React to tab changes
  $effect(() => {
    const tab = $activeDocTab;
    loadDocument(tab);
  });

  // React to search term changes
  $effect(() => {
    matchCount = countMatches(content, searchTerm);
  });

  function scrollToSection(id: string) {
    if (id === "top") {
      scrollContainer?.scrollTo({ top: 0, behavior: "smooth" });
      return;
    }
    const el = scrollContainer?.querySelector(`#${CSS.escape(id)}`);
    if (el) {
      el.scrollIntoView({ behavior: "smooth", block: "start" });
      activeSection = id;
    }
  }

  // Scroll spy: track which section is visible
  function handleScroll() {
    if (!scrollContainer || sections.length === 0) return;
    const headings = scrollContainer.querySelectorAll(".doc-heading");
    let current = "";
    for (const heading of headings) {
      const rect = heading.getBoundingClientRect();
      const containerRect = scrollContainer.getBoundingClientRect();
      if (rect.top - containerRect.top < 80) {
        current = heading.id;
      }
    }
    if (current) activeSection = current;
  }

  onMount(async () => {
    // Start file watcher
    try {
      await startWatching();
      watching = true;
    } catch {
      // Watcher may fail if no project dir set
    }

    // Listen for doc change events
    unlistenDocUpdated = await listen<{ name: string }>("document-updated", (event) => {
      const changedName = event.payload.name.replace(".md", "");
      if (changedName === $activeDocTab) {
        loadDocument($activeDocTab);
      }
      addToast($_("documents.updated"), "info", 2000);
    });

    unlistenProgressUpdated = await listen<{ name: string }>("progress-updated", () => {
      if ($activeDocTab === "PROGRESS") {
        loadDocument("PROGRESS");
      }
    });
  });

  onDestroy(() => {
    unlistenDocUpdated?.();
    unlistenProgressUpdated?.();
  });
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("documents.title")}>
    {#snippet actions()}
      {#if watching}
        <Badge variant="success" size="sm" pulse>
          <span class="flex items-center gap-1">
            <Radio size={10} />
            {$_("documents.watching")}
          </span>
        </Badge>
      {/if}
    {/snippet}
  </TopBar>

  <DocTabs />

  <div class="flex-1 overflow-hidden">
    {#if loading}
      <div class="flex items-center justify-center h-full">
        <Spinner size="lg" />
      </div>
    {:else if !content}
      <!-- Empty state: document not created -->
      <div class="flex flex-col items-center justify-center h-full gap-4 px-8" in:fadeScale={{ duration: 200 }}>
        <div class="flex items-center justify-center w-16 h-16 rounded-[var(--radius-xl)] bg-[var(--color-bg-elevated)]">
          <FileText size={28} class="text-[var(--color-text-tertiary)]" />
        </div>
        <div class="text-center max-w-sm">
          <p class="text-[var(--text-body)] text-[var(--color-text-secondary)] mb-1">
            {$_("documents.tabs." + $activeDocTab)}
          </p>
          <p class="text-[var(--text-caption)] text-[var(--color-text-tertiary)] leading-relaxed">
            {$_("documents.noContent")}
          </p>
        </div>
      </div>
    {:else}
      <!-- Document content with section nav -->
      {#snippet left()}
        <div class="h-full flex flex-col bg-[var(--color-bg-surface)] border-r border-[var(--color-border-subtle)]">
          <div class="px-2 pt-3 pb-2">
            <DocSearch bind:value={searchTerm} {matchCount} onchange={(v) => { searchTerm = v; }} />
          </div>
          <SectionNav {sections} {activeSection} onSelect={scrollToSection} />
        </div>
      {/snippet}

      {#snippet right()}
        <div
          bind:this={scrollContainer}
          class="h-full overflow-y-auto px-8 py-6"
          onscroll={handleScroll}
        >
          <div class="max-w-3xl mx-auto">
            <MarkdownRenderer {content} searchTerm={searchTerm} />
          </div>
        </div>
      {/snippet}

      <SplitPane direction="horizontal" initialSplit={200} minSize={140} {left} {right} />
    {/if}
  </div>
</div>
