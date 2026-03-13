use std::collections::HashMap;

use crate::format;
use crate::meter::{self, MeterConfig};

/// Resolve a template string by replacing `{field}`, `{field|format}`, `{meter:field}`,
/// `{c:name}`, and `{/c}` tokens.
pub fn resolve(
    template: &str,
    context: &HashMap<String, f64>,
    strings: &HashMap<String, String>,
    meter_config: &MeterConfig,
    use_color: bool,
    theme: &HashMap<String, String>,
) -> String {
    let mut result = String::with_capacity(template.len());
    let mut chars = template.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '{' {
            let mut token = String::new();
            let mut found_close = false;
            for inner in chars.by_ref() {
                if inner == '}' {
                    found_close = true;
                    break;
                }
                token.push(inner);
            }
            if found_close {
                result.push_str(&resolve_token(
                    &token,
                    context,
                    strings,
                    meter_config,
                    use_color,
                    theme,
                ));
            } else {
                result.push('{');
                result.push_str(&token);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Map a color name to an ANSI code. Checks theme first, then built-in names.
fn color_code(name: &str, theme: &HashMap<String, String>) -> Option<String> {
    if let Some(code) = theme.get(name) {
        return Some(code.clone());
    }
    let code = match name {
        "red" => "31",
        "green" => "32",
        "yellow" => "33",
        "blue" => "34",
        "magenta" => "35",
        "cyan" => "36",
        "white" => "37",
        "dim" => "2",
        "bold" => "1",
        "bright_red" => "91",
        "bright_green" => "92",
        "bright_yellow" => "93",
        "bright_blue" => "94",
        "bright_magenta" => "95",
        "bright_cyan" => "96",
        _ => return None,
    };
    Some(code.to_string())
}

fn resolve_token(
    token: &str,
    context: &HashMap<String, f64>,
    strings: &HashMap<String, String>,
    meter_config: &MeterConfig,
    use_color: bool,
    theme: &HashMap<String, String>,
) -> String {
    // {/c} → ANSI reset
    if token == "/c" {
        return if use_color {
            "\x1b[0m".into()
        } else {
            String::new()
        };
    }

    // {c:name} → ANSI color start
    if let Some(name) = token.strip_prefix("c:") {
        if !use_color {
            return String::new();
        }
        return color_code(name, theme)
            .map(|code| format!("\x1b[{code}m"))
            .unwrap_or_default();
    }

    // {meter:field.path}
    if let Some(field) = token.strip_prefix("meter:") {
        let pct = context.get(field).copied().unwrap_or(0.0);
        return meter::render(pct, meter_config, use_color);
    }

    // {field.path|format}, {field.path|color}, or {field.path|format|color}
    if let Some((field, spec)) = token.split_once('|') {
        let (format_spec, color_name) = match spec.rsplit_once('|') {
            // Two specs: {field|format|color}
            Some((fmt, clr)) if color_code(clr, theme).is_some() => (fmt, Some(clr)),
            _ => {
                // Single spec: color name or format specifier?
                if color_code(spec, theme).is_some() {
                    ("", Some(spec))
                } else {
                    (spec, None)
                }
            }
        };

        // Try numeric context
        if let Some(&value) = context.get(field) {
            let formatted = if format_spec.is_empty() {
                if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
                    (value as i64).to_string()
                } else {
                    value.to_string()
                }
            } else {
                format::apply(value, format_spec)
            };
            return wrap_color(&formatted, color_name, use_color, theme);
        }
        // Try string context
        if let Some(s) = strings.get(field) {
            return wrap_color(s, color_name, use_color, theme);
        }
        return String::new();
    }

    // {field.path} -- try string context first, then numeric
    if let Some(s) = strings.get(token) {
        return s.clone();
    }
    if let Some(&value) = context.get(token) {
        if value.fract() == 0.0 && value.abs() < i64::MAX as f64 {
            return (value as i64).to_string();
        }
        return value.to_string();
    }

    String::new()
}

fn wrap_color(
    text: &str,
    color_name: Option<&str>,
    use_color: bool,
    theme: &HashMap<String, String>,
) -> String {
    match (color_name, use_color) {
        (Some(name), true) => {
            if let Some(code) = color_code(name, theme) {
                format!("\x1b[{code}m{text}\x1b[0m")
            } else {
                text.to_string()
            }
        }
        _ => text.to_string(),
    }
}

/// Pad a left and right section to fill the given width.
pub fn pad_line(left: &str, right: &str, width: usize) -> String {
    let left_visible = strip_ansi_len(left);
    let right_visible = strip_ansi_len(right);
    let total = left_visible + right_visible;

    if total >= width {
        // Truncate right if it doesn't fit
        let available = width.saturating_sub(left_visible);
        let truncated_right = truncate_visible(right, available);
        format!("{left}{truncated_right}")
    } else {
        let gap = width - total;
        let spaces = " ".repeat(gap);
        format!("{left}{spaces}{right}")
    }
}

/// Count visible (non-ANSI-escape) characters in a string.
fn strip_ansi_len(s: &str) -> usize {
    let mut len = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
        } else {
            len += 1;
        }
    }
    len
}

