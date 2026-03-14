<script lang="ts">
  import { _ } from "svelte-i18n";
  import { Send } from "lucide-svelte";

  interface Props {
    disabled?: boolean;
    onsend: (message: string) => void;
  }

  let { disabled = false, onsend }: Props = $props();

  let value = $state("");
  let textareaEl = $state<HTMLTextAreaElement | null>(null);

  const canSend = $derived(value.trim().length > 0 && !disabled);

  function send() {
    if (!canSend) return;
    onsend(value.trim());
    value = "";
    resetHeight();
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.metaKey || e.ctrlKey) && e.key === "Enter") {
      e.preventDefault();
      send();
    }
  }

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    value = target.value;
    autoGrow(target);
  }

  function autoGrow(el: HTMLTextAreaElement) {
    el.style.height = "auto";
    const lineHeight = parseInt(getComputedStyle(el).lineHeight) || 20;
    const maxHeight = lineHeight * 6;
    el.style.height = Math.min(el.scrollHeight, maxHeight) + "px";
  }

  function resetHeight() {
    if (textareaEl) {
      textareaEl.style.height = "auto";
    }
  }

  /** Focus the input — called externally. */
  export function focus() {
    textareaEl?.focus();
  }
</script>

<div class="flex items-end gap-2 px-4 py-3 border-t border-[var(--color-border-subtle)] bg-[var(--color-bg-base)]">
  <textarea
    bind:this={textareaEl}
    {value}
    placeholder={disabled ? $_("chat.agentTyping") : $_("chat.inputPlaceholder")}
    rows={1}
    {disabled}
    oninput={handleInput}
    onkeydown={handleKeydown}
    class="flex-1 px-3 py-2 text-[13px] leading-[20px] bg-[var(--color-bg-surface)] text-[var(--color-text-primary)]
      placeholder-[var(--color-text-tertiary)] border border-[var(--color-border-default)]
      rounded-[var(--radius-lg)] resize-none
      focus:outline-none focus:border-[var(--color-border-active)] focus:shadow-[var(--glow-accent)]
      transition-all duration-150
      {disabled ? 'opacity-50 cursor-not-allowed' : ''}"
  ></textarea>

  <button
    class="flex items-center justify-center w-9 h-9 rounded-[var(--radius-md)] shrink-0
      transition-all duration-150 cursor-pointer
      {canSend
        ? 'bg-[var(--color-accent)] text-[var(--color-text-inverse)] hover:bg-[var(--color-accent-hover)] hover:shadow-[var(--glow-accent)] active:scale-[0.95]'
        : 'bg-[var(--color-bg-surface)] text-[var(--color-text-tertiary)] cursor-not-allowed'}"
    disabled={!canSend}
    onclick={send}
    title="{$_('chat.send')} (⌘↩)"
  >
    <Send size={16} />
  </button>
</div>
