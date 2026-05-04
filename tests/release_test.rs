use git_release_updater::release;

//=-- ---------------------------------------------------------------------------
//=-- CheckMode
//=-- ---------------------------------------------------------------------------

#[test]
fn test_check_mode_from_str_hash() {
    assert_eq!(release::CheckMode::from_str("hash"), Ok(release::CheckMode::Hash));
    assert_eq!(release::CheckMode::from_str("HASH"), Ok(release::CheckMode::Hash));
    assert_eq!(release::CheckMode::from_str("Hash"), Ok(release::CheckMode::Hash));
}

#[test]
fn test_check_mode_from_str_version() {
    assert_eq!(release::CheckMode::from_str("version"), Ok(release::CheckMode::Version));
}

#[test]
fn test_check_mode_from_str_both() {
    assert_eq!(release::CheckMode::from_str("both"), Ok(release::CheckMode::Both));
}

#[test]
fn test_check_mode_from_str_invalid() {
    assert!(release::CheckMode::from_str("invalid").is_err());
    assert!(release::CheckMode::from_str("").is_err());
}

#[test]
fn test_check_mode_wants_hash() {
    assert!(release::CheckMode::Hash.wants_hash());
    assert!(!release::CheckMode::Version.wants_hash());
    assert!(release::CheckMode::Both.wants_hash());
}

#[test]
fn test_check_mode_wants_version() {
    assert!(!release::CheckMode::Hash.wants_version());
    assert!(release::CheckMode::Version.wants_version());
    assert!(release::CheckMode::Both.wants_version());
}

//=-- ---------------------------------------------------------------------------
//=-- parse_repo_url
//=-- ---------------------------------------------------------------------------

#[test]
fn test_parse_repo_url_https() {
    let (owner, repo) = release::parse_repo_url("https://github.com/microsoft/winget-create").unwrap();
    assert_eq!(owner, "microsoft");
    assert_eq!(repo, "winget-create");
}

#[test]
fn test_parse_repo_url_with_trailing_slash() {
    let (owner, repo) = release::parse_repo_url("https://github.com/microsoft/winget-create/").unwrap();
    assert_eq!(owner, "microsoft");
    assert_eq!(repo, "winget-create");
}

#[test]
fn test_parse_repo_url_with_dot_git() {
    let (owner, repo) = release::parse_repo_url("https://github.com/microsoft/winget-create.git").unwrap();
    assert_eq!(owner, "microsoft");
    assert_eq!(repo, "winget-create");
}

#[test]
fn test_parse_repo_url_http() {
    let (owner, repo) = release::parse_repo_url("http://github.com/user/repo").unwrap();
    assert_eq!(owner, "user");
    assert_eq!(repo, "repo");
}

#[test]
fn test_parse_repo_url_invalid_scheme() {
    assert!(release::parse_repo_url("https://gitlab.com/user/repo").is_err());
    assert!(release::parse_repo_url("not-a-url").is_err());
}

#[test]
fn test_parse_repo_url_too_many_parts() {
    assert!(release::parse_repo_url("https://github.com/user/repo/extra").is_err());
}

#[test]
fn test_parse_repo_url_too_few_parts() {
    assert!(release::parse_repo_url("https://github.com/user").is_err());
}

//=-- ---------------------------------------------------------------------------
//=-- clean_tag
//=-- ---------------------------------------------------------------------------

#[test]
fn test_clean_tag_lowercase_v() {
    assert_eq!(release::clean_tag("v1.2.3"), "1.2.3");
}

#[test]
fn test_clean_tag_uppercase_v() {
    assert_eq!(release::clean_tag("V1.2.3"), "1.2.3");
}

#[test]
fn test_clean_tag_no_prefix() {
    assert_eq!(release::clean_tag("1.2.3"), "1.2.3");
}

#[test]
fn test_clean_tag_empty() {
    assert_eq!(release::clean_tag(""), "");
}

//=-- ---------------------------------------------------------------------------
//=-- resolve_exe_path
//=-- ---------------------------------------------------------------------------

#[test]
fn test_resolve_exe_path_full_path() {
    let p = release::resolve_exe_path(&std::path::Path::new("C:\\tools\\myapp.exe"), "other.exe");
    assert_eq!(p, std::path::Path::new("C:\\tools\\myapp.exe"));
}

#[test]
fn test_resolve_exe_path_dir_trailing_slash() {
    let p = release::resolve_exe_path(&std::path::Path::new("C:\\tools\\"), "myapp.exe");
    assert_eq!(p, std::path::Path::new("C:\\tools\\myapp.exe"));
}

#[test]
fn test_resolve_exe_path_dir_forward_slash() {
    let p = release::resolve_exe_path(&std::path::Path::new("C:/tools/"), "myapp.exe");
    assert_eq!(p, std::path::Path::new("C:/tools/myapp.exe"));
}

#[test]
fn test_resolve_exe_path_relative_dir() {
    let p = release::resolve_exe_path(&std::path::Path::new("target\\debug\\"), "wingetcreate.exe");
    assert_eq!(p, std::path::Path::new("target\\debug\\wingetcreate.exe"));
}

//=-- ---------------------------------------------------------------------------
//=-- find_asset_url
//=-- ---------------------------------------------------------------------------

#[test]
fn test_find_asset_url_found() {
    let release = git_release_updater::release::GitHubRelease {
        tag_name: "v1.0".into(),
        assets: vec![
            git_release_updater::release::GitHubAsset {
                name: "target.exe".into(),
                browser_download_url: "https://example.com/target.exe".into(),
            },
        ],
    };
    assert_eq!(
        release::find_asset_url(&release, "target.exe"),
        Some("https://example.com/target.exe")
    );
}

#[test]
fn test_find_asset_url_case_insensitive() {
    let release = git_release_updater::release::GitHubRelease {
        tag_name: "v1.0".into(),
        assets: vec![
            git_release_updater::release::GitHubAsset {
                name: "MyApp.EXE".into(),
                browser_download_url: "https://example.com/MyApp.EXE".into(),
            },
        ],
    };
    assert_eq!(
        release::find_asset_url(&release, "myapp.exe"),
        Some("https://example.com/MyApp.EXE")
    );
}

#[test]
fn test_find_asset_url_not_found() {
    let release = git_release_updater::release::GitHubRelease {
        tag_name: "v1.0".into(),
        assets: vec![],
    };
    assert_eq!(release::find_asset_url(&release, "missing.exe"), None);
}

//=-- ---------------------------------------------------------------------------
//=-- hash_local_file
//=-- ---------------------------------------------------------------------------

#[test]
fn test_hash_local_file_nonexistent() {
    let result = release::hash_local_file(std::path::Path::new("C:\\does_not_exist_12345.exe"));
    assert!(result.is_err());
}

#[test]
fn test_hash_local_file_empty() {
    let dir = std::env::temp_dir();
    let path = dir.join("gru_test_empty.bin");
    std::fs::write(&path, []).unwrap();
    let result = release::hash_local_file(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(result.unwrap(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
}

#[test]
fn test_hash_local_file_known() {
    let dir = std::env::temp_dir();
    let path = dir.join("gru_test_known.bin");
    std::fs::write(&path, b"hello world").unwrap();
    let result = release::hash_local_file(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        result.unwrap(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}
