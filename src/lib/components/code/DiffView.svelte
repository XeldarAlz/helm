<script lang="ts">
  interface DiffLine {
    type: "add" | "remove" | "context" | "header";
    content: string;
    oldLine?: number;
    newLine?: number;
  }

  interface Props {
    diff: string;
    class?: string;
  }

  let { diff, class: className = "" }: Props = $props();

  const parsedLines = $derived.by((): DiffLine[] => {
    if (!diff) return [];

    const lines = diff.split("\n");
    const result: DiffLine[] = [];
    let oldLine = 0;
    let newLine = 0;

    for (const line of lines) {
      if (line.startsWith("@@")) {
        const match = line.match(/@@ -(\d+),?\d* \+(\d+),?\d* @@/);
        if (match) {
          oldLine = parseInt(match[1], 10);
          newLine = parseInt(match[2], 10);
        }
        result.push({ type: "header", content: line });
      } else if (line.startsWith("+")) {
        result.push({ type: "add", content: line.slice(1), newLine });
        newLine++;
      } else if (line.startsWith("-")) {
        result.push({ type: "remove", content: line.slice(1), oldLine });
        oldLine++;
      } else if (line.startsWith(" ")) {
        result.push({ type: "context", content: line.slice(1), oldLine, newLine });
        oldLine++;
        newLine++;
      }
    }

    return result;
  });

  const stats = $derived.by(() => {
    let additions = 0;
    let deletions = 0;
    for (const line of parsedLines) {
      if (line.type === "add") additions++;
      if (line.type === "remove") deletions++;
    }
    return { additions, deletions };
  });

  function lineBg(type: string): string {
    if (type === "add") return "background: color-mix(in srgb, var(--color-status-success) 8%, transparent)";
    if (type === "remove") return "background: color-mix(in srgb, var(--color-status-error) 8%, transparent)";
    return "";
  }
</script>

<div class="diff-view {className}">
  <!-- Stats header -->
  {#if stats.additions > 0 || stats.deletions > 0}
    <div class="flex items-center gap-3 px-4 py-2 bg-[var(--color-bg-base)] border-b border-[var(--color-border-subtle)] text-[11px]">
      <span class="text-[var(--color-status-success)]">+{stats.additions}</span>
      <span class="text-[var(--color-status-error)]">-{stats.deletions}</span>
    </div>
  {/if}

  <!-- Diff content -->
  <div class="overflow-auto font-mono text-[12px] leading-[1.6] bg-[var(--color-bg-deep)]">
    {#each parsedLines as line}
      {#if line.type === "header"}
        <div class="px-4 py-1 text-[var(--color-status-info)] text-[11px] border-y border-[var(--color-border-subtle)]"
          style="background: color-mix(in srgb, var(--color-status-info) 10%, transparent)">
          {line.content}
        </div>
      {:else}
        <div class="flex hover:brightness-110" style={lineBg(line.type)}>
          <!-- Old line number -->
          <span class="w-10 shrink-0 text-right px-1.5 select-none text-[var(--color-text-tertiary)] border-r border-[var(--color-border-subtle)]">
            {line.type !== "add" ? line.oldLine ?? "" : ""}
          </span>

          <!-- New line number -->
          <span class="w-10 shrink-0 text-right px-1.5 select-none text-[var(--color-text-tertiary)] border-r border-[var(--color-border-subtle)]">
            {line.type !== "remove" ? line.newLine ?? "" : ""}
          </span>

          <!-- Sign -->
          <span class="w-5 shrink-0 text-center select-none"
            class:text-[var(--color-status-success)]={line.type === "add"}
            class:text-[var(--color-status-error)]={line.type === "remove"}
          >
            {line.type === "add" ? "+" : line.type === "remove" ? "-" : " "}
          </span>

          <!-- Content -->
          <span class="flex-1 px-2 whitespace-pre"
            class:text-[var(--color-status-success)]={line.type === "add"}
            class:text-[var(--color-status-error)]={line.type === "remove"}
            class:text-[var(--color-text-secondary)]={line.type === "context"}
          >
            {line.content}
          </span>
        </div>
      {/if}
    {/each}
  </div>
</div>
