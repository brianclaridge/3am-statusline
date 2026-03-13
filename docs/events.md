# Events

Timer-based command execution. The binary includes built-in event subcommands, and you can run arbitrary shell commands.

## Built-in events

```bash
3am-statusline event git                  # {"branch":"main","sha":"2620eb7","sync":"↑1","dirty":"~3 +1 ?2"}
3am-statusline event time                 # {"pst":"3:45","pst_icon":"☀️","mst":"5:45","mst_icon":"🌅",...}
3am-statusline event sys                  # {"cpu":"12%","cores":"8","mem":"4.2/16G (26%)","mem_pct":"26%","mem_used":"4G","mem_total":"16G"}
3am-statusline event status               # 🟢 ok
3am-statusline event version              # {"latest":"2.1.74"}
3am-statusline event weather --zip 98101  # {"emoji":"🌧️","temp":"39°F","condition":"drizzle"}
```

### Git

Branch name, short SHA, ahead/behind sync indicator, dirty file counts with symbols (`~` modified, `+` added, `-` deleted, `?` untracked).

### Time

World clocks for PST, MST, CST, EST with day/night emoji indicators based on hour.

### Sys

CPU percentage, core count, memory usage with used/total and percentage. Uses `sysinfo` crate for cross-platform support.

### Status

Checks `status.claude.com` API. Returns emoji + status text.

### Version

Fetches the latest Claude Code release tag from `api.github.com/repos/anthropics/claude-code/releases/latest`. Outputs `{"latest":"X.Y.Z"}`. At render time, the binary compares the payload's `version` field against this and injects `version.ok` (✓) or `version.outdated` (⇡).

### Weather

Current conditions from [Open-Meteo](https://open-meteo.com/) (free, no API key). Requires `--zip` flag with a US zip code. Geocoding via Zippopotam.us, cached permanently in `.data/statusline/geocode.json`. Outputs temperature with threshold coloring fields (`temp_cold`, `temp_warm`, `temp_hot`) and weather condition emoji.

## Config wiring

```yaml
events:
  - name: git
    command: "${CLAUDE_PLUGIN_ROOT}/bin/release/3am-statusline event git"
    interval: 5s
    capture: true
  - name: version
    command: "${CLAUDE_PLUGIN_ROOT}/bin/release/3am-statusline event version"
    interval: 5m
    capture: true
```

| `capture` | Behavior |
| --- | --- |
| `true` | Blocks, captures stdout, injects as `{event.name}` |
| `false` | Fire-and-forget, no stdout capture |

## Custom events

Any shell command can be an event:

```yaml
  - name: disk
    command: "df -h / | tail -1 | awk '{print $5}'"
    interval: 5m
    capture: true
```

JSON output is expanded into dotted fields (`{event.name.field}`). Plain text output is available as `{event.name}`.

## Template usage

```
{event.git.branch}         # main
{event.sys.cpu}            # 12%
{event.weather.emoji}      # 🌧️
{event.weather.temp}       # 39°F
{event.disk}               # 45%
```

## Cache

Event results are cached in `.data/statusline/events.json`. Events only fire when their interval elapses.
