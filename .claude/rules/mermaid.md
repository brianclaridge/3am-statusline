---
paths:
  - "**/*.md"
---

# Mermaid Diagram Conventions

Rules for Mermaid diagrams embedded in markdown, optimized for GitHub rendering and narrow viewports.

## Layout

- **Prefer `LR` (left-to-right) over `TD` (top-down)** for pipelines and data flows — horizontal layouts fit narrow screens better
- Use `TD` only when hierarchy depth exceeds width (deep trees, org charts)
- Keep node labels short — 2–4 words max, abbreviate where obvious
- Limit diagrams to 8–12 nodes — split larger diagrams into multiple blocks
- Avoid long edge labels — use node annotations or a legend below the diagram instead

## Node Style

- Use short lowercase IDs: `src`, `db`, `api`, `kb`
- Use display labels for readability: `src[Content Sources]`
- Prefer rounded nodes `()` for processes, square `[]` for data stores, stadium `([])` for inputs/outputs
- Group related nodes with `subgraph` when it clarifies structure — keep subgraph titles short

## Edges

- Use `-->` for standard flow, `-.->` for optional/async paths
- Use `==>` sparingly for emphasis on critical paths
- Label edges only when the relationship is non-obvious
- Avoid crossing edges — reorder node declarations to minimize crossings

## Dark Theme Compatibility

- **Never hardcode colors** with `style` or `classDef` — GitHub applies its own theme and overrides break in dark mode
- Do not use `fill`, `stroke`, or `color` properties — rely on Mermaid's default theming
- Use node shape differences (round, square, stadium, diamond) to convey meaning instead of color
- If distinction is essential, use `classDef` with only `stroke-width` or `stroke-dasharray` — these survive theme switching

## GitHub Rendering

- GitHub renders Mermaid natively in fenced blocks — no plugins needed
- Test that diagrams render at ~600px width (mobile GitHub / split-pane views)
- If a diagram overflows horizontally, break it into sequential sub-diagrams with a brief prose bridge between them
- Avoid `%%` comments in diagrams rendered on GitHub — some parsers choke on them