/// Truncate a string to `max_visible` visible characters, preserving ANSI codes.
fn truncate_visible(s: &str, max_visible: usize) -> String {
    let mut result = String::new();
    let mut visible = 0;
    let mut in_escape = false;
    for ch in s.chars() {
        if in_escape {
            result.push(ch);
            if ch.is_ascii_alphabetic() {
                in_escape = false;
            }
        } else if ch == '\x1b' {
            in_escape = true;
            result.push(ch);
        } else if visible < max_visible {
            result.push(ch);
            visible += 1;
        } else {
            break;
        }
    }
    result
}

/// Detect terminal width from the COLUMNS env var or default to 80.
pub fn terminal_width() -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(80)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_meter() -> MeterConfig {
        MeterConfig::default()
    }

    fn ctx(pairs: &[(&str, f64)]) -> HashMap<String, f64> {
        pairs.iter().map(|(k, v)| (k.to_string(), *v)).collect()
    }

    fn strs(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    fn no_theme() -> HashMap<String, String> {
        HashMap::new()
    }

    #[test]
    fn plain_field_string() {
        let result = resolve(
            "{model.display_name}",
            &ctx(&[]),
            &strs(&[("model.display_name", "Opus")]),
            &empty_meter(),
            false,
            &no_theme(),
        );
        assert_eq!(result, "Opus");
    }

    #[test]
    fn plain_field_numeric() {
        let result = resolve(
            "{context_window.total_input_tokens}",
            &ctx(&[("context_window.total_input_tokens", 15234.0)]),
            &strs(&[]),
            &empty_meter(),
            false,
            &no_theme(),
        );
        assert_eq!(result, "15234");
    }

    #[test]
    fn format_specifier() {
        let result = resolve(
            "{cost.total_cost_usd|currency}",
            &ctx(&[("cost.total_cost_usd", 0.55)]),
            &strs(&[]),
            &empty_meter(),
            false,
            &no_theme(),
        );
        assert_eq!(result, "$0.55");
    }

    #[test]
    fn meter_token() {
        let result = resolve(
            "{meter:context_window.used_percentage}",
            &ctx(&[("context_window.used_percentage", 50.0)]),
            &strs(&[]),
            &empty_meter(),
            false,
            &no_theme(),
        );
        assert_eq!(result, "[●●●●●○○○○○]");
    }

    #[test]
    fn mixed_template() {
        let result = resolve(
            "{meter:context_window.used_percentage} {context_window.used_percentage|pct} ctx",
            &ctx(&[("context_window.used_percentage", 30.0)]),
            &strs(&[]),
            &empty_meter(),
            false,
            &no_theme(),
        );
        assert_eq!(result, "[●●●○○○○○○○] 30% ctx");
    }

    #[test]
    fn missing_field_resolves_empty() {
        let result = resolve(
            "hello {missing}!",
            &ctx(&[]),
            &strs(&[]),
            &empty_meter(),
            false,
            &no_theme(),
        );
        assert_eq!(result, "hello !");
    }

    #[test]
    fn unclosed_brace_preserved() {
        let result = resolve(
            "hello {world",
            &ctx(&[]),
            &strs(&[]),
            &empty_meter(),
            false,
            &no_theme(),
        );
        assert_eq!(result, "hello {world");
    }

    #[test]
    fn pad_line_basic() {
        let result = pad_line("left", "right", 20);
        assert_eq!(result, "left           right");
        assert_eq!(result.len(), 20);
    }

    #[test]
    fn pad_line_exact_fit() {
        let result = pad_line("left", "right", 9);
        assert_eq!(result, "leftright");
    }

    #[test]
    fn pad_line_overflow_truncates_right() {
        let result = pad_line("left", "right", 6);
        assert_eq!(result, "leftri");
    }

    #[test]
    fn strip_ansi_len_no_ansi() {
        assert_eq!(strip_ansi_len("hello"), 5);
    }

    #[test]
    fn strip_ansi_len_with_color() {
        assert_eq!(strip_ansi_len("\x1b[32mhi\x1b[0m"), 2);
    }
}
