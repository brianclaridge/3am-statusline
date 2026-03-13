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

fn time_emoji(hour: u32) -> &'static str {
    match hour {
        6..=17 => "\u{2600}\u{fe0f}",  // ☀️
        18..=20 => "\u{1f305}",         // 🌅
        _ => "\u{1f319}",               // 🌙
    }
}

fn format_time(tz: Tz) -> (String, String) {
    let now = Utc::now().with_timezone(&tz);
    let h = {
        let h12 = now.format("%I").to_string().trim_start_matches('0').to_string();
        if h12.is_empty() { "12".to_string() } else { h12 }
    };
    let m = now.format("%M").to_string();
    let emoji = time_emoji(now.format("%H").to_string().parse::<u32>().unwrap_or(0));
    (format!("{h}:{m}"), emoji.to_string())
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
        let (time, icon) = resolve_tz(zone)
            .map(format_time)
            .unwrap_or_else(|| ("???".to_string(), "".to_string()));
        data.insert(key.clone(), time);
        data.insert(format!("{key}_icon"), icon);
    }
    println!("{}", serde_json::to_string(&data)?);
    Ok(())
}
