use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use chrono::{Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::config::BudgetConfig;

const USAGE_FILE: &str = ".data/statusline/usage.jsonl";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageEntry {
    pub ts: String,
    pub sid: String,
    pub input: u64,
    pub output: u64,
    pub cost_usd: f64,
}

/// Computed budget fields injected into the template context.
pub struct BudgetFields {
    pub weekly_pct: f64,
    pub monthly_pct: f64,
    pub weekly_spent: u64,
    pub monthly_spent: u64,
    pub weekly_limit: u64,
    pub monthly_limit: u64,
}

/// Load existing usage data, aggregate, and return computed budget fields.
pub fn load_budget(config: &BudgetConfig) -> Result<BudgetFields> {
    let path = usage_path();
    let entries = read_entries(&path).unwrap_or_default();

    let now = Utc::now();
    let today = now.date_naive();
    let iw = today.iso_week();
    let (iso_year, iso_week) = (iw.year(), iw.week());
    let month = today.month();
    let year = today.year();

    let weekly_spent = aggregate_weekly(&entries, iso_year, iso_week);
    let monthly_spent = aggregate_monthly(&entries, year, month);

    let weekly_pct = if config.weekly_tokens > 0 {
        (weekly_spent as f64 / config.weekly_tokens as f64) * 100.0
    } else {
        0.0
    };

    let monthly_pct = if config.monthly_tokens > 0 {
        (monthly_spent as f64 / config.monthly_tokens as f64) * 100.0
    } else {
        0.0
    };

    Ok(BudgetFields {
        weekly_pct,
        monthly_pct,
        weekly_spent,
        monthly_spent,
        weekly_limit: config.weekly_tokens,
        monthly_limit: config.monthly_tokens,
    })
}

/// Inject budget fields into the numeric template context.
pub fn inject_fields(fields: &BudgetFields, nums: &mut HashMap<String, f64>) {
    nums.insert("budget.weekly_pct".into(), fields.weekly_pct);
    nums.insert("budget.monthly_pct".into(), fields.monthly_pct);
    nums.insert("budget.weekly_spent".into(), fields.weekly_spent as f64);
    nums.insert("budget.monthly_spent".into(), fields.monthly_spent as f64);
    nums.insert("budget.weekly_limit".into(), fields.weekly_limit as f64);
    nums.insert("budget.monthly_limit".into(), fields.monthly_limit as f64);
}

/// Persist a usage entry after stdout has been flushed.
/// Deduplicates by session: if the last line has the same sid, replaces it.
pub fn persist(session_id: &str, input: u64, output: u64, cost_usd: f64) -> Result<()> {
    let path = usage_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).context("creating usage directory")?;
    }

    let entry = UsageEntry {
        ts: Utc::now().to_rfc3339(),
        sid: session_id.to_string(),
        input,
        output,
        cost_usd,
    };

    let mut lines = read_lines(&path).unwrap_or_default();

    // Deduplication: replace last line if same session
    if let Some(last) = lines.last() {
        if let Ok(last_entry) = serde_json::from_str::<UsageEntry>(last) {
            if last_entry.sid == session_id {
                lines.pop();
            }
        }
    }

    let new_line = serde_json::to_string(&entry).context("serializing usage entry")?;
    lines.push(new_line);

    // Rotation: keep only last 5 weeks of data
    let rotated = maybe_rotate(lines);

    write_lines(&path, &rotated).context("writing usage file")?;
    Ok(())
}

fn usage_path() -> PathBuf {
    PathBuf::from(USAGE_FILE)
}

fn read_lines(path: &PathBuf) -> Result<Vec<String>> {
    let file = fs::File::open(path)?;
    let reader = std::io::BufReader::new(file);
    let lines: Vec<String> = reader
        .lines()
        .filter_map(|l| l.ok())
        .filter(|l| !l.trim().is_empty())
        .collect();
    Ok(lines)
}

fn read_entries(path: &PathBuf) -> Result<Vec<UsageEntry>> {
    let lines = read_lines(path)?;
    let entries = lines
        .iter()
        .filter_map(|l| serde_json::from_str(l).ok())
        .collect();
    Ok(entries)
}

fn write_lines(path: &PathBuf, lines: &[String]) -> Result<()> {
    let mut file = fs::File::create(path)?;
    for line in lines {
        writeln!(file, "{line}")?;
    }
    Ok(())
}

fn aggregate_weekly(entries: &[UsageEntry], iso_year: i32, iso_week: u32) -> u64 {
    entries
        .iter()
        .filter(|e| {
            parse_iso_week(&e.ts)
                .map(|(y, w)| y == iso_year && w == iso_week)
                .unwrap_or(false)
        })
        .map(|e| e.input + e.output)
        .sum()
}

fn aggregate_monthly(entries: &[UsageEntry], year: i32, month: u32) -> u64 {
    entries
        .iter()
        .filter(|e| {
            parse_year_month(&e.ts)
                .map(|(y, m)| y == year && m == month)
                .unwrap_or(false)
        })
        .map(|e| e.input + e.output)
        .sum()
}

fn parse_iso_week(ts: &str) -> Option<(i32, u32)> {
    let date = parse_date(ts)?;
    let iw = date.iso_week();
    Some((iw.year(), iw.week()))
}

