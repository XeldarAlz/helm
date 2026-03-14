<script lang="ts">
  import { _ } from "svelte-i18n";
  import { onMount } from "svelte";
  import { addToast } from "$lib/stores/toasts";
  import { WifiOff } from "lucide-svelte";

  let isOnline = $state(true);
  let wasOffline = $state(false);

  onMount(() => {
    isOnline = navigator.onLine;
    wasOffline = !isOnline;

    function handleOnline() {
      isOnline = true;
      if (wasOffline) {
        addToast($_("network.reconnected"), "success");
        wasOffline = false;
      }
    }

    function handleOffline() {
      isOnline = false;
      wasOffline = true;
    }

    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    return () => {
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
    };
  });
</script>

{#if !isOnline}
  <div
    class="flex items-center justify-center gap-2 px-3 py-1.5 text-[var(--text-caption)]
      font-medium bg-[var(--color-status-warning)]/10 text-[var(--color-status-warning)]
      border-b border-[var(--color-status-warning)]/20 shrink-0"
  >
    <WifiOff size={14} />
    <span>{$_("network.offline")}</span>
  </div>
{/if}
