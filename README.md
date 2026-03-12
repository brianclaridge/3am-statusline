# 3am-statusline

A configurable status bar for [Claude Code](https://docs.anthropic.com/en/docs/claude-code) sessions. Displays model info, cost, context usage, rate limits, budget tracking, and custom event data.

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/brianclaridge/3am-statusline/main/install.sh | bash
```

Or clone and build from source:

```bash
git clone https://github.com/brianclaridge/3am-statusline.git
cd 3am-statusline
task build
```

## Setup

Add to your project's `.claude/settings.json`:

```json
{
  "statusLine": {
    "type": "command",
    "command": "node /path/to/3am-statusline/shim.js",
    "padding": 1
  }
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

Run shell commands at intervals and inject their output into the statusline:

```yaml
events:
  - name: branch
    command: "git rev-parse --abbrev-ref HEAD 2>/dev/null"
    interval: 10s
    capture: true

  - name: disk
    command: "df -h / | tail -1 | awk '{print $5}'"
    interval: 5m
    capture: true
```

Use in templates: `{event.branch|dim}`, `{event.disk|yellow}`

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
