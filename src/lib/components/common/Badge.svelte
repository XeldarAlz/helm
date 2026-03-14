<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    variant?: "default" | "success" | "warning" | "error" | "info" | "coder" | "tester" | "reviewer" | "unity" | "commit";
    size?: "sm" | "md";
    pulse?: boolean;
    class?: string;
    children: Snippet;
  }

  let {
    variant = "default",
    size = "sm",
    pulse = false,
    class: className = "",
    children,
  }: Props = $props();

  const variantClasses: Record<string, string> = {
    default: "bg-[var(--color-bg-overlay)] text-[var(--color-text-secondary)]",
    success: "bg-[var(--color-status-success)]/20 text-[var(--color-status-success)]",
    warning: "bg-[var(--color-status-warning)]/20 text-[var(--color-status-warning)]",
    error: "bg-[var(--color-status-error)]/20 text-[var(--color-status-error)]",
    info: "bg-[var(--color-status-info)]/20 text-[var(--color-status-info)]",
    coder: "bg-[var(--color-agent-coder)]/20 text-[var(--color-agent-coder)]",
    tester: "bg-[var(--color-agent-tester)]/20 text-[var(--color-agent-tester)]",
    reviewer: "bg-[var(--color-agent-reviewer)]/20 text-[var(--color-agent-reviewer)]",
    unity: "bg-[var(--color-agent-unity)]/20 text-[var(--color-agent-unity)]",
    commit: "bg-[var(--color-agent-commit)]/20 text-[var(--color-agent-commit)]",
  };

  const sizeClasses: Record<string, string> = {
    sm: "px-1.5 py-0.5 text-[10px]",
    md: "px-2 py-0.5 text-[11px]",
  };
</script>

<span
  class="inline-flex items-center font-medium rounded-[var(--radius-full)] {variantClasses[variant]} {sizeClasses[size]} {pulse ? 'animate-[glow-pulse_2s_ease-in-out_infinite]' : ''} {className}"
>
  {@render children()}
</span>
