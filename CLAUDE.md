# 3am-statusline

Standalone Rust binary that renders a configurable status display for Claude Code sessions.

## Quick reference

| Action | Command |
| --- | --- |
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
    version.rs  # Latest Claude Code version check via GitHub releases (JSON)
    weather.rs  # Weather via Open-Meteo + Zippopotam.us geocoding (JSON)
  events.rs     # Timer-based event system (shell commands at intervals)
  payload.rs    # Serde types for Claude Code JSON schema
  template.rs   # Template parser: {field}, {field|format}, {meter:field}, {sep}
  meter.rs      # Meter rendering with ANSI colors
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

## Extended docs

- [Configuration](docs/configuration.md) — config sections, template tokens, fields, color fields
- [Events](docs/events.md) — built-in events, custom events, wiring

## Cross-platform

| Target | Binary |
| --- | --- |
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
- ureq for blocking HTTP (Claude API status, Open-Meteo weather, GitHub releases, Zippopotam.us geocoding)
- anyhow for errors
- unicode-width for emoji/wide-character display width calculation
- Raw ANSI escape codes (no colored crate)
