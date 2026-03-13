use std::collections::BTreeMap;

use anyhow::Result;
use chrono::Utc;
use chrono_tz::Tz;

const ZONES: &[(&str, &str)] = &[
    ("PST", "America/Los_Angeles"),
    ("MST", "America/Denver"),
    ("CST", "America/Chicago"),
    ("EST", "America/New_York"),
    ("UTC", "UTC"),
    ("GMT", "Europe/London"),
    ("CET", "Europe/Paris"),
    ("JST", "Asia/Tokyo"),
    ("IST", "Asia/Kolkata"),
    ("AEST", "Australia/Sydney"),
];

const DEFAULT_ZONES: &[&str] = &["PST", "MST", "CST", "EST"];

fn format_time(tz: Tz) -> String {
    let now = Utc::now().with_timezone(&tz);
    let h = {
        let h12 = now.format("%I").to_string().trim_start_matches('0').to_string();
        if h12.is_empty() { "12".to_string() } else { h12 }
    };
    let m = now.format("%M").to_string();
    let ap = if now.format("%p").to_string() == "AM" { "a" } else { "p" };
    format!("{h}:{m}{ap}")
}

fn resolve_tz(label: &str) -> Option<Tz> {
    let upper = label.to_uppercase();
    ZONES
        .iter()
        .find(|(k, _)| *k == upper)
        .map(|(_, iana)| iana)
        .or(Some(&label))
        .and_then(|iana| iana.parse::<Tz>().ok())
}

pub fn run() -> Result<()> {
    let mut data = BTreeMap::new();
    for zone in DEFAULT_ZONES {
        let key = zone.to_lowercase();
        let val = resolve_tz(zone)
            .map(format_time)
            .unwrap_or_else(|| "???".to_string());
        data.insert(key, val);
    }
    println!("{}", serde_json::to_string(&data)?);
    Ok(())
}
