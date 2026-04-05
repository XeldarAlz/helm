<script lang="ts">
  import { _ } from "svelte-i18n";
  import { Copy, Check, FileText } from "lucide-svelte";
  import { formatFileSize } from "$lib/utils/format";
  import { codeToTokens, type ThemedToken } from "shiki";

  interface Props {
    content: string;
    filePath: string;
    language?: string;
    fileSize?: number | null;
  }

  let { content, filePath, language = "", fileSize = null }: Props = $props();

  let copied = $state(false);
  let tokenLines = $state<ThemedToken[][] | null>(null);

  const lines = $derived(content.split("\n"));
  const lineCount = $derived(lines.length);

  // Detect language from file extension
  const detectedLang = $derived.by(() => {
    if (language) return language;
    const ext = filePath.split(".").pop()?.toLowerCase() || "";
    const langMap: Record<string, string> = {
      cs: "csharp",
      rs: "rust",
      ts: "typescript",
      js: "javascript",
      svelte: "svelte",
      json: "json",
      md: "markdown",
      css: "css",
      scss: "scss",
      html: "html",
      toml: "toml",
      yaml: "yaml",
      yml: "yaml",
      xml: "xml",
      sh: "bash",
      py: "python",
    };
    return langMap[ext] || "text";
  });

  // Highlight code with Shiki
  $effect(() => {
    const code = content;
    const lang = detectedLang;

    tokenLines = null;

    if (lang === "text" || !code) return;

    codeToTokens(code, {
      lang: lang as any,
      theme: "github-dark-default",
    })
      .then((result) => {
        if (content === code && detectedLang === lang) {
          tokenLines = result.tokens;
        }
      })
      .catch(() => {
        // Fall back to plain text on error
        tokenLines = null;
      });
  });

  async function copyContent() {
    await navigator.clipboard.writeText(content);
    copied = true;
    setTimeout(() => { copied = false; }, 2000);
  }

  async function copyPath() {
    await navigator.clipboard.writeText(filePath);
  }
</script>

<div class="flex flex-col h-full">
  <!-- File header -->
  <div class="flex items-center justify-between px-4 py-2 bg-[var(--color-bg-base)] border-b border-[var(--color-border-subtle)]">
    <div class="flex items-center gap-2 min-w-0">
      <FileText size={14} class="text-[var(--color-text-tertiary)] shrink-0" />
      <span class="text-[12px] font-mono text-[var(--color-text-primary)] truncate">{filePath}</span>
      <span class="text-[10px] text-[var(--color-text-tertiary)] shrink-0">
        {detectedLang}
      </span>
    </div>
    <div class="flex items-center gap-3 shrink-0">
      {#if fileSize != null}
        <span class="text-[10px] text-[var(--color-text-tertiary)]">
          {formatFileSize(fileSize)}
        </span>
      {/if}
      <span class="text-[10px] text-[var(--color-text-tertiary)]">
        {lineCount} lines
      </span>
      <button
        onclick={copyContent}
        class="flex items-center gap-1 px-2 py-0.5 text-[10px] text-[var(--color-text-tertiary)] hover:text-[var(--color-text-secondary)]
          border border-[var(--color-border-subtle)] rounded-[var(--radius-sm)] transition-colors cursor-pointer"
      >
        {#if copied}
          <Check size={10} />
          {$_("code.copied")}
        {:else}
          <Copy size={10} />
          {$_("code.copyPath")}
        {/if}
      </button>
    </div>
  </div>

  <!-- Code content with line numbers -->
  <div class="flex-1 overflow-auto bg-[var(--color-bg-deep)]">
    <div class="flex min-w-fit">
      <!-- Line numbers -->
      <div class="sticky left-0 select-none bg-[var(--color-bg-deep)] border-r border-[var(--color-border-subtle)] z-10">
        {#each lines as _, i}
          <div class="px-3 py-0 text-right text-[12px] font-mono leading-[1.6] text-[var(--color-text-tertiary)]">
            {i + 1}
          </div>
        {/each}
      </div>

      <!-- Code content -->
      <pre class="flex-1 p-0 m-0"><code class="block text-[12px] font-mono leading-[1.6]">{#if tokenLines}{#each tokenLines as tokens, i}<div class="px-4 hover:bg-[var(--color-bg-surface)]/30">{#each tokens as token}<span style="color:{token.color}">{token.content}</span>{/each}{#if tokens.length === 0}{" "}{/if}</div>{/each}{:else}{#each lines as line, i}<div class="px-4 hover:bg-[var(--color-bg-surface)]/30 text-[var(--color-text-primary)]">{line || " "}</div>{/each}{/if}</code></pre>
    </div>
  </div>
</div>
