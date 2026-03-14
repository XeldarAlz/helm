<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { checkCli, getPipelineState, getSettings } from "$lib/utils/ipc";
  import { pipelineState } from "$lib/stores/pipeline";
  import { settings } from "$lib/stores/settings";

  interface Props {
    onReady: () => void;
  }

  let { onReady }: Props = $props();

  let progress = $state(0);
  let statusText = $state("");
  let logoVisible = $state(false);
  let textVisible = $state(false);
  let taglineVisible = $state(false);
  let barVisible = $state(false);
  let fadingOut = $state(false);

  async function initialize() {
    // Step 1: Check CLI
    statusText = $_("splash.checkingCli");
    progress = 15;
    try {
      await checkCli();
    } catch {
      // Non-fatal — continue
    }
    progress = 30;

    // Step 2: Load settings
    statusText = $_("splash.loadingSettings");
    try {
      const s = await getSettings();
      settings.set(s);
    } catch {
      // Use defaults
    }
    progress = 55;

    // Step 3: Detect project / load pipeline state
    statusText = $_("splash.loadingState");
    try {
      const ps = await getPipelineState();
      pipelineState.set({
        gddExists: ps.gddExists ?? (ps as any).gdd_exists ?? false,
        tddExists: ps.tddExists ?? (ps as any).tdd_exists ?? false,
        workflowExists: ps.workflowExists ?? (ps as any).workflow_exists ?? false,
        progressExists: ps.progressExists ?? (ps as any).progress_exists ?? false,
        currentPhase: ps.currentPhase ?? (ps as any).current_phase ?? "none",
      });
    } catch {
      // Use defaults
    }
    progress = 85;

    // Step 4: Ready
    statusText = $_("splash.ready");
    progress = 100;

    // Brief pause at 100%, then transition out
    await delay(400);
    fadingOut = true;
    await delay(400);
    onReady();
  }

  function delay(ms: number): Promise<void> {
    return new Promise((r) => setTimeout(r, ms));
  }

  onMount(async () => {
    // Stagger splash entrance
    await delay(100);
    logoVisible = true;
    await delay(300);
    textVisible = true;
    await delay(200);
    taglineVisible = true;
    await delay(200);
    barVisible = true;
    await delay(100);

    // Start actual initialization
    await initialize();
  });
</script>

<div
  class="fixed inset-0 z-50 flex flex-col items-center justify-center bg-[var(--color-bg-deep)] transition-all duration-400"
  class:opacity-0={fadingOut}
  class:scale-[1.02]={fadingOut}
>
  <!-- Logo mark -->
  <div
    class="mb-6 transition-all duration-700 ease-out"
    class:opacity-0={!logoVisible}
    class:scale-[0.8]={!logoVisible}
    class:opacity-100={logoVisible}
    class:scale-100={logoVisible}
  >
    <svg width="72" height="72" viewBox="0 0 72 72" fill="none">
      <!-- Helm wheel -->
      <circle cx="36" cy="36" r="28" stroke="var(--color-accent)" stroke-width="2.5" opacity="0.3" />
      <circle cx="36" cy="36" r="20" stroke="var(--color-accent)" stroke-width="2" opacity="0.2" />
      <circle cx="36" cy="36" r="8" fill="var(--color-accent)" opacity="0.9" />
      <!-- Spokes -->
      <line x1="36" y1="8" x2="36" y2="28" stroke="var(--color-accent)" stroke-width="2.5" stroke-linecap="round" />
      <line x1="36" y1="44" x2="36" y2="64" stroke="var(--color-accent)" stroke-width="2.5" stroke-linecap="round" />
      <line x1="8" y1="36" x2="28" y2="36" stroke="var(--color-accent)" stroke-width="2.5" stroke-linecap="round" />
      <line x1="44" y1="36" x2="64" y2="36" stroke="var(--color-accent)" stroke-width="2.5" stroke-linecap="round" />
      <!-- Diagonal spokes -->
      <line x1="15.2" y1="15.2" x2="22.3" y2="22.3" stroke="var(--color-accent)" stroke-width="2" stroke-linecap="round" opacity="0.6" />
      <line x1="49.7" y1="49.7" x2="56.8" y2="56.8" stroke="var(--color-accent)" stroke-width="2" stroke-linecap="round" opacity="0.6" />
      <line x1="56.8" y1="15.2" x2="49.7" y2="22.3" stroke="var(--color-accent)" stroke-width="2" stroke-linecap="round" opacity="0.6" />
      <line x1="22.3" y1="49.7" x2="15.2" y2="56.8" stroke="var(--color-accent)" stroke-width="2" stroke-linecap="round" opacity="0.6" />
    </svg>
  </div>

  <!-- App name -->
  <h1
    class="text-[var(--text-display)] font-bold text-[var(--color-accent)] tracking-wider mb-2 transition-all duration-500 ease-out"
    class:opacity-0={!textVisible}
    class:translate-y-2={!textVisible}
    class:opacity-100={textVisible}
    class:translate-y-0={textVisible}
  >
    {$_("app.name")}
  </h1>

  <!-- Tagline -->
  <p
    class="text-[var(--text-subtitle)] text-[var(--color-text-secondary)] mb-10 transition-all duration-500 ease-out"
    class:opacity-0={!taglineVisible}
    class:translate-y-2={!taglineVisible}
    class:opacity-100={taglineVisible}
    class:translate-y-0={taglineVisible}
  >
    {$_("app.tagline")}
  </p>

  <!-- Loading bar -->
  <div
    class="w-64 transition-all duration-400 ease-out"
    class:opacity-0={!barVisible}
    class:translate-y-2={!barVisible}
    class:opacity-100={barVisible}
    class:translate-y-0={barVisible}
  >
    <div class="w-full h-1 rounded-[var(--radius-full)] bg-[var(--color-bg-overlay)] overflow-hidden">
      <div
        class="h-full rounded-[var(--radius-full)] bg-[var(--color-accent)] transition-[width] duration-500 ease-out"
        style="width: {progress}%"
      ></div>
    </div>

    <!-- Status text -->
    <p class="text-[var(--text-caption)] text-[var(--color-text-tertiary)] text-center mt-3">
      {statusText}
    </p>
  </div>
</div>
