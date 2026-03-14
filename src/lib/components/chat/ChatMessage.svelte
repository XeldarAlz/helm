<script lang="ts">
  import { marked } from "marked";
  import { timeAgo } from "$lib/utils/format";
  import { fadeScale } from "$lib/animations";
  import type { Message } from "$lib/stores/session";
  import { Copy, Check } from "lucide-svelte";

  interface Props {
    message: Message;
  }

  let { message }: Props = $props();

  let showTimestamp = $state(false);
  let copiedBlock = $state<number | null>(null);

  // Configure marked for safe rendering
  const renderer = new marked.Renderer();

  // Add copy button to code blocks
  const originalCode = renderer.code.bind(renderer);
  renderer.code = ({ text, lang }) => {
    const langLabel = lang ? `<span class="code-lang">${lang}</span>` : "";
    const escaped = text.replace(/</g, "&lt;").replace(/>/g, "&gt;");
    return `<div class="code-block-wrapper">${langLabel}<pre><code class="language-${lang || "text"}">${escaped}</code></pre></div>`;
  };

  marked.setOptions({
    renderer,
    breaks: true,
    gfm: true,
  });

  const renderedContent = $derived(
    message.role === "user"
      ? message.content
      : marked.parse(message.content) as string,
  );

  const isUser = $derived(message.role === "user");
  const isSystem = $derived(message.role === "system");
  const isAgent = $derived(message.role === "agent");

  const bubbleClass = $derived(
    isUser
      ? "bg-[var(--color-accent)] text-[var(--color-text-inverse)]"
      : isSystem
        ? "border-l-2 border-l-[var(--color-status-error)]"
        : "bg-[var(--color-bg-surface)] border-l-2 border-l-[var(--color-accent)]",
  );
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div
  class="group flex w-full px-4 py-1.5"
  class:justify-end={isUser}
  class:justify-start={!isUser}
  in:fadeScale={{ duration: 200 }}
  onmouseenter={() => (showTimestamp = true)}
  onmouseleave={() => (showTimestamp = false)}
  data-msg-id={message.id}
>
  <div
    class="relative max-w-[80%] rounded-[var(--radius-lg)] transition-shadow duration-150 {bubbleClass}"
    style={isSystem ? "background: color-mix(in srgb, var(--color-status-error) 10%, transparent)" : ""}
  >
    <!-- Role label -->
    {#if !isUser}
      <div class="px-4 pt-3 pb-0">
        <span
          class="text-[10px] font-semibold uppercase tracking-wider"
          class:text-[var(--color-accent)]={isAgent}
          class:text-[var(--color-status-error)]={isSystem}
        >
          {isAgent ? "Claude" : "System"}
        </span>
      </div>
    {/if}

    <!-- Message content -->
    <div
      class="px-4 py-2 text-[var(--text-body)] leading-relaxed"
      class:text-[var(--color-text-primary)]={!isUser}
    >
      {#if isUser}
        <p class="whitespace-pre-wrap">{message.content}</p>
      {:else}
        <!-- eslint-disable-next-line svelte/no-at-html-tags -->
        <div class="chat-markdown">{@html renderedContent}</div>
      {/if}
    </div>

    <!-- Timestamp on hover -->
    {#if showTimestamp}
      <div
        class="absolute -bottom-5 text-[10px] text-[var(--color-text-tertiary)] whitespace-nowrap transition-opacity duration-150"
        class:right-2={isUser}
        class:left-2={!isUser}
      >
        {timeAgo(message.timestamp)}
      </div>
    {/if}
  </div>
</div>

<style>
  /* Markdown styles for agent messages */
  :global(.chat-markdown p) {
    margin-bottom: 0.5em;
  }
  :global(.chat-markdown p:last-child) {
    margin-bottom: 0;
  }
  :global(.chat-markdown strong) {
    font-weight: 600;
    color: var(--color-text-primary);
  }
  :global(.chat-markdown em) {
    font-style: italic;
  }
  :global(.chat-markdown ul),
  :global(.chat-markdown ol) {
    padding-left: 1.5em;
    margin-bottom: 0.5em;
  }
  :global(.chat-markdown li) {
    margin-bottom: 0.25em;
  }
  :global(.chat-markdown h1),
  :global(.chat-markdown h2),
  :global(.chat-markdown h3) {
    font-weight: 600;
    color: var(--color-text-primary);
    margin: 0.75em 0 0.25em;
  }
  :global(.chat-markdown h1) {
    font-size: var(--text-title);
  }
  :global(.chat-markdown h2) {
    font-size: var(--text-subtitle);
  }
  :global(.chat-markdown h3) {
    font-size: var(--text-body);
  }

  /* Inline code */
  :global(.chat-markdown code) {
    font-family: var(--font-mono);
    font-size: 0.9em;
    padding: 0.15em 0.4em;
    border-radius: var(--radius-sm);
    background: var(--color-bg-overlay);
    color: var(--color-accent);
  }

  /* Code blocks */
  :global(.chat-markdown .code-block-wrapper) {
    position: relative;
    margin: 0.5em 0;
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-bg-deep);
    border: 1px solid var(--color-border-subtle);
  }
  :global(.chat-markdown .code-block-wrapper .code-lang) {
    display: block;
    padding: 0.25em 0.75em;
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--color-text-tertiary);
    border-bottom: 1px solid var(--color-border-subtle);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  :global(.chat-markdown .code-block-wrapper pre) {
    margin: 0;
    padding: 0.75em;
    overflow-x: auto;
  }
  :global(.chat-markdown .code-block-wrapper pre code) {
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.6;
    background: none;
    padding: 0;
    color: var(--color-text-primary);
    border-radius: 0;
  }

  /* Blockquotes */
  :global(.chat-markdown blockquote) {
    border-left: 2px solid var(--color-border-default);
    padding-left: 0.75em;
    margin: 0.5em 0;
    color: var(--color-text-secondary);
  }

  /* Horizontal rules */
  :global(.chat-markdown hr) {
    border: none;
    border-top: 1px solid var(--color-border-subtle);
    margin: 0.75em 0;
  }

  /* Links */
  :global(.chat-markdown a) {
    color: var(--color-accent);
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  :global(.chat-markdown a:hover) {
    color: var(--color-accent-hover);
  }
</style>
