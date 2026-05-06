use git_release_updater::release;

//=-- ---------------------------------------------------------------------------
//=-- CheckMode
//=-- ---------------------------------------------------------------------------

#[test]
fn test_check_mode_from_str_hash() {
    assert_eq!(
        release::CheckMode::from_str("hash"),
        Ok(release::CheckMode::Hash)
    );
    assert_eq!(
        release::CheckMode::from_str("HASH"),
        Ok(release::CheckMode::Hash)
    );
    assert_eq!(
        release::CheckMode::from_str("Hash"),
        Ok(release::CheckMode::Hash)
    );
}

#[test]
fn test_check_mode_from_str_version() {
    assert_eq!(
        release::CheckMode::from_str("version"),
        Ok(release::CheckMode::Version)
    );
}

#[test]
fn test_check_mode_from_str_both() {
    assert_eq!(
        release::CheckMode::from_str("both"),
        Ok(release::CheckMode::Both)
    );
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
    let (owner, repo) =
        release::parse_repo_url("https://github.com/microsoft/winget-create").unwrap();
    assert_eq!(owner, "microsoft");
    assert_eq!(repo, "winget-create");
}

#[test]
fn test_parse_repo_url_with_trailing_slash() {
    let (owner, repo) =
        release::parse_repo_url("https://github.com/microsoft/winget-create/").unwrap();
    assert_eq!(owner, "microsoft");
    assert_eq!(repo, "winget-create");
}

#[test]
fn test_parse_repo_url_with_dot_git() {
    let (owner, repo) =
        release::parse_repo_url("https://github.com/microsoft/winget-create.git").unwrap();
    assert_eq!(owner, "microsoft");
    assert_eq!(repo, "winget-create");
}

#[test]
fn test_parse_repo_url_http() {
    let (owner, repo) =
        release::parse_repo_url("http://github.com/microsoft/winget-create").unwrap();
    assert_eq!(owner, "microsoft");
    assert_eq!(repo, "winget-create");
}

#[test]
fn test_parse_repo_url_invalid_scheme() {
    assert!(release::parse_repo_url("https://gitlab.com/user/repo").is_err());
    assert!(release::parse_repo_url("not-a-url").is_err());
}

#[test]
fn test_parse_repo_url_too_many_parts() {
    assert!(release::parse_repo_url("https://github.com/microsoft/winget-create/extra").is_err());
}

