use chrono::TimeDelta;
use thiserror::Error;

#[derive(Debug, Error, PartialEq)]
#[error(
    "invalid duration '{input}': expected a number followed by 'd' (days), 'h' (hours), or 'm' (minutes), e.g. '2d', '1h', '30m'"
)]
pub struct ParseDurationError {
    input: String,
}

pub fn parse_duration(input: &str) -> Result<TimeDelta, ParseDurationError> {
    let err = || ParseDurationError {
        input: input.to_string(),
    };

    if input.len() < 2 {
        return Err(err());
    }

    let (digits, unit) = input.split_at(input.len() - 1);
    let amount: i64 = digits.parse().map_err(|_| err())?;

    if amount <= 0 {
        return Err(err());
    }

    match unit {
        "d" => TimeDelta::try_days(amount).ok_or_else(err),
        "h" => TimeDelta::try_hours(amount).ok_or_else(err),
        "m" => TimeDelta::try_minutes(amount).ok_or_else(err),
        _ => Err(err()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_days() {
        assert_eq!(parse_duration("2d").unwrap(), TimeDelta::days(2));
    }

    #[test]
    fn parse_hours() {
        assert_eq!(parse_duration("1h").unwrap(), TimeDelta::hours(1));
    }

    #[test]
    fn parse_minutes() {
        assert_eq!(parse_duration("30m").unwrap(), TimeDelta::minutes(30));
    }

    #[test]
    fn parse_large_values() {
        assert_eq!(parse_duration("365d").unwrap(), TimeDelta::days(365));
        assert_eq!(parse_duration("100h").unwrap(), TimeDelta::hours(100));
    }

    #[test]
    fn rejects_empty_string() {
        assert!(parse_duration("").is_err());
    }

    #[test]
    fn rejects_only_unit() {
        assert!(parse_duration("d").is_err());
    }

    #[test]
    fn rejects_only_number() {
        assert!(parse_duration("42").is_err());
    }

    #[test]
    fn rejects_unknown_unit() {
        assert!(parse_duration("5s").is_err());
        assert!(parse_duration("2w").is_err());
    }

    #[test]
    fn rejects_zero() {
        assert!(parse_duration("0d").is_err());
    }

    #[test]
    fn rejects_negative() {
        assert!(parse_duration("-1h").is_err());
    }

    #[test]
    fn rejects_non_numeric() {
        assert!(parse_duration("abch").is_err());
    }

    #[test]
    fn error_message_includes_input() {
        let err = parse_duration("bad").unwrap_err();
        assert!(err.to_string().contains("bad"));
    }
}
