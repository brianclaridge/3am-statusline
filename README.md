# 3am-statusline

A configurable status bar for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) sessions. Displays model info, cost, context usage, rate limits, budget tracking, and custom event data.

## Install

### As a Claude Code plugin (recommended)

```bash
/plugin marketplace add brianclaridge/3am-statusline
/plugin install 3am-statusline@3am-statusline
```

Then run the setup skill:

```
/3am-statusline:setup
```

This detects your platform, wires the correct binary into your Claude Code settings, and copies the default config.

### From source

```bash
git clone https://github.com/brianclaridge/3am-statusline.git
cd 3am-statusline
task build
```

Then point your `.claude/settings.json` at the binary:

```json
{
  "statusLine": "/path/to/bin/release/3am-statusline-linux-x64"
}
```

## Configuration

Create `.claude/statusline.yml` in your project root. See `config/statusline.yml` for a full example.

```yaml
lines:
  - left: "{model.display_name}"
    right: "{cost.total_cost_usd|currency}"
  - left: "{meter:context_window.used_percentage} {context_window.used_percentage|pct} ctx"
    right: "{context_window.total_input_tokens|tokens} tok"

meter:
  width: 10
  filled: "●"
  empty: "○"
  thresholds:
    green: 0
    yellow: 60
    red: 85
```

### Template tokens

| Syntax | Example | Output |
| -------- | --------- | -------- |
| `{field}` | `{model.display_name}` | `Opus 4.6` |
| `{field\|format}` | `{cost.total_cost_usd\|currency}` | `$0.55` |
| `{field\|color}` | `{model.display_name\|dim}` | dim text |
| `{meter:field}` | `{meter:context_window.used_percentage}` | `[●●●○○○○○○○]` |
| `{event.name}` | `{event.branch}` | `main` |

### Format specifiers

`currency`, `pct`, `duration`, `tokens`, `comma`

### Available fields

- `model.display_name`, `model.id`
- `cost.total_cost_usd`, `cost.total_duration_secs`
- `context_window.used_percentage`, `context_window.total_input_tokens`, `context_window.total_output_tokens`
- `session_id`, `version`, `build_version`
- `vim.mode`, `agent.name`, `worktree.name`
- `plan.slug`
- `ratelimit.5h`, `ratelimit.7d`, `ratelimit.5h_eta`, `ratelimit.7d_eta`
- `budget.weekly_pct`, `budget.monthly_pct`
- `event.{name}` (from custom events)

## Events

The binary includes built-in event subcommands for common data sources:

```bash
3am-statusline event git                  # branch, SHA, ahead/behind, dirty symbols (JSON)
3am-statusline event time                 # world clocks with day/night emojis (JSON)
3am-statusline event sys                  # CPU/memory stats with separate fields (JSON)
3am-statusline event status               # Claude API status (plain text)
3am-statusline event weather --zip 98101  # current weather from Open-Meteo (JSON)
```

Wire them into your config:

```yaml
events:
  - name: git
    command: "${CLAUDE_PLUGIN_ROOT}/bin/release/3am-statusline event git"
    interval: 5s
    capture: true
  - name: sys
    command: "${CLAUDE_PLUGIN_ROOT}/bin/release/3am-statusline event sys"
    interval: 3s
    capture: true
```

You can also run arbitrary shell commands:

```yaml
  - name: disk
    command: "df -h / | tail -1 | awk '{print $5}'"
    interval: 5m
    capture: true
```

Weather uses [Open-Meteo](https://open-meteo.com/) (free, no API key). Zip code is geocoded via Zippopotam.us and cached permanently in `.data/statusline/geocode.json`.

Use in templates: `{event.git.branch}`, `{event.sys.cpu}`, `{event.weather.emoji}`, `{event.weather.temp}`, `{event.disk}`

## Budget tracking

Track token usage against weekly/monthly limits:

```yaml
budget:
  weekly_tokens: 5_000_000
  monthly_tokens: 20_000_000
```

Fields: `{budget.weekly_pct|pct}`, `{budget.monthly_pct|pct}`

## License

MIT
