# Rust conventions

Project-specific Rust patterns derived from the codebase. Follow these when writing or modifying Rust code.

## Toolchain: Rust 1.94.0 (2026-03-05)

Notable stabilizations relevant to this project:

| Feature | Use case |
| --- | --- |
| `<[T]>::array_windows` | Const-length sliding window on slices — prefer over `.windows(N)` when N is known at compile time |
| `Peekable::next_if_map` | Conditional consume + transform — useful for template/parser code |
| `LazyLock::get` / `LazyCell::get` | Inspect lazy-initialized values without forcing — useful if we adopt lazy statics |
| Cargo `include` key | Split `.cargo/config.toml` into shared + per-target files for cross-compile |
| Closure capturing changes | Closures now capture partial variables more precisely — may surface new borrow checker errors in closures that previously moved entire structs |
| TOML v1.1 in Cargo | `Cargo.toml` supports inline tables, new escape sequences — raises MSRV if used |

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
- Color defaults to enabled — stdout is piped by Claude Code, so `is_terminal()` is wrong; only `NO_COLOR` env var disables color

## Testing

- `unwrap()` is fine in tests — panics give clear failure messages
- Use `Mutex` for tests that mutate env vars (`static ENV_LOCK: Mutex<()>`)
- Migrate `std::env::set_var` calls to `temp_env` crate for thread safety
- Test both colored and non-colored output paths
- Event `fire()` tests use real shell commands (`echo hello`) — keep them fast

## Known tech debt

| Item | Location | Issue |
| --- | --- | --- |
| `#[allow(dead_code)]` | `config.rs:65` | `ThresholdConfig.green` read by serde but unused in logic |
| Shell wrapper for events | `events.rs:129` | Consider `Command::new` for built-in event commands |
| Missing `with_capacity` | `event/git.rs:47,91` | Small vecs, low priority |
