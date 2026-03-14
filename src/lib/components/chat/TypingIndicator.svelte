<script lang="ts">
  import { _ } from "svelte-i18n";
</script>

<div class="flex items-center gap-2 px-4 py-3">
  <div class="flex items-center gap-1">
    <span
      class="w-1.5 h-1.5 rounded-full bg-[var(--color-accent)] animate-[typing-dot_1.4s_ease-in-out_infinite]"
    ></span>
    <span
      class="w-1.5 h-1.5 rounded-full bg-[var(--color-accent)] animate-[typing-dot_1.4s_ease-in-out_0.2s_infinite]"
    ></span>
    <span
      class="w-1.5 h-1.5 rounded-full bg-[var(--color-accent)] animate-[typing-dot_1.4s_ease-in-out_0.4s_infinite]"
    ></span>
  </div>
  <span class="text-[var(--text-caption)] text-[var(--color-text-tertiary)]">
    {$_("chat.typing")}
  </span>
</div>

<style>
  @keyframes typing-dot {
    0%,
    60%,
    100% {
      opacity: 0.3;
      transform: scale(1);
    }
    30% {
      opacity: 1;
      transform: scale(1.2);
    }
  }
</style>
