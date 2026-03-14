<script lang="ts">
  import type { Snippet } from "svelte";
  import Spinner from "./Spinner.svelte";

  interface Props {
    variant?: "primary" | "secondary" | "ghost" | "danger";
    size?: "sm" | "md" | "lg";
    icon?: Snippet;
    loading?: boolean;
    disabled?: boolean;
    fullWidth?: boolean;
    onclick?: (e: MouseEvent) => void;
    class?: string;
    children: Snippet;
  }

  let {
    variant = "primary",
    size = "md",
    icon,
    loading = false,
    disabled = false,
    fullWidth = false,
    onclick,
    class: className = "",
    children,
  }: Props = $props();

  const baseClasses =
    "inline-flex items-center justify-center gap-2 font-medium transition-all duration-150 ease-out select-none cursor-pointer focus:outline-none";

  const variantClasses: Record<string, string> = {
    primary:
      "bg-[var(--color-accent)] text-[var(--color-text-inverse)] hover:bg-[var(--color-accent-hover)] hover:scale-[1.02] hover:shadow-[var(--glow-accent)] active:scale-[0.97] focus:ring-2 focus:ring-[var(--color-accent)] focus:ring-offset-2 focus:ring-offset-[var(--color-bg-base)]",
    secondary:
      "bg-[var(--color-bg-surface)] text-[var(--color-text-primary)] border border-[var(--color-border-default)] hover:bg-[var(--color-bg-elevated)] hover:scale-[1.02] active:scale-[0.97] focus:ring-2 focus:ring-[var(--color-accent)] focus:ring-offset-2 focus:ring-offset-[var(--color-bg-base)]",
    ghost:
      "bg-transparent text-[var(--color-text-secondary)] hover:bg-[var(--color-bg-surface)] hover:text-[var(--color-text-primary)] active:scale-[0.97] focus:ring-2 focus:ring-[var(--color-accent)]",
    danger:
      "bg-[var(--color-status-error)] text-white hover:brightness-110 hover:scale-[1.02] active:scale-[0.97] focus:ring-2 focus:ring-[var(--color-status-error)] focus:ring-offset-2 focus:ring-offset-[var(--color-bg-base)]",
  };

  const sizeClasses: Record<string, string> = {
    sm: "h-7 px-3 text-[11px] rounded-[var(--radius-sm)]",
    md: "h-9 px-4 text-[13px] rounded-[var(--radius-md)]",
    lg: "h-11 px-6 text-[15px] rounded-[var(--radius-md)]",
  };
</script>

<button
  class="{baseClasses} {variantClasses[variant]} {sizeClasses[size]} {fullWidth ? 'w-full' : ''} {disabled || loading ? 'opacity-50 cursor-not-allowed pointer-events-none' : ''} {className}"
  {disabled}
  {onclick}
>
  {#if loading}
    <Spinner size="sm" />
  {:else if icon}
    {@render icon()}
  {/if}
  {@render children()}
</button>
