#![allow(dead_code)]

use std::time::{SystemTime, UNIX_EPOCH};

/// Returns the current timestamp as a Unix timestamp (seconds since epoch).
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Formats a Unix timestamp into a human-readable date-time string.
/// Uses a simple ISO-ish format: YYYY-MM-DD HH:MM:SS
pub fn format_timestamp(secs: u64) -> String {
    let total_days = secs / 86400;
    let remaining = secs % 86400;
    let hours = remaining / 3600;
    let minutes = (remaining % 3600) / 60;
    let seconds = remaining % 60;

    //=-- Days since epoch → year/month/day
    let (y, m, d) = days_to_date(total_days as i64);

    format!("{y:04}-{m:02}-{d:02} {hours:02}:{minutes:02}:{seconds:02}")
}

/// Truncates a string to the specified maximum length, appending "..." if truncated.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len <= 3 {
        "...".to_string()
    } else {
        let cutoff = max_len - 3;
        let mut end = cutoff;
        while end > 0 && !s.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}...", &s[..end])
    }
}

/// Checks if a string is a valid URL by verifying it starts with http:// or https://.
pub fn is_valid_url(s: &str) -> bool {
    s.starts_with("http://") || s.starts_with("https://")
}

/// Converts days since Unix epoch (Jan 1 1970) to (year, month, day).
fn days_to_date(mut days: i64) -> (i64, i64, i64) {
    days += 719468; //=-- shift epoch to year 0
    let era = if days >= 0 { days } else { days - 146096 } / 146097;
    let doe = days - era * 146097; //=-- day of era [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; //=-- year of era [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); //=-- day of year [0, 365]
    let mp = (5 * doy + 2) / 153; //=-- month phase [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; //=-- day [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; //=-- month [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

//=-- ---------------------------------------------------------------------------
//=-- Inline tests (private fn coverage only; public API tested via tests/)
//=-- ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_days_to_date_epoch() {
        assert_eq!(days_to_date(0), (1970, 1, 1));
    }

    #[test]
    fn test_days_to_date_2024() {
        let days = (2024 - 1970) * 365 + 13 + 166;
        assert_eq!(days_to_date(days), (2024, 6, 15));
    }
}
