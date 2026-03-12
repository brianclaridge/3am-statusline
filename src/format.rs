/// Apply a named format specifier to a value.
pub fn apply(value: f64, specifier: &str) -> String {
    match specifier {
        "currency" => format_currency(value),
        "pct" => format_pct(value),
        "duration" => format_duration(value),
        "tokens" => format_tokens(value),
        "comma" => format_comma(value),
        _ => format!("{value}"),
    }
}

fn format_currency(value: f64) -> String {
    format!("${value:.2}")
}

fn format_pct(value: f64) -> String {
    format!("{}%", value.round() as i64)
}

fn format_duration(seconds: f64) -> String {
    let s = seconds as u64;
    if s >= 3600 {
        format!("{}h{}m", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{}m{}s", s / 60, s % 60)
    } else {
        format!("{s}s")
    }
}

fn format_tokens(value: f64) -> String {
    let v = value as u64;
    if v >= 1_000_000 {
        format!("{:.1}M", value / 1_000_000.0)
    } else if v >= 1_000 {
        format!("{:.1}K", value / 1_000.0)
    } else {
        format!("{v}")
    }
}

fn format_comma(value: f64) -> String {
    let v = value as i64;
    let s = v.to_string();
    let bytes = s.as_bytes();
    let negative = bytes[0] == b'-';
    let digits = if negative { &bytes[1..] } else { bytes };

    let mut result = String::with_capacity(s.len() + digits.len() / 3);
    if negative {
        result.push('-');
    }
    for (i, &b) in digits.iter().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(b as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn currency() {
        assert_eq!(apply(0.55, "currency"), "$0.55");
        assert_eq!(apply(12.1, "currency"), "$12.10");
        assert_eq!(apply(0.0, "currency"), "$0.00");
    }

    #[test]
    fn pct() {
        assert_eq!(apply(8.3, "pct"), "8%");
        assert_eq!(apply(99.5, "pct"), "100%");
        assert_eq!(apply(0.0, "pct"), "0%");
    }

    #[test]
    fn duration() {
        assert_eq!(apply(45.0, "duration"), "45s");
        assert_eq!(apply(90.0, "duration"), "1m30s");
        assert_eq!(apply(3661.0, "duration"), "1h1m");
    }

    #[test]
    fn tokens() {
        assert_eq!(apply(500.0, "tokens"), "500");
        assert_eq!(apply(15234.0, "tokens"), "15.2K");
        assert_eq!(apply(1_500_000.0, "tokens"), "1.5M");
    }

    #[test]
    fn comma() {
        assert_eq!(apply(15234.0, "comma"), "15,234");
        assert_eq!(apply(1000.0, "comma"), "1,000");
        assert_eq!(apply(999.0, "comma"), "999");
        assert_eq!(apply(1_000_000.0, "comma"), "1,000,000");
    }

    #[test]
    fn unknown_specifier_returns_raw() {
        assert_eq!(apply(42.0, "bogus"), "42");
    }
}
