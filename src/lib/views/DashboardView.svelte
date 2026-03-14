<script lang="ts">
  import { _ } from "svelte-i18n";
  import TopBar from "$lib/components/layout/TopBar.svelte";
</script>

<div class="flex flex-col h-full">
  <TopBar title={$_("nav.dashboard")} />
  <div class="flex-1 flex items-center justify-center p-8">
    <div class="text-center">
      <h2 class="text-[24px] font-semibold text-[var(--color-text-primary)] mb-2">
        {$_("dashboard.welcome")}
      </h2>
      <p class="text-[var(--color-text-secondary)]">
        {$_("dashboard.subtitle")}
      </p>
    </div>
  </div>
</div>
