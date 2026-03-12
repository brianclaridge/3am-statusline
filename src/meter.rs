use std::io::IsTerminal;

pub struct MeterConfig {
    pub width: usize,
    pub filled: char,
    pub empty: char,
    pub threshold_yellow: f64,
    pub threshold_red: f64,
}

impl Default for MeterConfig {
    fn default() -> Self {
        Self {
            width: 10,
            filled: '●',
            empty: '○',
            threshold_yellow: 60.0,
            threshold_red: 85.0,
        }
    }
}

const ANSI_GREEN: &str = "\x1b[32m";
const ANSI_YELLOW: &str = "\x1b[33m";
const ANSI_RED: &str = "\x1b[31m";
const ANSI_RESET: &str = "\x1b[0m";

/// Render a metered usage bar like `[●●●○○○○○○○]` with optional ANSI color.
pub fn render(percentage: f64, config: &MeterConfig, use_color: bool) -> String {
    let pct = percentage.clamp(0.0, 100.0);
    let filled_count = ((pct / 100.0) * config.width as f64).floor() as usize;
    let filled_count = filled_count.min(config.width);
    let empty_count = config.width - filled_count;

    let filled: String = std::iter::repeat(config.filled).take(filled_count).collect();
    let empty: String = std::iter::repeat(config.empty).take(empty_count).collect();

    if use_color && filled_count > 0 {
        let color = if pct >= config.threshold_red {
            ANSI_RED
        } else if pct >= config.threshold_yellow {
            ANSI_YELLOW
        } else {
            ANSI_GREEN
        };
        format!("[{color}{filled}{ANSI_RESET}{empty}]")
    } else {
        format!("[{filled}{empty}]")
    }
}

/// Determine if ANSI color should be used.
/// Respects NO_COLOR/FORCE_COLOR conventions, falls back to TTY detection.
pub fn should_use_color() -> bool {
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if std::env::var_os("FORCE_COLOR").is_some() {
        return true;
    }
    std::io::stdout().is_terminal()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> MeterConfig {
        MeterConfig::default()
    }

    #[test]
    fn zero_percent() {
        let result = render(0.0, &cfg(), false);
        assert_eq!(result, "[○○○○○○○○○○]");
    }

    #[test]
    fn fifty_percent() {
        let result = render(50.0, &cfg(), false);
        assert_eq!(result, "[●●●●●○○○○○]");
    }

    #[test]
    fn hundred_percent() {
        let result = render(100.0, &cfg(), false);
        assert_eq!(result, "[●●●●●●●●●●]");
    }

    #[test]
    fn eight_percent() {
        // 8% with width=10: floor(0.08*10) = 0 filled
        let result = render(8.0, &cfg(), false);
        assert_eq!(result, "[○○○○○○○○○○]");
    }

    #[test]
    fn ten_percent() {
        let result = render(10.0, &cfg(), false);
        assert_eq!(result, "[●○○○○○○○○○]");
    }

    #[test]
    fn clamps_above_100() {
        let result = render(150.0, &cfg(), false);
        assert_eq!(result, "[●●●●●●●●●●]");
    }

    #[test]
    fn clamps_below_zero() {
        let result = render(-10.0, &cfg(), false);
        assert_eq!(result, "[○○○○○○○○○○]");
    }

    #[test]
    fn color_green() {
        let result = render(30.0, &cfg(), true);
        assert!(result.contains("\x1b[32m")); // green
        assert!(result.contains("\x1b[0m")); // reset
    }

    #[test]
    fn color_yellow() {
        let result = render(65.0, &cfg(), true);
        assert!(result.contains("\x1b[33m")); // yellow
    }

    #[test]
    fn color_red() {
        let result = render(90.0, &cfg(), true);
        assert!(result.contains("\x1b[31m")); // red
    }

    #[test]
    fn no_color_codes_when_disabled() {
        let result = render(90.0, &cfg(), false);
        assert!(!result.contains("\x1b["));
    }

    #[test]
    fn custom_chars() {
        let config = MeterConfig {
            width: 5,
            filled: '#',
            empty: '-',
            ..Default::default()
        };
        let result = render(60.0, &config, false);
        assert_eq!(result, "[###--]");
    }
}
