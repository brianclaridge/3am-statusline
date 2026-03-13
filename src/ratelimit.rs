//! Rate limit data from Anthropic OAuth usage API.
//!
//! Reads cached data from `.data/statusline/ratelimits.json` (fast path).
//! Refreshes cache by calling the usage endpoint AFTER stdout is flushed.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

const CACHE_MAX_AGE_SECS: i64 = 300; // 5 minutes
const USAGE_URL: &str = "https://api.anthropic.com/api/oauth/usage";

#[derive(Debug, Serialize, Deserialize)]
pub struct RateLimitCache {
    pub claims: HashMap<String, ClaimData>,
    pub status: String,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ClaimData {
    /// Utilization as a percentage (0-100).
    pub utilization: f64,
    /// Unix timestamp when this claim resets.
    pub reset: i64,
}

/// Response from the OAuth usage API.
#[derive(Debug, Deserialize)]
struct UsageResponse {
    five_hour: Option<UsageClaim>,
    seven_day: Option<UsageClaim>,
}

#[derive(Debug, Deserialize)]
struct UsageClaim {
    utilization: f64,
    resets_at: String,
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
        // utilization is already a percentage (0-100)
        nums.insert(format!("ratelimit.{claim}"), data.utilization);
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

/// Parse an ISO 8601 timestamp to a unix epoch.
fn parse_reset_time(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp())
}

/// Refresh the cache by calling the OAuth usage API. Blocks briefly.
/// Call AFTER stdout is flushed so rendering is not delayed.
pub fn refresh(state_dir: &str) {
    let token = match read_api_token() {
        Some(t) => t,
        None => return,
    };

    let body = match ureq::get(USAGE_URL)
        .header("Authorization", &format!("Bearer {token}"))
        .header("anthropic-beta", "oauth-2025-04-20")
        .header("Content-Type", "application/json")
        .call()
    {
        Ok(mut r) => match r.body_mut().read_to_string() {
            Ok(s) => s,
            Err(_) => return,
        },
        Err(_) => return,
    };

    let usage: UsageResponse = match serde_json::from_str(&body) {
        Ok(u) => u,
        Err(_) => return,
    };

    let mut claims = HashMap::new();

    if let Some(claim) = usage.five_hour {
        let reset = parse_reset_time(&claim.resets_at).unwrap_or(0);
        claims.insert(
            "5h".to_string(),
            ClaimData {
                utilization: claim.utilization,
                reset,
            },
        );
    }

    if let Some(claim) = usage.seven_day {
        let reset = parse_reset_time(&claim.resets_at).unwrap_or(0);
        claims.insert(
            "7d".to_string(),
            ClaimData {
                utilization: claim.utilization,
                reset,
            },
        );
    }

    if claims.is_empty() {
        return;
    }

    let cache = RateLimitCache {
        claims,
        status: "ok".to_string(),
        updated_at: chrono::Utc::now().timestamp(),
    };

    let path = cache_path(state_dir);
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(&cache) {
        let _ = std::fs::write(path, json);
    }
}

fn read_api_token() -> Option<String> {
    let config_dir = std::env::var("CLAUDE_CONFIG_DIR").ok()?;
    let path = Path::new(&config_dir).join(".credentials.json");
    let content = std::fs::read_to_string(path).ok()?;
    let creds: serde_json::Value = serde_json::from_str(&content).ok()?;
    creds["claudeAiOauth"]["accessToken"]
        .as_str()
        .map(String::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_reset_time_iso8601() {
        let ts = parse_reset_time("2026-03-13T07:00:01.188734+00:00").unwrap();
        assert!(ts > 0);
    }

    #[test]
    fn parse_reset_time_invalid() {
        assert!(parse_reset_time("not-a-date").is_none());
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
            status: "ok".into(),
            updated_at: chrono::Utc::now().timestamp(),
        };
        cache.claims.insert(
            "5h".into(),
            ClaimData {
                utilization: 5.0,
                reset: chrono::Utc::now().timestamp() + 3600,
            },
        );

        let mut nums = HashMap::new();
        let mut strs = HashMap::new();
        inject_fields(&cache, &mut nums, &mut strs);

        // utilization is already a percentage
        assert!((nums["ratelimit.5h"] - 5.0).abs() < f64::EPSILON);
        assert!(strs.contains_key("ratelimit.5h_eta"));
        assert_eq!(strs["ratelimit.status"], "ok");
    }

    #[test]
    fn deserialize_usage_response() {
        let json = r#"{
            "five_hour": {"utilization": 1.0, "resets_at": "2026-03-13T07:00:01+00:00"},
            "seven_day": {"utilization": 23.0, "resets_at": "2026-03-13T04:00:00+00:00"},
            "seven_day_oauth_apps": null,
            "seven_day_opus": null,
            "seven_day_sonnet": {"utilization": 2.0, "resets_at": "2026-03-15T19:00:00+00:00"},
            "seven_day_cowork": null,
            "iguana_necktie": null,
            "extra_usage": {"is_enabled": true, "monthly_limit": 5000, "used_credits": 0.0, "utilization": null}
        }"#;
        let resp: UsageResponse = serde_json::from_str(json).unwrap();
        assert!((resp.five_hour.as_ref().unwrap().utilization - 1.0).abs() < f64::EPSILON);
        assert!((resp.seven_day.as_ref().unwrap().utilization - 23.0).abs() < f64::EPSILON);
    }
}
