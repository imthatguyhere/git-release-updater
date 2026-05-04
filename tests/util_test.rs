use git_release_updater::util;

//=-- ---------------------------------------------------------------------------
//=-- is_valid_url
//=-- ---------------------------------------------------------------------------

#[test]
fn test_valid_url_https() {
    assert!(util::is_valid_url("https://github.com/user/repo"));
}

#[test]
fn test_valid_url_http() {
    assert!(util::is_valid_url("http://example.com"));
}

#[test]
fn test_invalid_url_ftp() {
    assert!(!util::is_valid_url("ftp://example.com"));
}

#[test]
fn test_invalid_url_no_scheme() {
    assert!(!util::is_valid_url("not-a-url"));
}

#[test]
fn test_invalid_url_empty() {
    assert!(!util::is_valid_url(""));
}

//=-- ---------------------------------------------------------------------------
//=-- truncate
//=-- ---------------------------------------------------------------------------

#[test]
fn test_truncate_short_string() {
    assert_eq!(util::truncate("hello", 10), "hello");
}

#[test]
fn test_truncate_exact_length() {
    assert_eq!(util::truncate("hello", 5), "hello");
}

#[test]
fn test_truncate_long_string() {
    assert_eq!(util::truncate("hello world", 8), "hello...");
}

#[test]
fn test_truncate_minimal() {
    assert_eq!(util::truncate("hello", 3), "...");
}

#[test]
fn test_truncate_empty() {
    assert_eq!(util::truncate("", 5), "");
}

//=-- ---------------------------------------------------------------------------
//=-- current_timestamp
//=-- ---------------------------------------------------------------------------

#[test]
fn test_current_timestamp_positive() {
    let ts = util::current_timestamp();
    assert!(ts > 1_700_000_000, "timestamp should be reasonable: {ts}");
}

//=-- ---------------------------------------------------------------------------
//=-- format_timestamp
//=-- ---------------------------------------------------------------------------

#[test]
fn test_format_timestamp_epoch() {
    assert_eq!(util::format_timestamp(0), "1970-01-01 00:00:00");
}

#[test]
fn test_format_timestamp_known() {
    assert_eq!(util::format_timestamp(1705321800), "2024-01-15 12:30:00");
}
