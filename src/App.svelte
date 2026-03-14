<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import MainLayout from "$lib/components/layout/MainLayout.svelte";
  import SplashScreen from "$lib/components/SplashScreen.svelte";
  import {
    startSessionListeners,
    stopSessionListeners,
  } from "$lib/stores/session";
  import {
    initKeyboardShortcuts,
    destroyKeyboardShortcuts,
  } from "$lib/utils/keyboard";

  let showSplash = $state(true);

  function handleSplashReady() {
    showSplash = false;
  }

  onMount(() => {
    startSessionListeners();
    initKeyboardShortcuts();
  });

  onDestroy(() => {
    stopSessionListeners();
    destroyKeyboardShortcuts();
  });
</script>

{#if showSplash}
  <SplashScreen onReady={handleSplashReady} />
{:else}
  <div class="animate-[dashboardFadeIn_400ms_ease-out_both]">
    <MainLayout />
  </div>
{/if}

<style>
  @keyframes dashboardFadeIn {
    from {
      opacity: 0;
      transform: translateY(12px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
