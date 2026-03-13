# 3am-statusline

Standalone Rust binary that renders a configurable status display for Claude Code sessions.

## Quick reference

| Action | Command |
| -------- | --------- |
| Build all platforms | `task build` |
| Run tests | `task test` |
| Test render | `echo '{}' \| ./bin/target/debug/3am-statusline` |
| Test events | `./bin/target/debug/3am-statusline event git` |
| Config file | `config/statusline.yml` |
| Cache dir | `.data/statusline/` |
| Plugin manifest | `.claude-plugin/plugin.json` |

## Architecture

```text
src/
  main.rs       # clap CLI dispatch: default render or event subcommands
  config.rs     # YAML parsing, config discovery, defaults
  event/        # Built-in event subcommands (replace Python scripts)
    mod.rs      # Module declarations
    git.rs      # Branch, ahead/behind, dirty counts (JSON)
    time.rs     # World clock with chrono-tz (JSON)
    sys.rs      # CPU/mem via sysinfo crate (JSON)
    status.rs   # Claude API status via ureq (plain text)
  events.rs     # Timer-based event system (shell commands at intervals)
  payload.rs    # Serde types for Claude Code JSON schema
  template.rs   # Template parser: {field}, {field|format}, {meter:field}
  meter.rs      # Meter rendering with ANSI colors
  budget.rs     # JSONL persistence, weekly/monthly aggregation
  format.rs     # Format specifiers (currency, pct, duration, tokens, comma)
  ratelimit.rs  # Anthropic API rate limit cache
```

## Plugin structure

```text
.claude-plugin/
  plugin.json       # Plugin manifest (name, version, skills path)
  marketplace.json  # Self-hosted marketplace for git-based install
skills/
  setup/
    SKILL.md        # /3am-statusline:setup — platform detection + settings wiring
bin/release/        # Pre-built binaries per platform
```

## Config (`statusline.yml`)

Five sections: `lines`, `meter`, `events`, `budget`, `logging`.

### Config search order

1. `$STATUSLINE_CONFIG` (explicit override via env var)
2. `config/statusline.yml` (project-local)
3. `.claude/statusline.yml` (legacy project-local)
4. `$CLAUDE_CONFIG_DIR/statusline.yml` (per-user)
5. Built-in default (hardcoded two-line layout)

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

Timer-based command execution. Built-in event subcommands replace the old Python scripts:

```text
3am-statusline event git      # {"branch":"main","sha":"2620eb7","sync":"↑1","dirty":"~3 +1 ?2"}
3am-statusline event time     # {"pst":"3:45p","mst":"4:45p","cst":"5:45p","est":"6:45p"}
3am-statusline event sys      # {"cpu":"12%","cores":"8","mem":"4.2/16G (26%)","mem_pct":"26%","mem_used":"4G","mem_total":"16G"}
3am-statusline event status   # 🟢 ok
```

Config wires these as event commands at intervals:

```yaml
events:
  - name: git
    command: "${CLAUDE_PLUGIN_ROOT}/bin/release/3am-statusline event git"
    interval: 5s
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
- clap (derive) for CLI subcommand dispatch
- serde + serde_json + serde_yml for data
- chrono + chrono-tz for timestamps and timezone conversion
- sysinfo for cross-platform CPU/memory stats
- ureq for blocking HTTP (Claude API status)
- anyhow for errors
- Raw ANSI escape codes (no colored crate)
