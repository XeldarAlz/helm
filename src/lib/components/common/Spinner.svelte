<script lang="ts">
  interface Props {
    size?: "sm" | "md" | "lg";
    class?: string;
  }

  let { size = "md", class: className = "" }: Props = $props();

  const sizeClasses: Record<string, string> = {
    sm: "w-3.5 h-3.5",
    md: "w-5 h-5",
    lg: "w-8 h-8",
  };
</script>

<svg
  class="animate-spin {sizeClasses[size]} {className}"
  viewBox="0 0 24 24"
  fill="none"
>
  <circle
    cx="12"
    cy="12"
    r="10"
    stroke="currentColor"
    stroke-width="3"
    stroke-linecap="round"
    class="opacity-20"
  />
  <path
    d="M12 2a10 10 0 0 1 10 10"
    stroke="currentColor"
    stroke-width="3"
    stroke-linecap="round"
  />
</svg>
