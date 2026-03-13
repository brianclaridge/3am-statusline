# Rust conventions

Project-specific Rust patterns derived from the codebase. Follow these when writing or modifying Rust code.

## Error handling

- `anyhow::Result` for all fallible functions, `.context()` for meaningful error chains
- `let _ =` for fire-and-forget operations (file writes after stdout flush)
- Zero `unwrap()` in production code; `unwrap()` is acceptable in tests
- Return early with `?` rather than deeply nested `match` chains

## Do / Don't

| Do | Don't |
| --- | --- |
| `&Path` in function signatures | `&PathBuf` (unnecessary indirection) |
| `s.get(..10)` for safe slicing | `&s[..10]` (panics on short strings) |
| `chrono::TimeDelta` | `chrono::Duration` (deprecated) |
| `ureq` for HTTP requests | `Command::new("curl")` for HTTP |
| `Command::new("git").arg("status")` | `Command::new("sh").arg("-c").arg("git status")` for known commands |
| `Vec::with_capacity(n)` when size is known | `Vec::new()` + repeated push |
| `.clamp(min, max)` for range bounding | manual `if` chains for clamping |
| `temp_env` crate in tests | `std::env::set_var` (unsafe in edition 2024) |
| `String::with_capacity()` for template output | Default `String::new()` for large builds |

## Module patterns

- **Event modules** follow `gather() + run()`: `gather()` collects data into a struct, `run()` serializes and prints
- Serde structs use `#[derive(Serialize)]` (events) or `#[derive(Deserialize)]` (config), not both unless needed
- `#[serde(default)]` for optional config fields with fallback values
- `#[serde(skip_serializing_if = "Option::is_none")]` to keep JSON output clean

## Performance

- Flush stdout before expensive work (event firing, rate limit refresh, budget persistence)
- Fire events and refresh caches only when stale — check timestamps before spawning processes
- `serde_json::from_str` once, reuse the parsed value
- Prefer `HashMap` for template context; `BTreeMap` only when output ordering matters (time zones)

## Safety

- Zero `unsafe` blocks
- `Command::new("git").arg(x)` — never interpolate args into a shell string
- ANSI escape validation: `strip_ansi_len()` and `truncate_visible()` handle malformed sequences gracefully
- Cap captured stdout to `MAX_STDOUT_BYTES` (1024) to prevent memory blowups from rogue commands
- Log file rotation via `keep` limit and 5-week JSONL pruning

## Platform

- `#[cfg(windows)]` / `#[cfg(not(windows))]` for shell dispatch (`cmd /C` vs `sh -c`)
- `Path::join()` for path construction, never string concatenation with `/`
- `std::io::IsTerminal` for TTY detection (replaces `atty`)
- `NO_COLOR` / `FORCE_COLOR` env var conventions respected

## Testing

- `unwrap()` is fine in tests — panics give clear failure messages
- Use `Mutex` for tests that mutate env vars (`static ENV_LOCK: Mutex<()>`)
- Migrate `std::env::set_var` calls to `temp_env` crate for thread safety
- Test both colored and non-colored output paths
- Event `fire()` tests use real shell commands (`echo hello`) — keep them fast

## Known tech debt

| Item | Location | Issue |
| --- | --- | --- |
| Deprecated `Duration` | `budget.rs:194` | Use `TimeDelta::weeks(5)` instead |
| `curl` subprocess | `ratelimit.rs:86` | Replace with `ureq` (already a dep) |
| `&PathBuf` params | `budget.rs:120,131,140` | Change to `&Path` |
| `#[allow(dead_code)]` | `config.rs:54` | `ThresholdConfig.green` read by serde but unused in logic |
| `set_var` in tests | `config.rs:302` | Migrate to `temp_env` crate |
| Shell wrapper for events | `events.rs:119` | Consider `Command::new` for built-in event commands |
| Missing `with_capacity` | `event/git.rs:38,74` | Small vecs, low priority |
