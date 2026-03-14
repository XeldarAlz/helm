<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    size?: "sm" | "md" | "lg";
    variant?: "ghost" | "surface";
    tooltip?: string;
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    class?: string;
    children: Snippet;
  }

  let {
    size = "md",
    variant = "ghost",
    tooltip,
    disabled = false,
    onclick,
    class: className = "",
    children,
  }: Props = $props();

  const sizeClasses: Record<string, string> = {
    sm: "w-7 h-7 [&>*]:w-3.5 [&>*]:h-3.5",
    md: "w-9 h-9 [&>*]:w-4.5 [&>*]:h-4.5",
    lg: "w-11 h-11 [&>*]:w-5.5 [&>*]:h-5.5",
  };

  const variantClasses: Record<string, string> = {
    ghost:
      "bg-transparent hover:bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]",
    surface:
      "bg-[var(--color-bg-surface)] hover:bg-[var(--color-bg-elevated)] text-[var(--color-text-secondary)] hover:text-[var(--color-text-primary)]",
  };
</script>

<button
  class="inline-flex items-center justify-center rounded-[var(--radius-md)] transition-all duration-150 cursor-pointer {sizeClasses[size]} {variantClasses[variant]} {disabled ? 'opacity-50 cursor-not-allowed' : ''} {className}"
  title={tooltip}
  {disabled}
  {onclick}
>
  {@render children()}
</button>
