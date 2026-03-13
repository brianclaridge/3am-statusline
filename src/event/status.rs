use anyhow::{Context, Result};

const STATUS_URL: &str = "https://status.claude.com/api/v2/status.json";

fn indicator_to_text(indicator: &str) -> &'static str {
    match indicator {
        "none" => "\u{1f7e2} ok",
        "minor" => "\u{1f7e1} slow",
        "major" | "critical" => "\u{1f534} down",
        _ => "\u{26aa} ???",
    }
}

pub fn run() -> Result<()> {
    let body: String = ureq::get(STATUS_URL)
        .header("Accept", "application/json")
        .call()
        .context("requesting status.claude.com")?
        .body_mut()
        .read_to_string()
        .context("reading status response")?;

    let value: serde_json::Value =
        serde_json::from_str(&body).context("parsing status JSON")?;

    let indicator = value
        .pointer("/status/indicator")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown");

    println!("{}", indicator_to_text(indicator));
    Ok(())
}
