use crate::{download, hash, request, version};
use std::path::{Path, PathBuf};

pub use crate::download::{download_and_hash, download_bytes, save_bytes};
pub use crate::hash::{hash_local_file, sha256_bytes};
pub use crate::version::{clean_tag, get_local_version};

//=-- ---------------------------------------------------------------------------
//=-- EXE path resolution
//=-- ---------------------------------------------------------------------------

/// Resolves the actual exe path from a user-provided path + asset filename.
///
/// If `base` is a directory (ends with separator, is an existing dir, or has no
/// file extension), appends `asset_name`. Otherwise uses `base` as-is (full path
/// with filename).
pub fn resolve_exe_path(base: &Path, asset_name: &str) -> PathBuf {
    let base_str = base.to_string_lossy();
    if base_str.ends_with('/')
        || base_str.ends_with('\\')
        || base.is_dir()
        || base.extension().is_none()
    {
        base.join(asset_name)
    } else {
        base.to_path_buf()
    }
}

//=-- ---------------------------------------------------------------------------
//=-- GitHub API types
//=-- ---------------------------------------------------------------------------

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    /// GitHub release asset digest (e.g. "sha256:9f56bb...").
    /// May not be present on all GitHub instances; `#[serde(default)]` handles that.
    #[serde(default)]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub assets: Vec<GitHubAsset>,
}

//=-- ---------------------------------------------------------------------------
//=-- Check mode
//=-- ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum CheckMode {
    Hash,
    Version,
    Both,
}

impl CheckMode {
    /// Parses a check mode string into its enum variant.
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "hash" => Ok(Self::Hash),
            "version" => Ok(Self::Version),
            "both" => Ok(Self::Both),
            other => Err(format!(
                "unknown check mode: '{other}'. Valid modes: hash, version, both"
            )),
        }
    }

    /// Returns whether this mode performs hash comparison or verification.
    pub fn wants_hash(&self) -> bool {
        matches!(self, Self::Hash | Self::Both)
    }

    /// Returns whether this mode performs PE version comparison.
    pub fn wants_version(&self) -> bool {
        matches!(self, Self::Version | Self::Both)
    }
}

//=-- ---------------------------------------------------------------------------
//=-- GitHub helpers
//=-- ---------------------------------------------------------------------------

/// Parses a GitHub URL like `https://github.com/{owner}/{repo}` into
/// `(owner, repo)`.  Trailing slashes and `.git` suffixes are stripped.
pub fn parse_repo_url(url: &str) -> Result<(String, String), String> {
    let url = url.trim_end_matches('/');
    let url = url.strip_suffix(".git").unwrap_or(url);

    //=-- Expect: https://github.com/<owner>/<repo>
    let path = url
        .strip_prefix("https://github.com/")
        .or_else(|| url.strip_prefix("http://github.com/"))
        .ok_or_else(|| {
            format!("not a github.com URL: '{url}'. Expected https://github.com/<owner>/<repo>")
        })?;

    let parts: Vec<&str> = path.split('/').filter(|p| !p.is_empty()).collect();
    if parts.len() != 2 {
        return Err(format!(
            "invalid GitHub repo URL: '{url}'. Expected https://github.com/<owner>/<repo>"
        ));
    }

    Ok((parts[0].to_string(), parts[1].to_string()))
}

/// Fetches the latest release from a GitHub repo.
pub async fn get_latest_release(repo_url: &str) -> Result<GitHubRelease, String> {
    let (owner, repo) = parse_repo_url(repo_url)?;
    let api_url = format!("https://api.github.com/repos/{owner}/{repo}/releases/latest");
    request::get_json::<GitHubRelease>(&api_url).await
}

/// Fetches a release by tag from a GitHub repo.
pub async fn get_release_by_tag(repo_url: &str, tag: &str) -> Result<GitHubRelease, String> {
    let (owner, repo) = parse_repo_url(repo_url)?;
    let api_url = format!("https://api.github.com/repos/{owner}/{repo}/releases/tags/{tag}");
    request::get_json::<GitHubRelease>(&api_url).await
}

