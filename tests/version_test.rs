use git_release_updater::version;

//=-- ---------------------------------------------------------------------------
//=-- clean_tag
//=-- ---------------------------------------------------------------------------

#[test]
fn test_clean_tag_lowercase_v() {
    assert_eq!(version::clean_tag("v1.2.3"), "1.2.3");
}

#[test]
fn test_clean_tag_uppercase_v() {
    assert_eq!(version::clean_tag("V1.2.3"), "1.2.3");
}

#[test]
fn test_clean_tag_no_prefix() {
    assert_eq!(version::clean_tag("1.2.3"), "1.2.3");
}

#[test]
fn test_clean_tag_empty() {
    assert_eq!(version::clean_tag(""), "");
}
