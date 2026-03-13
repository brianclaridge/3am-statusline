//! Timer-based event system. Runs shell commands at configurable intervals
//! and injects their stdout into the template context as `event.{name}`.
//!
//! Sentinel file: `.data/statusline/events.json`

use std::collections::HashMap;
use std::path::Path;

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use crate::config::EventConfig;

const MAX_STDOUT_BYTES: usize = 1024;

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct EventCache {
    #[serde(flatten)]
    pub entries: HashMap<String, EventEntry>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EventEntry {
    pub fired_at: i64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    pub exit_code: i32,
}

/// Parse an interval string like `500ms`, `3s`, `1m`, `5m`, `1h` into seconds.
pub fn parse_interval(s: &str) -> Result<i64> {
    let s = s.trim();
    if let Some(rest) = s.strip_suffix("ms") {
        let val: i64 = rest.parse().map_err(|_| anyhow::anyhow!("invalid ms: {s}"))?;
        // Sub-second intervals round down to 0 seconds
        Ok(val / 1000)
    } else if let Some(rest) = s.strip_suffix('h') {
        let val: i64 = rest.parse().map_err(|_| anyhow::anyhow!("invalid hours: {s}"))?;
        Ok(val * 3600)
    } else if let Some(rest) = s.strip_suffix('m') {
        let val: i64 = rest.parse().map_err(|_| anyhow::anyhow!("invalid minutes: {s}"))?;
        Ok(val * 60)
    } else if let Some(rest) = s.strip_suffix('s') {
        let val: i64 = rest.parse().map_err(|_| anyhow::anyhow!("invalid seconds: {s}"))?;
        Ok(val)
    } else {
        bail!("unknown interval format: {s} (expected 500ms, 3s, 1m, 5m, 1h)")
    }
}

/// Load event cache from disk.
pub fn load_cache(state_dir: &str) -> Option<EventCache> {
    let path = format!("{state_dir}/events.json");
    std::fs::read_to_string(path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// Save event cache to disk.
pub fn save_cache(cache: &EventCache, state_dir: &str) {
    let path_str = format!("{state_dir}/events.json");
    let path = Path::new(&path_str);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(cache) {
        let _ = std::fs::write(path, json);
    }
}

/// Return events that are due to fire (stale or never fired).
pub fn due_events<'a>(configs: &'a [EventConfig], cache: &EventCache) -> Vec<&'a EventConfig> {
    let now = chrono::Utc::now().timestamp();
    configs
        .iter()
        .filter(|cfg| {
            let interval_secs = parse_interval(&cfg.interval).unwrap_or(60);
            match cache.entries.get(&cfg.name) {
                None => true,
                Some(entry) => now - entry.fired_at >= interval_secs,
            }
        })
        .collect()
}

/// Inject cached event stdout values into the template context as `event.{name}`.
/// If stdout is a JSON object, each key is also injected as `event.{name}.{key}`.
pub fn inject_fields(
    cache: &EventCache,
    _nums: &mut HashMap<String, f64>,
    strings: &mut HashMap<String, String>,
) {
    for (name, entry) in &cache.entries {
        if let Some(ref stdout) = entry.stdout {
            strings.insert(format!("event.{name}"), stdout.clone());
            // JSON expansion: if stdout is a JSON object, inject each key
            if let Ok(serde_json::Value::Object(map)) = serde_json::from_str(stdout) {
                for (key, val) in &map {
                    let s = match val {
                        serde_json::Value::String(s) => s.clone(),
                        other => other.to_string(),
                    };
                    strings.insert(format!("event.{name}.{key}"), s);
                }
            }
        }
    }
}

/// Execute due events and update the cache.
pub fn fire(events: &[&EventConfig], cache: &mut EventCache) {
    let now = chrono::Utc::now().timestamp();

    for cfg in events {
        let (shell, flag) = shell_command();

        if cfg.capture {
            // Blocking: capture stdout
            let result = std::process::Command::new(shell)
                .arg(flag)
                .arg(&cfg.command)
                .output();

            let (stdout, exit_code) = match result {
                Ok(output) => {
                    let mut s = String::from_utf8_lossy(&output.stdout).into_owned();
                    s.truncate(MAX_STDOUT_BYTES);
                    // Trim trailing newline
                    while s.ends_with('\n') || s.ends_with('\r') {
                        s.pop();
                    }
                    let code = output.status.code().unwrap_or(-1);
                    (Some(s), code)
                }
                Err(_) => (None, -1),
            };

            cache.entries.insert(
                cfg.name.clone(),
                EventEntry {
                    fired_at: now,
                    stdout,
                    exit_code,
                },
            );
        } else {
            // Fire-and-forget: spawn and drop
            let _ = std::process::Command::new(shell)
                .arg(flag)
                .arg(&cfg.command)
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();

            cache.entries.insert(
                cfg.name.clone(),
                EventEntry {
                    fired_at: now,
                    stdout: None,
                    exit_code: 0,
                },
            );
        }
    }
}

#[cfg(not(windows))]
fn shell_command() -> (&'static str, &'static str) {
    ("sh", "-c")
}

#[cfg(windows)]
fn shell_command() -> (&'static str, &'static str) {
    ("cmd", "/C")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_seconds() {
        assert_eq!(parse_interval("3s").unwrap(), 3);
        assert_eq!(parse_interval("60s").unwrap(), 60);
    }

    #[test]
    fn parse_interval_minutes() {
        assert_eq!(parse_interval("1m").unwrap(), 60);
        assert_eq!(parse_interval("5m").unwrap(), 300);
    }

    #[test]
    fn parse_interval_hours() {
        assert_eq!(parse_interval("1h").unwrap(), 3600);
        assert_eq!(parse_interval("2h").unwrap(), 7200);
    }

    #[test]
    fn parse_interval_milliseconds() {
        assert_eq!(parse_interval("500ms").unwrap(), 0);
        assert_eq!(parse_interval("1500ms").unwrap(), 1);
    }

    #[test]
    fn parse_interval_invalid() {
        assert!(parse_interval("abc").is_err());
        assert!(parse_interval("").is_err());
    }

    #[test]
    fn due_events_never_fired() {
        let configs = vec![EventConfig {
            name: "test".into(),
            command: "echo hi".into(),
            interval: "10s".into(),
            capture: true,
        }];
        let cache = EventCache::default();
        let due = due_events(&configs, &cache);
        assert_eq!(due.len(), 1);
    }

    #[test]
    fn due_events_recently_fired() {
        let configs = vec![EventConfig {
            name: "test".into(),
            command: "echo hi".into(),
            interval: "1h".into(),
            capture: true,
        }];
        let mut cache = EventCache::default();
        cache.entries.insert(
            "test".into(),
            EventEntry {
                fired_at: chrono::Utc::now().timestamp(),
                stdout: Some("hi".into()),
                exit_code: 0,
            },
        );
        let due = due_events(&configs, &cache);
        assert!(due.is_empty());
    }

    #[test]
    fn inject_populates_strings() {
        let mut cache = EventCache::default();
        cache.entries.insert(
            "branch".into(),
            EventEntry {
                fired_at: 0,
                stdout: Some("main".into()),
                exit_code: 0,
            },
        );
        let mut nums = HashMap::new();
        let mut strs = HashMap::new();
        inject_fields(&cache, &mut nums, &mut strs);
        assert_eq!(strs["event.branch"], "main");
    }

    #[test]
    fn fire_captures_stdout() {
        let configs = vec![EventConfig {
            name: "echo_test".into(),
            command: "echo hello".into(),
            interval: "1s".into(),
            capture: true,
        }];
        let mut cache = EventCache::default();
        let refs: Vec<&EventConfig> = configs.iter().collect();
        fire(&refs, &mut cache);
        let entry = &cache.entries["echo_test"];
        assert_eq!(entry.stdout.as_deref(), Some("hello"));
        assert_eq!(entry.exit_code, 0);
    }

    #[test]
    fn inject_expands_json_object() {
        let mut cache = EventCache::default();
        cache.entries.insert(
            "git".into(),
            EventEntry {
                fired_at: 0,
                stdout: Some(r#"{"branch":"main","sync":"+4","dirty":"3M 1S"}"#.into()),
                exit_code: 0,
            },
        );
        let mut nums = HashMap::new();
        let mut strs = HashMap::new();
        inject_fields(&cache, &mut nums, &mut strs);
        // Raw JSON preserved
        assert!(strs["event.git"].starts_with('{'));
        // Sub-keys expanded
        assert_eq!(strs["event.git.branch"], "main");
        assert_eq!(strs["event.git.sync"], "+4");
        assert_eq!(strs["event.git.dirty"], "3M 1S");
    }

    #[test]
    fn inject_plain_text_no_expansion() {
        let mut cache = EventCache::default();
        cache.entries.insert(
            "branch".into(),
            EventEntry {
                fired_at: 0,
                stdout: Some("main".into()),
                exit_code: 0,
            },
        );
        let mut nums = HashMap::new();
        let mut strs = HashMap::new();
        inject_fields(&cache, &mut nums, &mut strs);
        assert_eq!(strs["event.branch"], "main");
        // No sub-keys for plain text
        assert_eq!(strs.len(), 1);
    }

    #[test]
    fn fire_no_capture() {
        let configs = vec![EventConfig {
            name: "bg_test".into(),
            command: "echo background".into(),
            interval: "1s".into(),
            capture: false,
        }];
        let mut cache = EventCache::default();
        let refs: Vec<&EventConfig> = configs.iter().collect();
        fire(&refs, &mut cache);
        let entry = &cache.entries["bg_test"];
        assert!(entry.stdout.is_none());
    }
}