#[test]
fn test_parse_repo_url_too_few_parts() {
    assert!(release::parse_repo_url("https://github.com/microsoft").is_err());
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
fn test_resolve_exe_path_dir_without_trailing_separator() {
    let p = release::resolve_exe_path(&std::path::Path::new("C:\\tools"), "myapp.exe");
    assert_eq!(p, std::path::Path::new("C:\\tools\\myapp.exe"));
}

#[test]
fn test_resolve_exe_path_nested_dir_without_trailing_separator() {
    let p = release::resolve_exe_path(
        &std::path::Path::new("C:\\kworking\\cdk-drive-updater"),
        "cdk-drive-updater.exe",
    );
    assert_eq!(
        p,
        std::path::Path::new("C:\\kworking\\cdk-drive-updater\\cdk-drive-updater.exe")
    );
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
        assets: vec![git_release_updater::release::GitHubAsset {
            name: "target.exe".into(),
            browser_download_url: "https://example.com/target.exe".into(),
            digest: None,
        }],
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
        assets: vec![git_release_updater::release::GitHubAsset {
            name: "MyApp.EXE".into(),
            browser_download_url: "https://example.com/MyApp.EXE".into(),
            digest: None,
        }],
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
    assert_eq!(
        result.unwrap(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
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

//=-- ---------------------------------------------------------------------------
//=-- sha256_bytes
//=-- ---------------------------------------------------------------------------

#[test]
fn test_sha256_bytes_known() {
    let hash = release::sha256_bytes(b"hello world");
    assert_eq!(
        hash,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_sha256_bytes_empty() {
    let hash = release::sha256_bytes(b"");
    assert_eq!(
        hash,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

//=-- ---------------------------------------------------------------------------
//=-- download_bytes / download_and_hash are network-dependent — tested via
//=-- integration tests with GitHub API in functional testing.
//=-- ---------------------------------------------------------------------------
//=-- API URL construction (no network)
//=-- ---------------------------------------------------------------------------

fn assert_api_url(repo_url: &str, tag: &str, expected: &str) {
    let (owner, repo) = release::parse_repo_url(repo_url).unwrap();
    let actual = if tag == "latest" {
        format!("https://api.github.com/repos/{owner}/{repo}/releases/latest")
    } else {
        format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}")
    };
    assert_eq!(
        actual, expected,
        "API URL mismatch for repo={repo_url} tag={tag}"
    );
}

#[test]
fn test_api_url_latest_default() {
    assert_api_url(
        "https://github.com/microsoft/winget-create",
        "latest",
        "https://api.github.com/repos/microsoft/winget-create/releases/latest",
    );
}

#[test]
fn test_api_url_latest_trailing_slash() {
    assert_api_url(
        "https://github.com/microsoft/winget-create/",
        "latest",
        "https://api.github.com/repos/microsoft/winget-create/releases/latest",
    );
}

#[test]
fn test_api_url_latest_dot_git() {
    assert_api_url(
        "https://github.com/microsoft/winget-create.git",
        "latest",
        "https://api.github.com/repos/microsoft/winget-create/releases/latest",
    );
}

#[test]
fn test_api_url_by_tag() {
    assert_api_url(
        "https://github.com/microsoft/winget-create",
        "v1.2.3",
        "https://api.github.com/repos/microsoft/winget-create/releases/tags/v1.2.3",
    );
}

#[test]
fn test_api_url_by_tag_with_v() {
    assert_api_url(
        "https://github.com/microsoft/winget-create",
        "v1.0.0-beta",
        "https://api.github.com/repos/microsoft/winget-create/releases/tags/v1.0.0-beta",
    )
}

//=-- ---------------------------------------------------------------------------
//=-- find_asset + digest
//=-- ---------------------------------------------------------------------------

#[test]
fn test_find_asset_returns_full_asset() {
    let release = git_release_updater::release::GitHubRelease {
        tag_name: "v1.0".into(),
        assets: vec![git_release_updater::release::GitHubAsset {
            name: "tool.exe".into(),
            browser_download_url: "https://example.com/tool.exe".into(),
            digest: Some("sha256:abc123".into()),
        }],
    };
    let asset = release::find_asset(&release, "tool.exe").unwrap();
    assert_eq!(asset.name, "tool.exe");
    assert_eq!(asset.browser_download_url, "https://example.com/tool.exe");
    assert_eq!(asset.digest, Some("sha256:abc123".into()));
}

#[test]
fn test_find_asset_digest_none() {
    let release = git_release_updater::release::GitHubRelease {
        tag_name: "v1.0".into(),
        assets: vec![git_release_updater::release::GitHubAsset {
            name: "tool.exe".into(),
            browser_download_url: "https://example.com/tool.exe".into(),
            digest: None,
        }],
    };
    let asset = release::find_asset(&release, "tool.exe").unwrap();
    assert!(asset.digest.is_none());
}

#[test]
fn test_find_asset_not_found() {
    let release = git_release_updater::release::GitHubRelease {
        tag_name: "v1.0".into(),
        assets: vec![],
    };
    assert!(release::find_asset(&release, "missing.exe").is_none());
}

//=-- ---------------------------------------------------------------------------
//=-- save_bytes
//=-- ---------------------------------------------------------------------------

#[test]
fn test_save_bytes_roundtrip() {
    let dir = std::env::temp_dir().join("gru_save_test");
    let path = dir.join("test.bin");
    let data = b"hello from save test";
    release::save_bytes(data, &path).unwrap();
    let read_back = std::fs::read(&path).unwrap();
    assert_eq!(read_back, data);
    std::fs::remove_dir_all(&dir).unwrap();
}
