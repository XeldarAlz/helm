import { describe, it, expect } from "vitest";
import { parseSections, highlightMatches } from "../documents";

describe("parseSections", () => {
  it("extracts headings with correct levels", () => {
    const md = `# Title\n\nSome text\n\n## Section One\n\n### Sub-section\n\n## Section Two`;
    const sections = parseSections(md);

    expect(sections).toHaveLength(4);
    expect(sections[0]).toEqual({ id: "title", text: "Title", level: 1 });
    expect(sections[1]).toEqual({ id: "section-one", text: "Section One", level: 2 });
    expect(sections[2]).toEqual({ id: "sub-section", text: "Sub-section", level: 3 });
    expect(sections[3]).toEqual({ id: "section-two", text: "Section Two", level: 2 });
  });

  it("returns empty array for text without headings", () => {
    expect(parseSections("Just a paragraph.\nAnother line.")).toEqual([]);
  });

  it("strips markdown formatting from heading text", () => {
    const sections = parseSections("## **Bold** and _italic_ heading");
    expect(sections[0].text).toBe("Bold and italic heading");
  });

  it("handles h4 headings", () => {
    const sections = parseSections("#### Deep heading");
    expect(sections[0]).toEqual({ id: "deep-heading", text: "Deep heading", level: 4 });
  });

  it("ignores h5+ headings", () => {
    expect(parseSections("##### Too deep")).toEqual([]);
  });
});

describe("highlightMatches", () => {
  it("wraps matching text in <mark> tags", () => {
    const result = highlightMatches("Hello World", "World");
    expect(result).toBe('Hello <mark class="doc-search-highlight">World</mark>');
  });

  it("is case-insensitive", () => {
    const result = highlightMatches("Hello World", "hello");
    expect(result).toBe('<mark class="doc-search-highlight">Hello</mark> World');
  });

  it("returns original for short terms (< 2 chars)", () => {
    expect(highlightMatches("Hello", "H")).toBe("Hello");
    expect(highlightMatches("Hello", "")).toBe("Hello");
  });

  it("escapes regex special characters in search term", () => {
    const result = highlightMatches("price is $100.00", "$100.00");
    expect(result).toContain("doc-search-highlight");
  });

  it("highlights multiple occurrences", () => {
    const result = highlightMatches("foo bar foo", "foo");
    const matches = result.match(/doc-search-highlight/g);
    expect(matches).toHaveLength(2);
  });
});
