# Claude Code agents reference

## File structure

Agent definitions are `.md` files with YAML frontmatter:

```text
.claude/agents/
  code-reviewer.md    # project-level agent
~/.claude/agents/
  my-agent.md         # user-level agent
```

## Agent definition format

```yaml
---
name: code-reviewer
description: Expert code reviewer. Use proactively after code changes.
tools: Read, Glob, Grep, Bash
model: sonnet
---

You are a senior code reviewer. When invoked, analyze code changes...
```

The body after frontmatter becomes the agent's system prompt.

## Frontmatter fields

| Field | Type | Required | Description |
| --- | --- | --- | --- |
| `name` | string | yes | Unique identifier (lowercase, hyphens) |
| `description` | string | yes | When Claude should delegate to this agent |
| `tools` | string | no | Comma-separated tool allowlist. Inherits all if omitted |
| `disallowedTools` | string | no | Comma-separated tool denylist |
| `model` | string | no | `sonnet`, `opus`, `haiku`, full model ID, or `inherit` (default) |
| `permissionMode` | string | no | `default`, `acceptEdits`, `dontAsk`, `bypassPermissions`, `plan` |
| `maxTurns` | int | no | Maximum agentic turns before stopping |
| `skills` | string list | no | Skills to preload into agent context |
| `mcpServers` | object | no | MCP servers available to agent |
| `hooks` | object | no | Lifecycle hooks (`PreToolUse`, `PostToolUse`, `Stop`) |
| `memory` | string | no | Persistent memory scope: `user`, `project`, or `local` |
| `background` | bool | no | Run as background task (default: false) |
| `isolation` | string | no | `worktree` for git worktree isolation |

## Tool restriction syntax

- `tools: Read, Grep, Bash` -- simple allowlist
- `tools: Agent(worker, researcher)` -- restrict subagent spawning to named agents
- `disallowedTools: Write, Edit` -- denylist (removed from inherited set)

## Storage priority (highest first)

1. `--agents` CLI flag (session only, JSON)
2. `.claude/agents/` (project, version-controlled)
3. `~/.claude/agents/` (user, personal)
4. Plugin `agents/` directory

## Agents vs skills

| Aspect | Agents | Skills |
| --- | --- | --- |
| Context | Own isolated context window | Injected into main conversation |
| System prompt | Custom full system prompt | Content injection, not system prompt |
| Tool access | Restricted via `tools` field | Inherit parent's tools |
| Model | Can override | Uses parent's model |
| Use case | Self-contained isolated tasks | Reusable workflows in main context |

## Built-in agents

| Agent | Model | Tools | Purpose |
| --- | --- | --- | --- |
| Explore | haiku | Read-only | Fast codebase search |
| Plan | inherit | Read-only | Research during plan mode |
| general-purpose | inherit | All | Complex multi-step operations |
