# Claude Code plugin development

Reference for building and installing Claude Code plugins.

## File structure

```text
my-plugin/
  .claude-plugin/
    plugin.json       # manifest (required)
  skills/
    SKILL.md          # one file per skill
  bin/release/        # compiled binaries (if applicable)
```

## Manifest (`plugin.json`)

| Field | Type | Description |
| --- | --- | --- |
| `name` | string | Plugin identifier (lowercase, hyphens) |
| `version` | string | Version string (this project uses `YYYY.MM.DD-gitsha`) |
| `description` | string | One-line summary |
| `author` | object/string | `{ "name": "..." }` or plain string |
| `repository` | string | Git URL |
| `license` | string | SPDX identifier |
| `keywords` | string[] | Discovery tags |
| `skills` | string | Path to skills directory (default `skills/`) |

## Skills

Each skill is a `SKILL.md` file in the skills directory.

| Concept | Detail |
| --- | --- |
| **Naming** | `plugin-name:skill-name` (colon-separated) |
| **Invocation** | `/plugin-name:skill-name` from the Claude Code prompt |
| **Format** | Markdown with frontmatter defining the skill metadata |
| **Execution** | Skills can run shell commands and access environment variables |

### SKILL.md frontmatter

```yaml
---
name: skill-name
description: What this skill does
---
```

The body contains the prompt template and instructions for Claude Code.

## Installation

| Method | Command |
| --- | --- |
| **From marketplace** | `claude plugin add <plugin-name>` |
| **Local development** | `claude plugin add --plugin-dir ./` |
| **Remove** | `claude plugin remove <plugin-name>` |

## Key concepts

- **Plugin root** is available as `$CLAUDE_PLUGIN_ROOT` in shell commands within skills.
- Skills run in the user's shell environment and inherit all env vars.
- Binaries in `bin/release/` can be invoked from skills via `$CLAUDE_PLUGIN_ROOT/bin/release/<binary>`.
- Plugins are isolated -- each gets its own namespace and cannot conflict with other plugins.
- Test locally with `--plugin-dir` before publishing.
