<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    rows?: number;
    maxRows?: number;
    disabled?: boolean;
    oninput?: (value: string) => void;
    onkeydown?: (e: KeyboardEvent) => void;
    class?: string;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    rows = 1,
    maxRows = 6,
    disabled = false,
    oninput,
    onkeydown,
    class: className = "",
  }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLTextAreaElement;
    value = target.value;

    // Auto-grow
    target.style.height = "auto";
    const lineHeight = parseInt(getComputedStyle(target).lineHeight);
    const maxHeight = lineHeight * maxRows;
    target.style.height = Math.min(target.scrollHeight, maxHeight) + "px";

    oninput?.(value);
  }
</script>

<textarea
  {value}
  {placeholder}
  {rows}
  {disabled}
  oninput={handleInput}
  {onkeydown}
  class="w-full px-3 py-2 text-[13px] bg-[var(--color-bg-surface)] text-[var(--color-text-primary)] placeholder-[var(--color-text-tertiary)]
    border border-[var(--color-border-default)] rounded-[var(--radius-md)] resize-none
    focus:outline-none focus:border-[var(--color-border-active)] focus:shadow-[var(--glow-accent)]
    transition-all duration-150
    {disabled ? 'opacity-50 cursor-not-allowed' : ''} {className}"
></textarea>