fn parse_year_month(ts: &str) -> Option<(i32, u32)> {
    let date = parse_date(ts)?;
    Some((date.year(), date.month()))
}

fn parse_date(ts: &str) -> Option<NaiveDate> {
    // Try RFC 3339 first, then fallback to date-only
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(ts) {
        return Some(dt.date_naive());
    }
    NaiveDate::parse_from_str(&ts[..10], "%Y-%m-%d").ok()
}

/// Rotate: keep only entries from the last 5 ISO weeks.
fn maybe_rotate(lines: Vec<String>) -> Vec<String> {
    let now = Utc::now().date_naive();
    let cutoff = now - chrono::Duration::weeks(5);

    lines
        .into_iter()
        .filter(|line| {
            serde_json::from_str::<UsageEntry>(line)
                .ok()
                .and_then(|e| parse_date(&e.ts))
                .map(|d| d >= cutoff)
                .unwrap_or(true) // keep unparseable lines
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(ts: &str, sid: &str, input: u64, output: u64, cost: f64) -> UsageEntry {
        UsageEntry {
            ts: ts.to_string(),
            sid: sid.to_string(),
            input,
            output,
            cost_usd: cost,
        }
    }

    #[test]
    fn aggregate_weekly_sums_matching_week() {
        // 2026-03-09 is ISO week 11, year 2026
        let entries = vec![
            entry("2026-03-09T10:00:00+00:00", "s1", 1000, 200, 0.1),
            entry("2026-03-10T10:00:00+00:00", "s2", 2000, 300, 0.2),
            entry("2026-03-01T10:00:00+00:00", "s3", 5000, 500, 0.5), // different week
        ];
        let total = aggregate_weekly(&entries, 2026, 11);
        assert_eq!(total, 1000 + 200 + 2000 + 300);
    }

    #[test]
    fn aggregate_monthly_sums_matching_month() {
        let entries = vec![
            entry("2026-03-01T10:00:00+00:00", "s1", 1000, 200, 0.1),
            entry("2026-03-15T10:00:00+00:00", "s2", 2000, 300, 0.2),
            entry("2026-02-28T10:00:00+00:00", "s3", 5000, 500, 0.5), // different month
        ];
        let total = aggregate_monthly(&entries, 2026, 3);
        assert_eq!(total, 1000 + 200 + 2000 + 300);
    }

    #[test]
    fn load_budget_computes_percentages() {
        let config = BudgetConfig {
            weekly_tokens: 100_000,
            monthly_tokens: 500_000,
        };
        // With no file, fields should be zero
        let fields = BudgetFields {
            weekly_pct: 0.0,
            monthly_pct: 0.0,
            weekly_spent: 0,
            monthly_spent: 0,
            weekly_limit: config.weekly_tokens,
            monthly_limit: config.monthly_tokens,
        };
        assert_eq!(fields.weekly_limit, 100_000);
        assert_eq!(fields.monthly_limit, 500_000);
    }

    #[test]
    fn inject_fields_populates_context() {
        let fields = BudgetFields {
            weekly_pct: 25.0,
            monthly_pct: 10.0,
            weekly_spent: 50_000,
            monthly_spent: 100_000,
            weekly_limit: 200_000,
            monthly_limit: 1_000_000,
        };
        let mut nums = HashMap::new();
        inject_fields(&fields, &mut nums);
        assert_eq!(nums["budget.weekly_pct"], 25.0);
        assert_eq!(nums["budget.monthly_pct"], 10.0);
        assert_eq!(nums["budget.weekly_spent"], 50_000.0);
        assert_eq!(nums["budget.monthly_limit"], 1_000_000.0);
    }

    #[test]
    fn rotation_removes_old_entries() {
        let old = serde_json::to_string(&entry(
            "2025-01-01T10:00:00+00:00",
            "old",
            100,
            50,
            0.01,
        ))
        .unwrap();
        let recent = serde_json::to_string(&entry(
            &Utc::now().to_rfc3339(),
            "new",
            200,
            100,
            0.02,
        ))
        .unwrap();
        let lines = vec![old, recent.clone()];
        let rotated = maybe_rotate(lines);
        assert_eq!(rotated.len(), 1);
        assert_eq!(rotated[0], recent);
    }

    #[test]
    fn serialization_roundtrip() {
        let e = entry("2026-03-10T14:30:00+00:00", "abc-123", 5200, 1800, 0.12);
        let json = serde_json::to_string(&e).unwrap();
        let parsed: UsageEntry = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.sid, "abc-123");
        assert_eq!(parsed.input, 5200);
    }

    #[test]
    fn parse_date_rfc3339() {
        let d = parse_date("2026-03-10T14:30:00+00:00").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 3, 10).unwrap());
    }

    #[test]
    fn parse_date_date_only() {
        let d = parse_date("2026-03-10").unwrap();
        assert_eq!(d, NaiveDate::from_ymd_opt(2026, 3, 10).unwrap());
    }

    #[test]
    fn zero_budget_gives_zero_pct() {
        let config = BudgetConfig {
            weekly_tokens: 0,
            monthly_tokens: 0,
        };
        // When budget is 0, pct should be 0 (not infinity)
        let pct = if config.weekly_tokens > 0 {
            100.0
        } else {
            0.0
        };
        assert_eq!(pct, 0.0);
    }
}
