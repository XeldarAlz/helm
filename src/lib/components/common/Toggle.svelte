<script lang="ts">
  interface Props {
    checked?: boolean;
    disabled?: boolean;
    onchange?: (checked: boolean) => void;
    class?: string;
  }

  let {
    checked = $bindable(false),
    disabled = false,
    onchange,
    class: className = "",
  }: Props = $props();

  function toggle() {
    if (disabled) return;
    checked = !checked;
    onchange?.(checked);
  }
</script>

<button
  role="switch"
  aria-checked={checked}
  class="relative inline-flex h-5 w-9 shrink-0 rounded-[var(--radius-full)] transition-colors duration-150 cursor-pointer
    {checked ? 'bg-[var(--color-accent)]' : 'bg-[var(--color-bg-overlay)]'}
    {disabled ? 'opacity-50 cursor-not-allowed' : ''} {className}"
  {disabled}
  onclick={toggle}
>
  <span
    class="pointer-events-none inline-block h-4 w-4 rounded-full bg-white shadow-sm transition-transform duration-150
      {checked ? 'translate-x-4.5' : 'translate-x-0.5'} mt-0.5"
  ></span>
</button>
