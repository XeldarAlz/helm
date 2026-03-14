<script lang="ts">
  interface Props {
    class?: string;
  }

  let { class: className = "" }: Props = $props();
</script>

<hr class="border-0 border-t border-[var(--color-border-subtle)] {className}" />
