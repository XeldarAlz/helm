<script lang="ts">
  import { marked } from "marked";
  import { Copy, Check } from "lucide-svelte";
  import { highlightMatches } from "$lib/stores/documents";

  interface Props {
    content: string;
    searchTerm?: string;
    class?: string;
  }

  let { content, searchTerm = "", class: className = "" }: Props = $props();

  let copiedBlock = $state<number | null>(null);

  // Configure marked renderer
  const renderer = new marked.Renderer();

  renderer.heading = ({ text, depth }) => {
    const id = text
      .replace(/<[^>]*>/g, "")
      .replace(/[*_`]/g, "")
      .trim()
      .toLowerCase()
      .replace(/[^\w\s-]/g, "")
      .replace(/\s+/g, "-");
    return `<h${depth} id="${id}" class="doc-heading doc-h${depth}">${text}</h${depth}>`;
  };

  renderer.code = ({ text, lang }) => {
    const langLabel = lang ? `<span class="doc-code-lang">${lang}</span>` : "";
    const escaped = text.replace(/</g, "&lt;").replace(/>/g, "&gt;");
    return `<div class="doc-code-block"><div class="doc-code-header">${langLabel}<button class="doc-copy-btn" data-code="${encodeURIComponent(text)}">Copy</button></div><pre><code class="language-${lang || "text"}">${escaped}</code></pre></div>`;
  };

  marked.setOptions({ renderer, breaks: true, gfm: true });

  const renderedHtml = $derived.by(() => {
    let html = marked.parse(content) as string;
    if (searchTerm && searchTerm.length >= 2) {
      html = highlightMatches(html, searchTerm);
    }
    return html;
  });

  function handleClick(e: MouseEvent) {
    const target = e.target as HTMLElement;
    if (target.classList.contains("doc-copy-btn")) {
      const code = decodeURIComponent(target.dataset.code || "");
      navigator.clipboard.writeText(code);
      target.textContent = "Copied!";
      setTimeout(() => {
        target.textContent = "Copy";
      }, 2000);
    }
  }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="doc-renderer {className}" onclick={handleClick}>
  {@html renderedHtml}
</div>

<style>
  /* Headings */
  :global(.doc-renderer .doc-heading) {
    font-weight: 600;
    color: var(--color-text-primary);
    scroll-margin-top: 16px;
  }
  :global(.doc-renderer .doc-h1) {
    font-size: var(--text-heading);
    margin: 1.5em 0 0.5em;
    padding-bottom: 0.3em;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  :global(.doc-renderer .doc-h2) {
    font-size: var(--text-title);
    margin: 1.25em 0 0.4em;
    padding-bottom: 0.2em;
    border-bottom: 1px solid var(--color-border-subtle);
  }
  :global(.doc-renderer .doc-h3) {
    font-size: var(--text-subtitle);
    margin: 1em 0 0.3em;
  }
  :global(.doc-renderer .doc-h4) {
    font-size: var(--text-body);
    margin: 0.75em 0 0.25em;
  }
  :global(.doc-renderer .doc-h1:first-child) {
    margin-top: 0;
  }

  /* Paragraphs */
  :global(.doc-renderer p) {
    margin-bottom: 0.75em;
    line-height: 1.7;
    color: var(--color-text-secondary);
  }

  /* Lists */
  :global(.doc-renderer ul),
  :global(.doc-renderer ol) {
    padding-left: 1.5em;
    margin-bottom: 0.75em;
  }
  :global(.doc-renderer li) {
    margin-bottom: 0.35em;
    line-height: 1.6;
    color: var(--color-text-secondary);
  }
  :global(.doc-renderer li::marker) {
    color: var(--color-text-tertiary);
  }

  /* Inline code */
  :global(.doc-renderer code) {
    font-family: var(--font-mono);
    font-size: 0.88em;
    padding: 0.15em 0.4em;
    border-radius: var(--radius-sm);
    background: var(--color-bg-overlay);
    color: var(--color-accent);
  }

  /* Code blocks */
  :global(.doc-renderer .doc-code-block) {
    margin: 0.75em 0;
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--color-bg-deep);
    border: 1px solid var(--color-border-subtle);
  }
  :global(.doc-renderer .doc-code-header) {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0.25em 0.75em;
    border-bottom: 1px solid var(--color-border-subtle);
    background: color-mix(in srgb, var(--color-bg-deep) 50%, var(--color-bg-surface));
  }
  :global(.doc-renderer .doc-code-lang) {
    font-size: 10px;
    font-family: var(--font-mono);
    color: var(--color-text-tertiary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  :global(.doc-renderer .doc-copy-btn) {
    font-size: 10px;
    color: var(--color-text-tertiary);
    background: none;
    border: none;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    transition: color 0.15s, background 0.15s;
  }
  :global(.doc-renderer .doc-copy-btn:hover) {
    color: var(--color-text-secondary);
    background: var(--color-bg-overlay);
  }
  :global(.doc-renderer .doc-code-block pre) {
    margin: 0;
    padding: 0.75em;
    overflow-x: auto;
  }
  :global(.doc-renderer .doc-code-block pre code) {
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.6;
    background: none;
    padding: 0;
    color: var(--color-text-primary);
    border-radius: 0;
  }

  /* Tables */
  :global(.doc-renderer table) {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
    margin: 0.75em 0;
    border: 1px solid var(--color-border-subtle);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  :global(.doc-renderer th) {
    text-align: left;
    padding: 0.5em 0.75em;
    font-weight: 600;
    color: var(--color-text-primary);
    background: var(--color-bg-elevated);
    border-bottom: 1px solid var(--color-border-subtle);
  }
  :global(.doc-renderer td) {
    padding: 0.5em 0.75em;
    color: var(--color-text-secondary);
    border-bottom: 1px solid var(--color-border-subtle);
  }
  :global(.doc-renderer tr:nth-child(even)) {
    background: color-mix(in srgb, var(--color-bg-surface) 50%, transparent);
  }
  :global(.doc-renderer tbody tr:last-child td) {
    border-bottom: none;
  }

  /* Blockquotes */
  :global(.doc-renderer blockquote) {
    border-left: 3px solid var(--color-accent);
    padding: 0.5em 0.75em;
    margin: 0.75em 0;
    background: var(--color-accent-muted);
    border-radius: 0 var(--radius-sm) var(--radius-sm) 0;
  }
  :global(.doc-renderer blockquote p) {
    margin-bottom: 0;
    color: var(--color-text-primary);
  }

  /* Horizontal rules */
  :global(.doc-renderer hr) {
    border: none;
    border-top: 1px solid var(--color-border-subtle);
    margin: 1.5em 0;
  }

  /* Links */
  :global(.doc-renderer a) {
    color: var(--color-accent);
    text-decoration: underline;
    text-underline-offset: 2px;
  }
  :global(.doc-renderer a:hover) {
    color: var(--color-accent-hover);
  }

  /* Strong and emphasis */
  :global(.doc-renderer strong) {
    font-weight: 600;
    color: var(--color-text-primary);
  }
  :global(.doc-renderer em) {
    font-style: italic;
  }

  /* Search highlight */
  :global(.doc-renderer .doc-search-highlight) {
    background: var(--color-status-warning);
    color: var(--color-text-inverse);
    padding: 0.05em 0.15em;
    border-radius: 2px;
  }

  /* Checkbox lists */
  :global(.doc-renderer input[type="checkbox"]) {
    margin-right: 0.4em;
    accent-color: var(--color-accent);
  }

  /* Images */
  :global(.doc-renderer img) {
    max-width: 100%;
    border-radius: var(--radius-md);
    margin: 0.5em 0;
  }
</style>
