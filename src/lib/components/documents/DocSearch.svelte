<script lang="ts">
  import { _ } from "svelte-i18n";
  import { Search, X } from "lucide-svelte";

  interface Props {
    value: string;
    matchCount?: number;
    onchange: (value: string) => void;
  }

  let { value = $bindable(""), matchCount = 0, onchange }: Props = $props();

  let inputEl = $state<HTMLInputElement | null>(null);

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value;
    onchange(value);
  }

  function clear() {
    value = "";
    onchange("");
    inputEl?.focus();
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === "Escape") {
      clear();
    }
  }
</script>

<div class="relative flex items-center">
  <Search size={14} class="absolute left-2.5 text-[var(--color-text-tertiary)] pointer-events-none" />
  <input
    bind:this={inputEl}
    type="text"
    {value}
    placeholder={$_("documents.search")}
    oninput={handleInput}
    onkeydown={handleKeydown}
    class="w-full h-8 pl-8 pr-16 text-[12px] bg-[var(--color-bg-surface)] text-[var(--color-text-primary)] placeholder-[var(--color-text-tertiary)]
      border border-[var(--color-border-subtle)] rounded-[var(--radius-md)]
      focus:outline-none focus:border-[var(--color-border-active)]
      transition-colors duration-150"
  />

  {#if value}
    <div class="absolute right-2 flex items-center gap-1.5">
      <span class="text-[10px] text-[var(--color-text-tertiary)]">
        {matchCount}
      </span>
      <button
        onclick={clear}
        class="p-0.5 text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)] cursor-pointer"
      >
        <X size={12} />
      </button>
    </div>
  {/if}
</div>
