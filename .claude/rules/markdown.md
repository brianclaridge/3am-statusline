---
paths:
  - "**/*.md"
---

# Markdown Formatting Conventions

Follow these conventions when writing or editing markdown files in this project.

## Document Structure

- Single H1 (`#`) as the document title — no body text on the same level
- Use H2 (`##`) for all major sections
- Use H3 (`###`) sparingly, only for subsections within H2 blocks
- Single blank line between sections, no double blanks
- No trailing whitespace on any line

## Headings

- Title case for H1
- Sentence case for H2 and below (capitalize first word and proper nouns only)
- No punctuation at the end of headings

## Inline Formatting

- **Bold** (`**`) for key terms, labels, and emphasis in lists
- Backticks for code references: commands (`task setup`), paths (`.ai-inbox/`), filenames (`config.yml`), variables, and inline code
- No italic unless quoting external terms

## Lists

- Dash (`-`) prefix for unordered lists, not asterisks
- **Bold label** followed by em dash (`—`) or colon for definition-style lists:
  ```markdown
  - **Python** (Typer CLI + Rich) — core pipeline and daemon
  ```
- No trailing punctuation on list items that are sentence fragments
- Full sentences in lists get a period

## Tables

- Pipe-delimited with header separator row
- Bold the first column when it serves as a label
- Align separator dashes with hyphens only (`---`), no colons unless alignment is needed

## Code Blocks

- Always fenced with triple backticks
- Always include a language hint: `bash`, `python`, `text`, `yaml`, etc.

## Diagrams

- **Prefer Mermaid over text diagrams** — use `mermaid` fenced code blocks for flowcharts, graphs, sequences, architecture, and relationships
- Use `text` fenced blocks only for directory trees where box-drawing characters (`├──`, `│`, `└──`) are clearer than Mermaid
- For architecture overviews, use Mermaid `graph TD` or `graph LR`
- For data flows and pipelines, use Mermaid `flowchart LR`
- For sequences and interactions, use Mermaid `sequenceDiagram`
- For entity relationships, use Mermaid `erDiagram`

## Content Guidelines

- Terse, direct language — no filler
- Present tense for descriptions of what the system does
- Second person ("you") for instructions and quick-start guides
- Keep paragraphs short (2–4 sentences max)