/// Finds the download URL for a specific asset name inside a release.
pub fn find_asset_url<'a>(release: &'a GitHubRelease, target_exe: &str) -> Option<&'a str> {
    release
        .assets
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(target_exe))
        .map(|a| a.browser_download_url.as_str())
}

/// Finds the full asset struct by name inside a release.
/// Used when digest verification is needed.
pub fn find_asset<'a>(release: &'a GitHubRelease, target_exe: &str) -> Option<&'a GitHubAsset> {
    release
        .assets
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(target_exe))
}

/// Determines whether the selected mode needs to download the release asset.
///
/// `Both` mode uses the local hash comparison against the expected release hash,
/// so a matching version alone never skips hash validation.
fn should_download(
    mode: &CheckMode,
    version_match: Option<bool>,
    local_expected_match: Option<bool>,
) -> bool {
    match mode {
        CheckMode::Hash => true,
        CheckMode::Version => !version_match.unwrap_or(false),
        CheckMode::Both => local_expected_match != Some(true),
    }
}

//=-- ---------------------------------------------------------------------------
//=-- Check logic
//=-- ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct CheckResult {
    pub mode: CheckMode,
    pub release_tag: String,
    pub release_version: String,
    pub download_performed: bool,
    pub file_saved: bool,
    /// Why save was skipped (None if saved or download not needed)
    pub save_skipped_reason: Option<String>,
    /// Path where file was actually saved (effective path, not just --output)
    pub actual_save_path: Option<PathBuf>,
    /// Hash of the downloaded file (set when download happens)
    pub downloaded_hash: Option<String>,
    /// Local exe version string
    pub local_version: Option<String>,
    /// Whether local version starts with release tag version
    pub version_match: Option<bool>,
    /// Hash of the local exe (taken BEFORE download)
    pub local_hash: Option<String>,
    /// Whether local hash equals downloaded hash
    pub hash_match: Option<bool>,
    /// Digest from GitHub release asset metadata (sha256: prefix stripped)
    pub github_digest: Option<String>,
    /// Expected hash from --hash CLI arg (sha256: prefix stripped)
    pub cli_expected_hash: Option<String>,
    /// Whether the hash check passed (uses github_digest > cli_expected_hash)
    pub expected_hash_ok: Option<bool>,
}

