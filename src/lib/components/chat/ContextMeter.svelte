<script lang="ts">
  import { _ } from "svelte-i18n";

  interface Props {
    value?: number; // 0-1 ratio
    class?: string;
  }

  let { value = 0, class: className = "" }: Props = $props();

  const percentage = $derived(Math.max(0, Math.min(100, value * 100)));

  const level = $derived<"low" | "medium" | "high">(
    percentage < 50 ? "low" : percentage < 75 ? "medium" : "high",
  );

  const barColor = $derived(
    level === "low"
      ? "var(--color-status-success)"
      : level === "medium"
        ? "var(--color-status-warning)"
        : "var(--color-status-error)",
  );

  const label = $derived($_(
    level === "low"
      ? "chat.contextMeter.low"
      : level === "medium"
        ? "chat.contextMeter.medium"
        : "chat.contextMeter.high",
  ));
</script>

<div class="flex items-center gap-2 {className}" title="{$_('chat.contextMeter.label')}: {Math.round(percentage)}% — {label}">
  <span class="text-[var(--text-caption)] text-[var(--color-text-tertiary)] whitespace-nowrap">
    {$_("chat.contextMeter.label")}
  </span>
  <div class="w-20 h-1.5 rounded-[var(--radius-full)] bg-[var(--color-bg-overlay)] overflow-hidden">
    <div
      class="h-full rounded-[var(--radius-full)] transition-[width] duration-500 ease-out"
      class:animate-pulse={level === "high"}
      style="width: {percentage}%; background: {barColor}"
    ></div>
  </div>
</div>
