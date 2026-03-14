<script lang="ts">
  interface Props {
    answers: string[];
    onselect: (answer: string) => void;
    disabled?: boolean;
  }

  let { answers, onselect, disabled = false }: Props = $props();
</script>

{#if answers.length > 0}
  <div class="flex flex-wrap gap-2 px-4 pb-2">
    {#each answers as answer}
      <button
        class="px-3 py-1.5 text-[var(--text-caption)] font-medium rounded-[var(--radius-full)]
          bg-[var(--color-bg-surface)] border border-[var(--color-border-default)]
          text-[var(--color-text-secondary)] cursor-pointer
          hover:bg-[var(--color-bg-elevated)] hover:text-[var(--color-text-primary)]
          hover:border-[var(--color-accent)] hover:shadow-[var(--glow-accent)]
          active:scale-[0.97] transition-all duration-150
          disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:bg-[var(--color-bg-surface)]
          disabled:hover:border-[var(--color-border-default)] disabled:hover:shadow-none"
        {disabled}
        onclick={() => onselect(answer)}
      >
        {answer}
      </button>
    {/each}
  </div>
{/if}
