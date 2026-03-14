<script lang="ts">
  import { toasts, dismissToast } from "$lib/stores/toasts";
  import Toast from "./Toast.svelte";
</script>

<div class="fixed top-4 right-4 z-[100] flex flex-col gap-2 max-w-sm">
  {#each $toasts.slice(0, 3) as toast (toast.id)}
    <Toast
      variant={toast.variant}
      message={toast.message}
      action={toast.action}
      ondismiss={() => dismissToast(toast.id)}
    />
  {/each}
</div>