/// Runs the configured check.
///
/// Flow:
/// 1. Fetch release (latest or by tag)
/// 2. Find matching asset (URL + digest)
/// 3. Pre-download checks (version, local hash)
/// 4. Download to memory only
/// 5. Verify download hash against GitHub digest → fail if mismatch
/// 6. Fall back to --hash verification if no GitHub digest
/// 7. Require hash source (GitHub digest or --hash) before saving (any mode)
/// 8. Save to effective output path only if update needed
pub async fn run_check(
    repo_url: &str,
    target_exe: &str,
    version_filter: &str,
    mode: CheckMode,
    local_exe: Option<&Path>,
    expected_hash: Option<&str>,
    output_path: Option<&Path>,
) -> Result<CheckResult, String> {
    //=-- 1. Fetch release (latest or specific tag)
    let release = if version_filter == "latest" || version_filter.is_empty() {
        get_latest_release(repo_url).await?
    } else {
        get_release_by_tag(repo_url, version_filter).await?
    };
    let release_tag = release.tag_name.clone();
    let release_version = version::clean_tag(&release.tag_name).to_string();

    println!("Release: {release_version}  (tag: {release_tag})");

    let asset = find_asset(&release, target_exe).ok_or_else(|| {
        format!(
            "asset '{target_exe}' not found in release {release_tag}. Available: {}",
            release
                .assets
                .iter()
                .map(|a| a.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;
    let dl_url = asset.browser_download_url.as_str();
    let gh_digest = asset
        .digest
        .as_ref()
        .map(|d| hash::strip_hash_prefix(d).to_string());

    //=-- 2. Pre-download checks (read local BEFORE download)

    //=-- Version check (version/both mode)
    let (local_version, version_match) = if mode.wants_version() {
        if let Some(exe) = local_exe {
            match version::get_local_version(exe) {
                Ok(v) => {
                    let matches = v.starts_with(&release_version);
                    (Some(v), Some(matches))
                }
                Err(e) => {
                    eprintln!("Warning: {e}");
                    (None, None)
                }
            }
        } else {
            (None, None)
        }
    } else {
        (None, None)
    };

    //=-- Local hash (taken BEFORE any download, for hash/both mode)
    let local_hash = if mode.wants_hash() {
        local_exe.and_then(|exe| hash::hash_local_file(exe).ok())
    } else {
        None
    };

    let local_expected_match =
        hash::local_hash_matches_expected(local_hash.as_ref(), gh_digest.as_ref(), expected_hash);

    //=-- 3. Decide if download needed
    //=--    hash mode -> always download (need remote hash)
    //=--    version mode -> download only if version mismatch or unknown
    //=--    both mode -> verify hash too; skip download only when local hash already matches expected digest
    let needs_download = should_download(&mode, version_match, local_expected_match);

    //=-- 4. Download to memory, verify, then save
    let (
        downloaded_hash,
        hash_match,
        expected_hash_ok,
        file_saved,
        save_skipped_reason,
        actual_save_path,
    ) = if needs_download {
        let (dl_hash, dl_bytes) = download::download_and_hash(dl_url).await?;

        //=-- Resolve effective save path (--output → EXE_PATH → None)
        let effective_save = output_path.or(local_exe);

        //=-- Determine expected hash priority: GitHub digest > --hash > None
        let (exp_ok, gh_exp_hash) = match (&gh_digest, expected_hash) {
            (Some(gd), _) => {
                let matches = hash::hash_eq(&dl_hash, gd);
                (Some(matches), Some(gd.clone()))
            }
            (None, Some(eh)) => {
                let eh_clean = hash::strip_hash_prefix(eh).to_string();
                let matches = hash::hash_eq(&dl_hash, &eh_clean);
                (Some(matches), Some(eh_clean))
            }
            (None, None) => (None, None),
        };

        //=-- Fail fast if expected hash mismatch
        if exp_ok == Some(false) {
            let exp_str = gh_exp_hash.as_deref().unwrap_or("???");
            return Err(format!(
                "Hash mismatch! Expected {exp_str}, got {dl_hash}. Download discarded."
            ));
        }

        //=-- Hash comparison (local vs download)
        let h_match = if mode.wants_hash() {
            local_hash.as_ref().map(|lh| hash::hash_eq(lh, &dl_hash))
        } else {
            None
        };

        //=-- Decide whether to save
        //=--   hash/both: save only if hashes differ
        //=--   version: save (version already confirmed outdated)
        let should_save = match &mode {
            CheckMode::Hash | CheckMode::Both => h_match != Some(true),
            CheckMode::Version => true,
        };

        //=-- Require hash source before saving (any mode)
        if should_save && gh_digest.is_none() && expected_hash.is_none() {
            return Err(
                "Cannot verify download: GitHub asset has no digest and --hash was not provided. "
                    .to_string()
                    + "Provide --hash to enable integrity verification before saving.",
            );
        }

        let (saved, skip_reason, actual_path) = if should_save {
            match effective_save {
                Some(path) => {
                    download::save_bytes(&dl_bytes, path)?;
                    (true, None, Some(path.to_path_buf()))
                }
                None => (
                    false,
                    Some("no output path configured — set EXE_PATH or OUTPUT_PATH".into()),
                    None,
                ),
            }
        } else {
            (
                false,
                Some("local file hash already matches — no save needed".into()),
                None,
            )
        };

        (
            Some(dl_hash),
            h_match,
            exp_ok,
            saved,
            skip_reason,
            actual_path,
        )
    } else {
        (
            None,
            local_expected_match,
            local_expected_match,
            false,
            None,
            None,
        )
    };

    Ok(CheckResult {
        mode,
        release_tag,
        release_version,
        download_performed: needs_download,
        file_saved,
        save_skipped_reason,
        actual_save_path,
        downloaded_hash,
        local_version,
        version_match,
        local_hash,
        hash_match,
        github_digest: gh_digest.clone(),
        cli_expected_hash: expected_hash.map(hash::strip_hash_prefix).map(String::from),
        expected_hash_ok,
    })
}

/// Prints a human-readable summary of the check result.
pub fn print_result(result: &CheckResult) {
    println!();
    println!("═══════════════════════════════════════");
    println!("  Release Check Results");
    println!("═══════════════════════════════════════");
    println!("  Release tag:     {}", result.release_tag);

    //=-- Version check
    if result.mode.wants_version() {
        print!("  Version check:   ");
        match result.version_match {
            Some(true) => println!("✓ MATCH"),
            Some(false) => println!("✗ MISMATCH ⇒ update needed"),
            None => println!("? skipped"),
        }
        if let Some(ref lv) = result.local_version {
            println!("    local:   {lv}");
        }
        println!("    release: {}", result.release_version);
    }

    //=-- Hash check (local vs downloaded)
    if result.mode.wants_hash() && (result.download_performed || result.hash_match.is_some()) {
        print!("  Hash check:      ");
        match result.hash_match {
            Some(true) => println!("✓ MATCH (already current)"),
            Some(false) => println!("✗ MISMATCH ⇒ update needed"),
            None => println!("? no local file to compare"),
        }
        if let Some(ref lh) = result.local_hash {
            println!("    local:    {lh}");
        }
        if let Some(ref dh) = result.downloaded_hash {
            println!("    remote:   {dh}");
        } else if let Some(ref gd) = result.github_digest {
            println!("    remote:   {gd}");
        } else if let Some(ref eh) = result.cli_expected_hash {
            println!("    remote:   {eh}");
        }
    }

    //=-- GitHub digest check
    if let Some(ref gd) = result.github_digest {
        print!("  GitHub digest:   ");
        match result.expected_hash_ok {
            Some(true) => println!("✓ MATCH"),
            Some(false) => println!("✗ MISMATCH — download discarded"),
            None => println!("? not checked"),
        }
        println!("    hash: {gd}");
    }

    //=-- CLI expected hash (only if no GitHub digest)
    if result.github_digest.is_none() {
        if let Some(ref eh) = result.cli_expected_hash {
            print!("  Expected hash:   ");
            match result.expected_hash_ok {
                Some(true) => println!("✓ MATCH"),
                Some(false) => println!("✗ MISMATCH — download discarded"),
                None => println!("? not checked"),
            }
            println!("    hash: {eh}");
        }
    }

    //=-- Download + save info
    if result.download_performed && result.file_saved {
        if let Some(ref dh) = result.downloaded_hash {
            println!("  Download hash:   {dh}");
        }
        if let Some(ref ap) = result.actual_save_path {
            println!("  Saved to:        {}", ap.display());
        }
    } else if result.download_performed && !result.file_saved {
        let reason = result
            .save_skipped_reason
            .as_deref()
            .unwrap_or("unknown reason");
        println!("  Save skipped:    {reason}");
        if let Some(ref dh) = result.downloaded_hash {
            println!("  Download hash:   {dh}");
        }
    } else {
        println!("  Download:        not performed (already current)");
    }

    //=-- Summary line
    println!("═══════════════════════════════════════");

    let any_comparison = result.version_match.is_some()
        || result.hash_match.is_some()
        || result.expected_hash_ok.is_some();

    if !any_comparison && result.download_performed {
        println!("  Result: downloaded (no local file to compare)");
    } else if !any_comparison {
        println!("  Result: no checks performed");
    } else {
        let all_good = result.version_match.unwrap_or(true)
            && result.hash_match.unwrap_or(true)
            && result.expected_hash_ok.unwrap_or(true);
        if result.file_saved {
            println!("  Result: ✓ saved (file updated)");
        } else if result.download_performed && !result.file_saved && all_good {
            println!("  Result: ✓ up-to-date (no save needed)");
        } else if !result.download_performed && all_good {
            println!("  Result: ✓ up-to-date");
        } else {
            println!("  Result: ✗ update needed");
        }
    }
    println!("═══════════════════════════════════════");
    println!();
}

//=-- ---------------------------------------------------------------------------
//=-- Inline tests (private fn coverage only; public API tested via tests/)
//=-- ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_download_both_when_version_matches_but_hash_differs() {
        assert!(should_download(&CheckMode::Both, Some(true), Some(false)));
    }

    #[test]
    fn test_should_download_both_skips_when_local_hash_matches_digest() {
        assert!(!should_download(&CheckMode::Both, Some(true), Some(true)));
    }
}
