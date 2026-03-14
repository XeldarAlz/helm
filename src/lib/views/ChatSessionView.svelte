<script lang="ts">
  import { _ } from "svelte-i18n";
  import TopBar from "$lib/components/layout/TopBar.svelte";
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("nav.chat")} />
  <div class="flex-1 flex items-center justify-center p-8">
    <p class="text-[var(--color-text-secondary)]">Chat session view — coming in Phase 4</p>
  </div>
</div>
