use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::Deserialize;

use crate::meter::MeterConfig;
use crate::template::hex_to_ansi;

#[derive(Debug, Deserialize)]
pub struct StatuslineConfig {
    #[serde(default)]
    pub lines: Vec<LineConfig>,
    #[serde(default)]
    pub meter: MeterYaml,
    #[serde(default)]
    pub theme: HashMap<String, String>,
    pub state: Option<StateConfig>,
    pub logging: Option<LoggingConfig>,
    #[serde(default)]
    pub events: Vec<EventConfig>,
    #[serde(default)]
    pub color_fields: Vec<ColorFieldConfig>,
    #[serde(default)]
    pub columns: Option<usize>,
    #[serde(default)]
    pub colors: HashMap<String, String>,
    #[serde(default)]
    pub themes: HashMap<String, HashMap<String, String>>,
    #[serde(default)]
    pub current_theme: Option<String>,
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

#[derive(Debug, Deserialize)]
pub struct ColorFieldConfig {
    pub name: String,
    pub source: String,
    #[serde(default)]
    pub format: Option<String>,
    #[serde(default = "default_cf_yellow")]
    pub yellow: f64,
    #[serde(default = "default_cf_red")]
    pub red: f64,
}

fn default_cf_yellow() -> f64 { 40.0 }
fn default_cf_red() -> f64 { 60.0 }

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

    /// Resolve the active theme. If `themes` is non-empty, select by `current_theme`
    /// (defaulting to "default") and resolve color aliases. Otherwise fall back to
    /// the legacy flat `theme` map.
    pub fn resolve_theme(&self) -> HashMap<String, String> {
        if self.themes.is_empty() {
            return self.theme.clone();
        }
        let key = self.current_theme.as_deref().unwrap_or("default");
        let raw = match self.themes.get(key) {
            Some(t) => t,
            None => return self.theme.clone(),
        };
        raw.iter()
            .map(|(k, v)| (k.clone(), self.resolve_color_value(v)))
            .collect()
    }

    /// Resolve a color value: named alias → hex → raw passthrough.
    fn resolve_color_value(&self, value: &str) -> String {
        if let Some(resolved) = self.colors.get(value) {
            if let Some(code) = hex_to_ansi(resolved) {
                return code;
            }
            return resolved.clone();
        }
        if let Some(code) = hex_to_ansi(value) {
            return code;
        }
        value.to_string()
    }

    pub fn to_meter_config(&self, theme: &HashMap<String, String>) -> MeterConfig {
        let color_green = theme.get("meter_green")
            .map(|c| format!("\x1b[{c}m"))
            .unwrap_or_else(|| "\x1b[32m".into());
        let color_yellow = theme.get("meter_yellow")
            .map(|c| format!("\x1b[{c}m"))
            .unwrap_or_else(|| "\x1b[33m".into());
        let color_red = theme.get("meter_red")
            .map(|c| format!("\x1b[{c}m"))
            .unwrap_or_else(|| "\x1b[31m".into());

        MeterConfig {
            width: self.meter.width,
            filled: self.meter.filled.chars().next().unwrap_or('●'),
            empty: self.meter.empty.chars().next().unwrap_or('○'),
            threshold_yellow: self.meter.thresholds.yellow,
            threshold_red: self.meter.thresholds.red,
            color_green,
            color_yellow,
            color_red,
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
"#;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        assert_eq!(config.lines.len(), 2);
        assert_eq!(config.meter.width, 10);
        assert_eq!(config.meter.thresholds.yellow, 60.0);
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
    }

    #[test]
    fn deserialize_empty_config() {
        let yaml = "{}";
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        assert!(config.lines.is_empty());
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
        let theme = config.resolve_theme();
        let mc = config.to_meter_config(&theme);
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
        temp_env::with_var("STATUSLINE_CONFIG", Some("/tmp/custom.yml"), || {
            let paths = config_candidates();
            assert_eq!(paths[0], PathBuf::from("/tmp/custom.yml"));
        });
    }

    #[test]
    fn config_candidates_includes_claude_dir() {
        temp_env::with_var_unset("STATUSLINE_CONFIG", || {
            let paths = config_candidates();
            assert_eq!(paths[0], PathBuf::from("config/statusline.yml"));
            assert_eq!(paths[1], PathBuf::from(".claude/statusline.yml"));
        });
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

    #[test]
    fn resolve_theme_legacy_fallback() {
        let yaml = r#"
theme:
  model: "1;36"
  sep: "2"
"#;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        let theme = config.resolve_theme();
        assert_eq!(theme.get("model").unwrap(), "1;36");
        assert_eq!(theme.get("sep").unwrap(), "2");
    }

    #[test]
    fn resolve_theme_named_theme() {
        let yaml = r##"
colors:
  neon_pink: "#FF1493"

current_theme: cyber

themes:
  cyber:
    sep: neon_pink
    model: "1"
"##;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        let theme = config.resolve_theme();
        // neon_pink alias resolved to truecolor ANSI
        assert_eq!(theme.get("sep").unwrap(), "38;2;255;20;147");
        // Raw ANSI passed through
        assert_eq!(theme.get("model").unwrap(), "1");
    }

    #[test]
    fn resolve_theme_default_selection() {
        let yaml = r#"
themes:
  default:
    model: "1;36"
  other:
    model: "31"
"#;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        let theme = config.resolve_theme();
        assert_eq!(theme.get("model").unwrap(), "1;36");
    }

    #[test]
    fn resolve_theme_hex_direct() {
        let yaml = r##"
themes:
  default:
    model: "#00FF00"
"##;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        let theme = config.resolve_theme();
        assert_eq!(theme.get("model").unwrap(), "38;2;0;255;0");
    }

    #[test]
    fn meter_config_with_theme_colors() {
        let yaml = r##"
colors:
  neon_green: "#39FF14"

themes:
  default:
    meter_green: neon_green
    meter_red: "#FF0000"
"##;
        let config: StatuslineConfig = serde_yml::from_str(yaml).unwrap();
        let theme = config.resolve_theme();
        let mc = config.to_meter_config(&theme);
        assert_eq!(mc.color_green, "\x1b[38;2;57;255;20m");
        assert_eq!(mc.color_red, "\x1b[38;2;255;0;0m");
        // yellow falls back to default
        assert_eq!(mc.color_yellow, "\x1b[33m");
    }
}
