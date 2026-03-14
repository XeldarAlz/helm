<script lang="ts">
  interface Props {
    value?: string;
    placeholder?: string;
    type?: "text" | "password" | "email" | "url";
    disabled?: boolean;
    oninput?: (value: string) => void;
    class?: string;
  }

  let {
    value = $bindable(""),
    placeholder = "",
    type = "text",
    disabled = false,
    oninput,
    class: className = "",
  }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = target.value;
    oninput?.(value);
  }
</script>

<input
  {type}
  {value}
  {placeholder}
  {disabled}
  oninput={handleInput}
  class="w-full h-9 px-3 text-[13px] bg-[var(--color-bg-surface)] text-[var(--color-text-primary)] placeholder-[var(--color-text-tertiary)]
    border border-[var(--color-border-default)] rounded-[var(--radius-md)]
    focus:outline-none focus:border-[var(--color-border-active)] focus:shadow-[var(--glow-accent)]
    transition-all duration-150
    {disabled ? 'opacity-50 cursor-not-allowed' : ''} {className}"
/>
