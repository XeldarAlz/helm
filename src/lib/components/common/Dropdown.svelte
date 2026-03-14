<script lang="ts">
  interface Option {
    value: string;
    label: string;
  }

  interface Props {
    options: Option[];
    value?: string;
    placeholder?: string;
    disabled?: boolean;
    onchange?: (value: string) => void;
    class?: string;
  }

  let {
    options,
    value = $bindable(""),
    placeholder = "Select...",
    disabled = false,
    onchange,
    class: className = "",
  }: Props = $props();

  function handleChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    value = target.value;
    onchange?.(value);
  }
</script>

<select
  {value}
  {disabled}
  onchange={handleChange}
  class="w-full h-9 px-3 text-[13px] bg-[var(--color-bg-surface)] text-[var(--color-text-primary)]
    border border-[var(--color-border-default)] rounded-[var(--radius-md)]
    focus:outline-none focus:border-[var(--color-border-active)] focus:shadow-[var(--glow-accent)]
    transition-all duration-150 cursor-pointer appearance-none
    bg-[url('data:image/svg+xml;charset=UTF-8,%3Csvg%20xmlns%3D%22http%3A%2F%2Fwww.w3.org%2F2000%2Fsvg%22%20width%3D%2212%22%20height%3D%2212%22%20viewBox%3D%220%200%2024%2024%22%20fill%3D%22none%22%20stroke%3D%22%238B949E%22%20stroke-width%3D%222%22%3E%3Cpath%20d%3D%22M6%209l6%206%206-6%22%2F%3E%3C%2Fsvg%3E')]
    bg-no-repeat bg-[right_12px_center]
    {disabled ? 'opacity-50 cursor-not-allowed' : ''} {className}"
>
  {#if placeholder}
    <option value="" disabled>{placeholder}</option>
  {/if}
  {#each options as opt}
    <option value={opt.value}>{opt.label}</option>
  {/each}
</select>
