# Configuration

Create `config/statusline.yml` or `.claude/statusline.yml` in your project root. See `config/statusline.yml` for a full example.

## Config search order

1. `$STATUSLINE_CONFIG` (explicit override via env var)
2. `config/statusline.yml` (project-local)
3. `.claude/statusline.yml` (legacy project-local)
4. `$CLAUDE_CONFIG_DIR/statusline.yml` (per-user)
5. Built-in default (hardcoded two-line layout)

## Sections

Six top-level keys: `lines`, `meter`, `color_fields`, `events`, `logging`, `columns`.

### Lines

Each line has a `left` and optional `right` template string. Left and right are padded to fill the terminal width.

```yaml
lines:
  - left: "{model.display_name}"
    right: "{cost.total_cost_usd|currency}"
  - left: "{meter:context_window.used_percentage} {context_window.used_percentage|pct} ctx"
    right: "{context_window.total_input_tokens|tokens} tok"
```

### Meter

Controls the visual meter bar rendered by `{meter:field}`.

```yaml
meter:
  width: 10
  filled: "●"
  empty: "○"
  thresholds:
    green: 0
    yellow: 60
    red: 85
```

### Theme

Maps color names to ANSI codes. Referenced by `{c:name}...{/c}` tags and the `{sep}` token.

```yaml
theme:
  model: "1"         # bold
  model_name: "1;36" # bold cyan
  dim: "2"
  sep: "2"           # dim separator
  sep_char: "|"      # separator character (default |)
```

### Columns

Override terminal width for line padding. Resolution order: `columns` config > `COLUMNS` env var > 80.

```yaml
columns: 120
```

### Logging

```yaml
logging:
  file: ".data/statusline/statusline.log"
  json:
    dir: ".data/statusline/json"
    keep: 15
```

## Template tokens

| Syntax | Example | Output |
| --- | --- | --- |
| `{field}` | `{model.display_name}` | `Opus 4.6` |
| `{field\|format}` | `{cost.total_cost_usd\|currency}` | `$0.55` |
| `{field\|color}` | `{model.display_name\|dim}` | dim text |
| `{field\|format\|color}` | `{cost.total_cost_usd\|currency\|green}` | green `$0.55` |
| `{meter:field}` | `{meter:context_window.used_percentage}` | `[●●●○○○○○○○]` |
| `{sep}` | `{sep}` | themed separator (default `\|`) |
| `{event.name}` | `{event.git.branch}` | `main` |
| `{c:name}...{/c}` | `{c:green}ok{/c}` | green "ok" |
| `{c:code}...{/c}` | `{c:1;33}warn{/c}` | bold yellow "warn" |

### Format specifiers

| Specifier | Input | Output |
| --- | --- | --- |
| `currency` | `0.55` | `$0.55` |
| `pct` | `8.3` | `8%` |
| `duration` | `45` | `45s` |
| `tokens` | `15234` | `15.2K` |
| `comma` | `15234` | `15,234` |

## Available fields

### Model

`model.display_name`, `model.id`

### Cost

`cost.total_cost_usd`, `cost.total_duration_secs`, `cost.total_lines_added`, `cost.total_lines_removed`

### Context window

`context_window.used_percentage`, `context_window.total_input_tokens`, `context_window.total_output_tokens`, `context_window.remaining_percentage`, `context_window.context_window_size`

### Current usage (per-turn)

`current_usage.input_tokens`, `current_usage.output_tokens`, `current_usage.cache_creation_input_tokens`, `current_usage.cache_read_input_tokens`

### Session

`session_id`, `version`, `build_version`, `cwd`

### Optional

`vim.mode`, `agent.name`, `worktree.name`, `plan.slug`

### Rate limits

`ratelimit.5h`, `ratelimit.7d`, `ratelimit.5h_eta`, `ratelimit.7d_eta`

### Version check

`version.ok` (✓ when current), `version.outdated` (⇡ when update available) — requires `version` event

### Events

`event.{name}` — any field from event JSON output (e.g., `{event.git.branch}`, `{event.sys.cpu}`)

## Color fields

Threshold-based coloring for inline values. Each color field generates three template slots (`_green`, `_yellow`, `_red`) — only the active tier is populated.

```yaml
color_fields:
  - name: ctx_pct
    source: "context_window.used_percentage"
    format: "pct"
    yellow: 40
    red: 60
```

Use in templates:

```yaml
- left: "{c:green}{ctx_pct_green}{/c}{c:1;33}{ctx_pct_yellow}{/c}{c:1;31}{ctx_pct_red}{/c} ctx"
```

Meters (`{meter:field}`) with a matching color field automatically use the same thresholds, so dot colors and text colors stay in sync.
