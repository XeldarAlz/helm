<script lang="ts">
  interface Props {
    value?: number;
    min?: number;
    max?: number;
    step?: number;
    disabled?: boolean;
    onchange?: (value: number) => void;
    class?: string;
  }

  let {
    value = $bindable(0),
    min = 0,
    max = 100,
    step = 1,
    disabled = false,
    onchange,
    class: className = "",
  }: Props = $props();

  function handleInput(e: Event) {
    const target = e.target as HTMLInputElement;
    value = Number(target.value);
    onchange?.(value);
  }
</script>

<input
  type="range"
  {min}
  {max}
  {step}
  {value}
  {disabled}
  oninput={handleInput}
  class="w-full h-1.5 rounded-[var(--radius-full)] appearance-none cursor-pointer bg-[var(--color-bg-overlay)]
    [&::-webkit-slider-thumb]:appearance-none [&::-webkit-slider-thumb]:w-4 [&::-webkit-slider-thumb]:h-4
    [&::-webkit-slider-thumb]:rounded-full [&::-webkit-slider-thumb]:bg-[var(--color-accent)]
    [&::-webkit-slider-thumb]:shadow-[var(--shadow-sm)] [&::-webkit-slider-thumb]:cursor-pointer
    [&::-webkit-slider-thumb]:transition-transform [&::-webkit-slider-thumb]:duration-150
    [&::-webkit-slider-thumb]:hover:scale-110
    {disabled ? 'opacity-50 cursor-not-allowed' : ''} {className}"
/>
