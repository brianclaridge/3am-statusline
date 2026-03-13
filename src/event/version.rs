use anyhow::{Context, Result};
use serde::Serialize;

const RELEASES_URL: &str =
    "https://api.github.com/repos/anthropics/claude-code/releases/latest";

#[derive(Serialize)]
struct VersionInfo {
    latest: String,
}

pub fn run() -> Result<()> {
    let info = gather()?;
    let json = serde_json::to_string(&info).context("serializing version info")?;
    println!("{json}");
    Ok(())
}

fn gather() -> Result<VersionInfo> {
    let body: String = ureq::get(RELEASES_URL)
        .header("Accept", "application/vnd.github+json")
        .header("User-Agent", "3am-statusline")
        .call()
        .context("fetching GitHub latest release")?
        .body_mut()
        .read_to_string()
        .context("reading GitHub response")?;

    let value: serde_json::Value =
        serde_json::from_str(&body).context("parsing GitHub release JSON")?;

    let tag = value["tag_name"]
        .as_str()
        .context("missing 'tag_name' in GitHub release")?;

    // Strip leading 'v' if present (e.g. "v2.1.74" -> "2.1.74")
    let latest = tag.strip_prefix('v').unwrap_or(tag).to_string();

    Ok(VersionInfo { latest })
}

/// Compare two dotted version strings numerically.
/// Returns true if `current >= latest`.
pub fn is_current(current: &str, latest: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.split('.').filter_map(|s| s.parse().ok()).collect()
    };
    parse(current) >= parse(latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_version_is_current() {
        assert!(is_current("2.1.74", "2.1.74"));
    }

    #[test]
    fn newer_is_current() {
        assert!(is_current("2.1.75", "2.1.74"));
    }

    #[test]
    fn older_is_not_current() {
        assert!(!is_current("2.1.73", "2.1.74"));
    }

    #[test]
    fn major_version_difference() {
        assert!(!is_current("1.9.99", "2.0.0"));
        assert!(is_current("3.0.0", "2.9.99"));
    }
}
