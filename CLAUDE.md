# 3am-statusline

Standalone Rust binary that renders a configurable status display for Claude Code sessions.

## Quick reference

| Action | Command |
| -------- | --------- |
| Build all platforms | `task build` |
| Run tests | `task test` |
| Test manually | `echo '{}' \| ./bin/target/debug/3am-statusline` |
| Config file | `.claude/statusline.yml` |
| Cache dir | `.data/statusline/` |

## Architecture

```text
src/
  main.rs       # stdin -> config -> log -> render -> stdout -> events -> persist
  config.rs     # YAML parsing, config discovery, defaults
  events.rs     # Timer-based event system (shell commands at intervals)
  payload.rs    # Serde types for Claude Code JSON schema
  template.rs   # Template parser: {field}, {field|format}, {meter:field}
  meter.rs      # Meter rendering with ANSI colors
  budget.rs     # JSONL persistence, weekly/monthly aggregation
  format.rs     # Format specifiers (currency, pct, duration, tokens, comma)
  ratelimit.rs  # Anthropic API rate limit cache
```

## Config (`statusline.yml`)

Five sections: `lines`, `meter`, `events`, `budget`, `logging`.

### Config search order

1. `$STATUSLINE_CONFIG` (explicit override via env var)
2. `.claude/statusline.yml` (project-local)
3. `$CLAUDE_CONFIG_DIR/statusline.yml` (per-user)
4. Built-in default (hardcoded two-line layout)

### Template tokens

| Form | Example | Renders |
| ------ | --------- | --------- |
| `{field.path}` | `{model.display_name}` | `Opus 4.6` |
| `{field.path\|format}` | `{cost.total_cost_usd\|currency}` | `$0.55` |
| `{field.path\|color}` | `{model.display_name\|dim}` | dim text |
| `{meter:field.path}` | `{meter:context_window.used_percentage}` | `[###-------]` |
| `{event.name}` | `{event.branch}` | `main` |

### Format specifiers

| Specifier | Input | Output |
| ----------- | ------- | -------- |
| `currency` | `0.55` | `$0.55` |
| `pct` | `8.3` | `8%` |
| `duration` | `45` | `45s` |
| `tokens` | `15234` | `15.2K` |
| `comma` | `15234` | `15,234` |

## Events

Timer-based shell command execution. Runs commands at configurable intervals, caches stdout, injects into templates as `{event.name}`.

```yaml
events:
  - name: branch
    command: "git rev-parse --abbrev-ref HEAD 2>/dev/null"
    interval: 10s
    capture: true
```

| `capture` | Behavior |
| ----------- | ---------- |
| `true` | Blocks, captures stdout, injects as `{event.name}` |
| `false` | Fire-and-forget, no stdout capture |

Sentinel: `.data/statusline/events.json`

## Cross-platform

| Target | Binary |
| -------- | -------- |
| `x86_64-unknown-linux-musl` | `3am-statusline-linux-x64` |
| `aarch64-unknown-linux-gnu` | `3am-statusline-linux-arm64` |
| `x86_64-pc-windows-gnu` | `3am-statusline-win-x64.exe` |
| `x86_64-apple-darwin` | `3am-statusline-darwin-x64` |
| `aarch64-apple-darwin` | `3am-statusline-darwin-arm64` |

macOS binaries require building on macOS (no cross-compile from Linux).

## Stack

- Rust 2021 edition, zero async runtime
- serde + serde_json + serde_yaml for data
- chrono for timestamps
- anyhow for errors
- Raw ANSI escape codes (no colored crate)
