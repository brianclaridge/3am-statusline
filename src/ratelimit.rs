//! Rate limit data from Anthropic API response headers.
//!
//! Reads cached data from `.data/statusline/ratelimits.json` (fast path).
//! Refreshes cache by calling the API with curl AFTER stdout is flushed.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

const CACHE_MAX_AGE_SECS: i64 = 300; // 5 minutes
const API_URL: &str = "https://api.anthropic.com/v1/messages";
const QUOTA_BODY: &str = r#"{"model":"claude-haiku-4-5-20251001","max_tokens":1,"messages":[{"role":"user","content":"ok"}]}"#;
const CLAIMS: &[&str] = &["5h", "7d"];

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitCache {
    pub claims: HashMap<String, ClaimData>,
    pub status: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimData {
    pub utilization: f64,
    pub reset: i64,
}

fn cache_path(state_dir: &str) -> std::path::PathBuf {
    Path::new(state_dir).join("ratelimits.json")
}

/// Load cached rate limit data. Returns None if missing or unreadable.
pub fn load_cached(state_dir: &str) -> Option<RateLimitCache> {
    std::fs::read_to_string(cache_path(state_dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
}

/// True if cache is missing or older than 5 minutes.
pub fn is_stale(cache: &Option<RateLimitCache>) -> bool {
    match cache {
        None => true,
        Some(c) => chrono::Utc::now().timestamp() - c.updated_at > CACHE_MAX_AGE_SECS,
    }
}

/// Inject rate limit template fields into the context maps.
pub fn inject_fields(
    cache: &RateLimitCache,
    nums: &mut HashMap<String, f64>,
    strings: &mut HashMap<String, String>,
) {
    let now = chrono::Utc::now().timestamp();
    for (claim, data) in &cache.claims {
        nums.insert(format!("ratelimit.{claim}"), data.utilization * 100.0);
        let eta = (data.reset - now).max(0);
        strings.insert(format!("ratelimit.{claim}_eta"), format_eta(eta));
    }
    strings.insert("ratelimit.status".into(), cache.status.clone());
}

fn format_eta(secs: i64) -> String {
    if secs <= 0 {
        return "now".into();
    }
    let h = secs / 3600;
    let m = (secs % 3600) / 60;
    if h > 24 {
        format!("{}d {}h", h / 24, h % 24)
    } else if h > 0 {
        format!("{h}h {m}m")
    } else {
        format!("{m}m")
    }
}

/// Refresh the cache by calling the API with curl. Blocks ~500ms.
/// Call AFTER stdout is flushed so rendering is not delayed.
pub fn refresh(state_dir: &str) {
    let token = match read_api_token() {
        Some(t) => t,
        None => return,
    };

    let output = match std::process::Command::new("curl")
        .args([
            "-s",
            "-D-",
            "-o",
            "/dev/null",
            "--max-time",
            "5",
            API_URL,
            "-H",
            &format!("x-api-key: {token}"),
            "-H",
            "anthropic-version: 2023-06-01",
            "-H",
            "content-type: application/json",
            "-d",
            QUOTA_BODY,
        ])
        .output()
    {
        Ok(o) => o,
        Err(_) => return,
    };

    let stdout = match String::from_utf8(output.stdout) {
        Ok(s) => s,
        Err(_) => return,
    };

    if let Some(cache) = parse_headers(&stdout) {
        let path = cache_path(state_dir);
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(json) = serde_json::to_string_pretty(&cache) {
            let _ = std::fs::write(path, json);
        }
    }
}

fn read_api_token() -> Option<String> {
    let config_dir = std::env::var("CLAUDE_CONFIG_DIR").ok()?;
    let path = Path::new(&config_dir).join(".credentials.json");
    let content = std::fs::read_to_string(path).ok()?;
    let creds: serde_json::Value = serde_json::from_str(&content).ok()?;
    creds["claudeAiOauth"]["accessToken"].as_str().map(String::from)
}

fn parse_headers(raw: &str) -> Option<RateLimitCache> {
    let mut claims = HashMap::new();
    let mut status = "allowed".to_string();

    for line in raw.lines() {
        let line = line.trim();

        if let Some(val) = line.strip_prefix("anthropic-ratelimit-unified-status: ") {
            status = val.to_string();
            continue;
        }

        for &claim in CLAIMS {
            let util_prefix = format!("anthropic-ratelimit-unified-{claim}-utilization: ");
            if let Some(val) = line.strip_prefix(&util_prefix) {
                if let Ok(u) = val.parse::<f64>() {
                    claims
                        .entry(claim.to_string())
                        .or_insert(ClaimData {
                            utilization: 0.0,
                            reset: 0,
                        })
                        .utilization = u;
                }
            }
            let reset_prefix = format!("anthropic-ratelimit-unified-{claim}-reset: ");
            if let Some(val) = line.strip_prefix(&reset_prefix) {
                if let Ok(r) = val.parse::<i64>() {
                    claims
                        .entry(claim.to_string())
                        .or_insert(ClaimData {
                            utilization: 0.0,
                            reset: 0,
                        })
                        .reset = r;
                }
            }
        }
    }

    if claims.is_empty() {
        return None;
    }

    Some(RateLimitCache {
        claims,
        status,
        updated_at: chrono::Utc::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_real_headers() {
        let headers = "\
            HTTP/2 200\r\n\
            anthropic-ratelimit-unified-status: allowed\r\n\
            anthropic-ratelimit-unified-5h-status: allowed\r\n\
            anthropic-ratelimit-unified-5h-reset: 1773288000\r\n\
            anthropic-ratelimit-unified-5h-utilization: 0.05\r\n\
            anthropic-ratelimit-unified-7d-status: allowed\r\n\
            anthropic-ratelimit-unified-7d-reset: 1773374400\r\n\
            anthropic-ratelimit-unified-7d-utilization: 0.15\r\n";
        let cache = parse_headers(headers).unwrap();
        assert_eq!(cache.status, "allowed");
        assert_eq!(cache.claims.len(), 2);
        assert!((cache.claims["5h"].utilization - 0.05).abs() < f64::EPSILON);
        assert_eq!(cache.claims["5h"].reset, 1773288000);
        assert!((cache.claims["7d"].utilization - 0.15).abs() < f64::EPSILON);
        assert_eq!(cache.claims["7d"].reset, 1773374400);
    }

    #[test]
    fn parse_missing_claims_returns_none() {
        let headers = "HTTP/2 200\r\nserver: cloudflare\r\n";
        assert!(parse_headers(headers).is_none());
    }

    #[test]
    fn format_eta_ranges() {
        assert_eq!(format_eta(0), "now");
        assert_eq!(format_eta(-10), "now");
        assert_eq!(format_eta(300), "5m");
        assert_eq!(format_eta(3661), "1h 1m");
        assert_eq!(format_eta(90000), "1d 1h");
    }

    #[test]
    fn inject_populates_context() {
        let mut cache = RateLimitCache {
            claims: HashMap::new(),
            status: "allowed".into(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        cache.claims.insert(
            "5h".into(),
            ClaimData {
                utilization: 0.05,
                reset: chrono::Utc::now().timestamp() + 3600,
            },
        );

        let mut nums = HashMap::new();
        let mut strs = HashMap::new();
        inject_fields(&cache, &mut nums, &mut strs);

        assert!((nums["ratelimit.5h"] - 5.0).abs() < f64::EPSILON);
        assert!(strs.contains_key("ratelimit.5h_eta"));
        assert_eq!(strs["ratelimit.status"], "allowed");
    }
}
