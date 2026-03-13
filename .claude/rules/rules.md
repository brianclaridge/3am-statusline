# Writing rules

Guidelines for creating and maintaining `.claude/rules/` files in this project.

## Structure

- One concern per file, descriptive filename (e.g., `rust.md`, `release.md`)
- Keep each file under 100 lines for context efficiency
- Use markdown headers, tables, and bullet lists — scannable over prose
- Bold **MANDATORY** for non-negotiable requirements

## Path-scoped rules

Use YAML frontmatter to scope rules to specific file patterns:

```yaml
---
paths:
  - "src/**/*.rs"
---
```

- Rules without `paths` load at session start unconditionally
- Rules with `paths` load on demand when Claude reads matching files
- Glob syntax: `**/*.rs`, `src/**/*`, `*.md`, `src/event/*.rs`

## Precedence (highest to lowest)

1. Managed policy CLAUDE.md (org-wide, cannot exclude)
2. Project CLAUDE.md (`.claude/CLAUDE.md` or `./CLAUDE.md`)
3. Project rules (`.claude/rules/*.md`)
4. User rules (`~/.claude/rules/*.md`)
5. User CLAUDE.md (`~/.claude/CLAUDE.md`)

## Avoid

- Contradicting instructions across rule files — Claude picks arbitrarily
- Duplicating content already in CLAUDE.md
- Rules longer than 100 lines — split into separate files
- Dynamic or conditional logic — use hooks for that

## Debugging

- `/memory` lists all loaded rules and CLAUDE.md files
- `InstructionsLoaded` hook logs which rules loaded and why
