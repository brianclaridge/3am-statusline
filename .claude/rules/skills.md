# Claude Code skills reference

## File structure

Skills are `SKILL.md` files in a named directory:

```text
skills/
  my-skill/
    SKILL.md          # required
    reference.md      # optional supporting files
    scripts/
      helper.sh       # bundled scripts
```

## SKILL.md format

YAML frontmatter followed by markdown instructions:

```yaml
---
name: my-skill
description: What this skill does and when to use it
---

# Instructions

Claude follows these instructions when the skill is invoked.
```

## Frontmatter fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | string | no | Lowercase, hyphens, max 64 chars. Defaults to directory name |
| `description` | string | no | When/why to invoke. Defaults to first paragraph of body |
| `disable-model-invocation` | bool | no | Only user can invoke (`/skill-name`). Use for side-effect workflows |
| `user-invocable` | bool | no | Set `false` to hide from `/` menu. Claude still auto-invokes |
| `allowed-tools` | string | no | Comma-separated tool allowlist (e.g., `Read, Grep, Bash`) |
| `model` | string | no | Model override (`sonnet`, `opus`, `haiku`, or full ID) |
| `context` | string | no | `fork` to run in isolated subagent context |
| `agent` | string | no | Subagent type when `context: fork` (`Explore`, `Plan`, etc.) |
| `argument-hint` | string | no | Autocomplete hint (e.g., `[issue-number]`) |
| `hooks` | object | no | Skill-scoped lifecycle hooks |

## Invocation control

| Frontmatter | User invokes | Claude invokes | Description loaded |
| --- | --- | --- | --- |
| (default) | yes | yes | Always in context |
| `disable-model-invocation: true` | yes | no | Not in context |
| `user-invocable: false` | no | yes | Always in context |

## Discovery locations (priority order)

1. Enterprise managed settings
2. Personal (`~/.claude/skills/<name>/SKILL.md`)
3. Project (`.claude/skills/<name>/SKILL.md`)
4. Plugin (`<plugin>/skills/<name>/SKILL.md`) -- namespaced as `plugin-name:skill-name`

## String substitutions

| Variable | Value |
| --- | --- |
| `$ARGUMENTS` | All arguments passed at invocation |
| `$ARGUMENTS[N]` or `$N` | Nth argument (0-based) |
| `${CLAUDE_SESSION_ID}` | Current session ID |
| `${CLAUDE_SKILL_DIR}` | Directory containing SKILL.md |

Shell command injection: `` !`command` `` runs before content is sent to Claude.

## Guidelines

- Keep SKILL.md under 500 lines. Link to supporting files for long reference content
- Description quality is critical for auto-invocation. Use "WHEN + WHEN NOT" pattern
- Use `disable-model-invocation: true` for side-effect workflows (deploy, commit, send)
- Use `allowed-tools` to create read-only or restricted modes
- Use `context: fork` for tasks that benefit from isolated context
