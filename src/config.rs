use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

use crate::meter::MeterConfig;

#[derive(Debug, Deserialize)]
pub struct StatuslineConfig {
    #[serde(default)]
    pub lines: Vec<LineConfig>,
    #[serde(default)]
    pub meter: MeterYaml,
    #[serde(default)]
    pub theme: std::collections::HashMap<String, String>,
    pub budget: Option<BudgetConfig>,
    pub state: Option<StateConfig>,
    pub logging: Option<LoggingConfig>,
    #[serde(default)]
    pub events: Vec<EventConfig>,
}

#[derive(Debug, Deserialize)]
pub struct LineConfig {
    pub left: String,
    #[serde(default)]
    pub right: String,
}

#[derive(Debug, Deserialize)]
pub struct MeterYaml {
    #[serde(default = "default_width")]
    pub width: usize,
    #[serde(default = "default_filled")]
    pub filled: String,
    #[serde(default = "default_empty")]
    pub empty: String,
    #[serde(default)]
    pub thresholds: ThresholdConfig,
}

impl Default for MeterYaml {
    fn default() -> Self {
        Self {
            width: default_width(),
            filled: default_filled(),
            empty: default_empty(),
            thresholds: ThresholdConfig::default(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct ThresholdConfig {
    #[serde(default)]
    pub green: f64,
    #[serde(default = "default_yellow")]
    pub yellow: f64,
    #[serde(default = "default_red")]
    pub red: f64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            green: 0.0,
            yellow: default_yellow(),
            red: default_red(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct BudgetConfig {
    #[serde(default)]
    pub weekly_tokens: u64,
    #[serde(default)]
    pub monthly_tokens: u64,
}

#[derive(Debug, Deserialize)]
pub struct StateConfig {
    #[serde(default = "default_state_dir")]
    pub dir: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_log_file")]
    pub file: String,
    pub json: Option<JsonLogConfig>,
}

#[derive(Debug, Deserialize)]
pub struct JsonLogConfig {
    #[serde(default = "default_json_log_dir")]
    pub dir: String,
    #[serde(default = "default_keep")]
    pub keep: usize,
}

#[derive(Debug, Deserialize)]
pub struct EventConfig {
    pub name: String,
    pub command: String,
    pub interval: String,
    #[serde(default)]
    pub capture: bool,
}

fn default_state_dir() -> String { ".data/statusline".into() }
fn default_log_file() -> String { ".data/statusline/statusline.log".into() }
fn default_json_log_dir() -> String { ".data/logs/statusline/json".into() }
fn default_keep() -> usize { 50 }
fn default_width() -> usize { 10 }
fn default_filled() -> String { "●".into() }
fn default_empty() -> String { "○".into() }
fn default_yellow() -> f64 { 60.0 }
fn default_red() -> f64 { 85.0 }

impl StatuslineConfig {
    pub fn state_dir(&self) -> &str {
        self.state.as_ref().map(|s| s.dir.as_str()).unwrap_or(".data/statusline")
    }

    pub fn to_meter_config(&self) -> MeterConfig {
        MeterConfig {
            width: self.meter.width,
            filled: self.meter.filled.chars().next().unwrap_or('●'),
            empty: self.meter.empty.chars().next().unwrap_or('○'),
            threshold_yellow: self.meter.thresholds.yellow,
            threshold_red: self.meter.thresholds.red,
        }
    }

    pub fn line_templates(&self) -> Vec<(String, String)> {
        self.lines
            .iter()
            .map(|l| (l.left.clone(), l.right.clone()))
            .collect()
    }
}

/// Load config from the first file found in the search order.
/// Returns None if no config file exists (use built-in defaults).
pub fn load() -> Result<Option<StatuslineConfig>> {
    if let Some(path) = find_config_path() {
        let content = std::fs::read_to_string(&path)?;
        let config: StatuslineConfig = serde_yml::from_str(&content)?;
        Ok(Some(config))
    } else {
        Ok(None)
    }
}

/// Search order: $STATUSLINE_CONFIG > config/statusline.yml > .claude/statusline.yml > $CLAUDE_CONFIG_DIR/statusline.yml
fn find_config_path() -> Option<PathBuf> {
    let candidates = config_candidates();
    candidates.into_iter().find(|p| p.exists())
}

fn config_candidates() -> Vec<PathBuf> {
    let mut paths = vec![];

    if let Ok(explicit) = std::env::var("STATUSLINE_CONFIG") {
        paths.push(PathBuf::from(explicit));
    }

    paths.push(PathBuf::from("config/statusline.yml"));
    paths.push(PathBuf::from(".claude/statusline.yml"));

    if let Ok(config_dir) = std::env::var("CLAUDE_CONFIG_DIR") {
        paths.push(Path::new(&config_dir).join("statusline.yml"));
    }

    paths
}

/// Build the default two-line layout when no config file is present.
pub fn default_lines() -> Vec<(String, String)> {
    vec![
        (
            "{model.display_name}".into(),
            "{cost.total_cost_usd|currency}".into(),
        ),
        (
            "{meter:context_window.used_percentage} {context_window.used_percentage|pct} ctx".into(),
            "{context_window.total_input_tokens|tokens} tok".into(),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn deserialize_full_config() {
        let yaml = r#"
lines:
  - left: "{model.display_name}"
    right: "{cost.total_cost_usd|currency}"
  - left: "{meter:context_window.used_percentage} {context_window.used_percentage|pct} ctx"
    right: "{context_window.total_input_tokens|tokens} tok"

meter:
  width: 10
  filled: "●"
  empty: "○"
  thresholds:
    green: 0
    yellow: 60
    red: 85

budget:
  weekly_tokens: 5000000
  monthly_tokens: 20000000
"#;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.lines.len(), 2);
        assert_eq!(config.meter.width, 10);
        assert_eq!(config.meter.thresholds.yellow, 60.0);
        let budget = config.budget.unwrap();
        assert_eq!(budget.weekly_tokens, 5_000_000);
        assert_eq!(budget.monthly_tokens, 20_000_000);
    }

    #[test]
    fn deserialize_minimal_config() {
        let yaml = r#"
lines:
  - left: "{model.display_name}"
"#;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.lines.len(), 1);
        assert_eq!(config.lines[0].right, "");
        assert_eq!(config.meter.width, 10); // default
        assert!(config.budget.is_none());
    }

    #[test]
    fn deserialize_empty_config() {
        let yaml = "{}";
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        assert!(config.lines.is_empty());
        assert!(config.budget.is_none());
    }

    #[test]
    fn to_meter_config() {
        let yaml = r##"
meter:
  width: 5
  filled: "#"
  empty: "-"
  thresholds:
    yellow: 50
    red: 75
"##;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        let mc = config.to_meter_config();
        assert_eq!(mc.width, 5);
        assert_eq!(mc.filled, '#');
        assert_eq!(mc.empty, '-');
        assert_eq!(mc.threshold_yellow, 50.0);
        assert_eq!(mc.threshold_red, 75.0);
    }

    #[test]
    fn custom_lines() {
        let yaml = r#"
lines:
  - left: "L1"
    right: "R1"
  - left: "L2"
    right: "R2"
  - left: "L3"
    right: "R3"
"#;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        let templates = config.line_templates();
        assert_eq!(templates.len(), 3);
        assert_eq!(templates[0], ("L1".into(), "R1".into()));
        assert_eq!(templates[2], ("L3".into(), "R3".into()));
    }

    #[test]
    fn default_lines_has_two() {
        let lines = default_lines();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].0.contains("model.display_name"));
        assert!(lines[1].0.contains("meter:"));
    }

    #[test]
    fn config_candidates_statusline_config_env() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::set_var("STATUSLINE_CONFIG", "/tmp/custom.yml");
        let paths = config_candidates();
        assert_eq!(paths[0], PathBuf::from("/tmp/custom.yml"));
        std::env::remove_var("STATUSLINE_CONFIG");
    }

    #[test]
    fn config_candidates_includes_claude_dir() {
        let _lock = ENV_LOCK.lock().unwrap();
        std::env::remove_var("STATUSLINE_CONFIG");
        let paths = config_candidates();
        assert_eq!(paths[0], PathBuf::from("config/statusline.yml"));
        assert_eq!(paths[1], PathBuf::from(".claude/statusline.yml"));
    }

    #[test]
    fn deserialize_events_config() {
        let yaml = r#"
events:
  - name: branch
    command: "git rev-parse --abbrev-ref HEAD 2>/dev/null"
    interval: 10s
    capture: true
  - name: webhook
    command: "curl -s https://hooks.example.com/ping"
    interval: 1h
    capture: false
"#;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.events.len(), 2);
        assert_eq!(config.events[0].name, "branch");
        assert!(config.events[0].capture);
        assert_eq!(config.events[1].name, "webhook");
        assert!(!config.events[1].capture);
    }

    #[test]
    fn events_default_empty() {
        let yaml = "{}";
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        assert!(config.events.is_empty());
    }
}
