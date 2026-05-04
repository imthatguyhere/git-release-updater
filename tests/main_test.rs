//=-- Integration: verify all modules compile and expose expected public API.
//=-- Network-dependent checks (run_check, download) are tested via release_test.rs.

#[test]
fn test_module_wiring() {
    let _ = git_release_updater::release::CheckMode::from_str("hash");
    let _ = git_release_updater::release::parse_repo_url("https://github.com/a/b");
    let _ = git_release_updater::release::clean_tag("v1.0");
    let _ = git_release_updater::util::current_timestamp();
    let _ = git_release_updater::util::is_valid_url("https://example.com");
}
