import { writable, derived } from "svelte/store";

export type DocName = "GDD" | "TDD" | "WORKFLOW" | "PROGRESS" | "ACTIVITY_LOG";

export interface DocSection {
  id: string;
  text: string;
  level: number;
}

export const documents = writable<Record<string, string>>({});
export const activeDocTab = writable<DocName>("GDD");
export const docSearchTerm = writable("");

/** Parse markdown headings into a section list for navigation */
export function parseSections(markdown: string): DocSection[] {
  const sections: DocSection[] = [];
  const lines = markdown.split("\n");

  for (const line of lines) {
    const match = line.match(/^(#{1,4})\s+(.+)/);
    if (match) {
      const level = match[1].length;
      const text = match[2].replace(/[*_`]/g, "").trim();
      const id = text
        .toLowerCase()
        .replace(/[^\w\s-]/g, "")
        .replace(/\s+/g, "-");
      sections.push({ id, text, level });
    }
  }

  return sections;
}

/** Highlight search matches in text */
export function highlightMatches(html: string, term: string): string {
  if (!term || term.length < 2) return html;
  const escaped = term.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const regex = new RegExp(`(${escaped})`, "gi");
  return html.replace(regex, '<mark class="doc-search-highlight">$1</mark>');
}
