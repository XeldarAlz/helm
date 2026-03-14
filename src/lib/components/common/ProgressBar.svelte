<script lang="ts">
  interface Props {
    value?: number;
    variant?: "default" | "success" | "warning" | "error";
    size?: "sm" | "md";
    animated?: boolean;
    shimmer?: boolean;
    class?: string;
  }

  let {
    value = 0,
    variant = "default",
    size = "md",
    animated = true,
    shimmer = false,
    class: className = "",
  }: Props = $props();

  const clampedValue = $derived(Math.max(0, Math.min(100, value)));

  const barColors: Record<string, string> = {
    default: "bg-[var(--color-accent)]",
    success: "bg-[var(--color-status-success)]",
    warning: "bg-[var(--color-status-warning)]",
    error: "bg-[var(--color-status-error)]",
  };

  const sizeClasses: Record<string, string> = {
    sm: "h-1",
    md: "h-2",
  };
</script>

<div
  class="w-full rounded-[var(--radius-full)] bg-[var(--color-bg-overlay)] overflow-hidden {sizeClasses[size]} {className}"
>
  <div
    class="h-full rounded-[var(--radius-full)] {barColors[variant]} {animated ? 'transition-[width] duration-500 ease-out' : ''} {shimmer && clampedValue >= 100 ? 'animate-[shimmer_0.4s_ease-out]' : ''}"
    style="width: {clampedValue}%"
  ></div>
</div>
