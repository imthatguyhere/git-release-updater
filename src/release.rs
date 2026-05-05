use crate::request;
use sha2::{Sha256, Digest};
use std::io::Read;
use std::path::{Path, PathBuf};

//=-- ---------------------------------------------------------------------------
//=-- EXE path resolution
//=-- ---------------------------------------------------------------------------

/// Resolves the actual exe path from a user-provided path + asset filename.
///
/// If `base` is a directory (ends with separator or is an existing dir),
/// appends `asset_name`. Otherwise uses `base` as-is (full path with filename).
pub fn resolve_exe_path(base: &Path, asset_name: &str) -> PathBuf {
    let base_str = base.to_string_lossy();
    if base_str.ends_with('/') || base_str.ends_with('\\') || base.is_dir() {
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

    pub fn wants_hash(&self) -> bool {
        matches!(self, Self::Hash | Self::Both)
    }

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

/// Finds the download URL for a specific asset name inside a release.
pub fn find_asset_url<'a>(release: &'a GitHubRelease, target_exe: &str) -> Option<&'a str> {
    release
        .assets
        .iter()
        .find(|a| a.name.eq_ignore_ascii_case(target_exe))
        .map(|a| a.browser_download_url.as_str())
}

/// Strips the leading 'v' / 'V' from a tag name and returns the version string.
pub fn clean_tag(tag: &str) -> &str {
    tag.strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag)
}

/// Strips "sha256:" or "SHA256:" prefix from a hash string, if present.
fn strip_hash_prefix(h: &str) -> &str {
    h.strip_prefix("sha256:")
        .or_else(|| h.strip_prefix("SHA256:"))
        .unwrap_or(h)
}

//=-- ---------------------------------------------------------------------------
//=-- Download + hash (+ optional save)
//=-- ---------------------------------------------------------------------------

/// Downloads a file from `url` and returns its SHA-256 hash as a hex string.
/// If `save_path` is provided, the downloaded bytes are written to that file.
pub async fn download_and_hash(url: &str, save_path: Option<&Path>) -> Result<String, String> {
    let resp = request::get_bytes(url).await?;

    if !(200..300).contains(&resp.status) {
        return Err(format!(
            "download failed: HTTP {} from {url}",
            resp.status
        ));
    }

    //=-- Hash
    let mut hasher = Sha256::new();
    hasher.update(&resp.body);
    let hash = hex::encode(hasher.finalize());

    //=-- Save to disk if path given
    if let Some(path) = save_path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| format!("cannot create directory '{}': {e}", parent.display()))?;
        }
        std::fs::write(path, &resp.body)
            .map_err(|e| format!("cannot write '{}': {e}", path.display()))?;
        println!("  Saved: {}", path.display());
    }

    Ok(hash)
}

/// Computes the SHA-256 hash of a local file.
pub fn hash_local_file(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| format!("cannot open '{}': {e}", path.display()))?;

    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("error reading '{}': {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

//=-- ---------------------------------------------------------------------------
//=-- Version extraction (Windows only)
//=-- ---------------------------------------------------------------------------

#[cfg(windows)]
mod win_version {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileVersionInfoSizeW, GetFileVersionInfoW, VerQueryValueW,
    };

    /// Returns the "Product version" string from a PE file's version resource.
    pub fn get_product_version(path: &Path) -> Option<String> {
        get_string_version(path, "ProductVersion")
    }

    /// Returns the "File version" string from a PE file's version resource.
    pub fn get_file_version(path: &Path) -> Option<String> {
        get_string_version(path, "FileVersion")
    }

    fn wide(s: &OsStr) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_wide().collect();
        v.push(0);
        v
    }

    fn get_string_version(path: &Path, key: &str) -> Option<String> {
        let path_wide = wide(path.as_os_str());

        let mut dummy: u32 = 0;
        let info_size =
            unsafe { GetFileVersionInfoSizeW(path_wide.as_ptr(), &mut dummy) };

        if info_size == 0 {
            return None;
        }

        let mut buf: Vec<u8> = vec![0u8; info_size as usize];
        if unsafe {
            GetFileVersionInfoW(path_wide.as_ptr(), 0, info_size, buf.as_mut_ptr() as *mut std::ffi::c_void)
        } == 0
        {
            return None;
        }

        //=-- query the translation table first
        let mut trans_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut trans_len: u32 = 0;
        let trans_query = wide(OsStr::new("\\VarFileInfo\\Translation"));
        if unsafe {
            VerQueryValueW(
                buf.as_ptr() as *const std::ffi::c_void,
                trans_query.as_ptr(),
                &mut trans_ptr,
                &mut trans_len,
            )
        } == 0
            || trans_ptr.is_null()
            || trans_len == 0
        {
            return None;
        }

        //=-- translation is an array of (lang, codepage) u16 pairs; use the first
        let lang = unsafe { *(trans_ptr as *const u16) };
        let codepage = unsafe { *(trans_ptr.add(2) as *const u16) };

        let sub_block = format!(
            "\\StringFileInfo\\{lang:04x}{codepage:04x}\\{key}"
        );
        let sub_block_wide = wide(OsStr::new(&sub_block));

        let mut str_ptr: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut str_len: u32 = 0;
        if unsafe {
            VerQueryValueW(
                buf.as_ptr() as *const std::ffi::c_void,
                sub_block_wide.as_ptr(),
                &mut str_ptr,
                &mut str_len,
            )
        } == 0
            || str_ptr.is_null()
            || str_len == 0
        {
            return None;
        }

        //=-- str_len is in characters, excluding the null terminator
        let slice = unsafe {
            std::slice::from_raw_parts(str_ptr as *const u16, str_len as usize)
        };
        String::from_utf16(slice).ok()
    }
}

//=-- ---------------------------------------------------------------------------
//=-- Public version helpers
//=-- ---------------------------------------------------------------------------

/// Extracts the version string from a local exe for comparison.
///
/// Strategy (per user spec):
/// 1. Try File version
/// 2. Fall back to Product version
/// 3. Split at '+', keep left of '+'
/// 4. Remove any remaining '+' characters
#[cfg(windows)]
pub fn get_local_version(path: &Path) -> Result<String, String> {
    let raw = win_version::get_file_version(path)
        .or_else(|| win_version::get_product_version(path))
        .ok_or_else(|| {
            format!(
                "no version info found in '{}'",
                path.display()
            )
        })?;

    Ok(clean_version_string(&raw))
}

#[cfg(not(windows))]
pub fn get_local_version(_path: &Path) -> Result<String, String> {
    Err("file version extraction is only supported on Windows".to_string())
}

/// Cleans a raw version string:
/// - Split at '+' → keep left side
/// - Remove any remaining '+' characters
fn clean_version_string(raw: &str) -> String {
    raw.split('+')
        .next()
        .unwrap_or("")
        .replace('+', "")
}

//=-- ---------------------------------------------------------------------------
//=-- Check logic
//=-- ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct CheckResult {
    pub mode: CheckMode,
    pub latest_tag: String,
    pub latest_version: String,
    pub download_performed: bool,
    /// Hash of the downloaded file (always set when download happens)
    pub downloaded_hash: Option<String>,
    /// Local exe version string (version/both mode)
    pub local_version: Option<String>,
    /// Whether local version starts with remote tag version
    pub version_match: Option<bool>,
    /// Hash of the local exe (hash/both mode)
    pub local_hash: Option<String>,
    /// Whether local hash equals downloaded hash (hash/both mode)
    pub hash_match: Option<bool>,
    /// Expected hash from --hash CLI arg (after sha256: prefix stripping)
    pub expected_hash: Option<String>,
    /// Whether downloaded hash matches the expected hash
    pub expected_hash_ok: Option<bool>,
}

/// Runs the configured check.
pub async fn run_check(
    repo_url: &str,
    target_exe: &str,
    version_filter: &str,
    mode: CheckMode,
    local_exe: Option<&Path>,
    expected_hash: Option<&str>,
) -> Result<CheckResult, String> {
    //=-- 1. Fetch latest release
    let release = get_latest_release(repo_url).await?;
    let latest_tag = release.tag_name.clone();
    let latest_version = clean_tag(&release.tag_name).to_string();

    println!("Latest release: {latest_version}  (tag: {latest_tag})");

    if version_filter != "latest" {
        println!("Requested version: {version_filter}");
    }

    let dl_url = find_asset_url(&release, target_exe).ok_or_else(|| {
        format!(
            "asset '{target_exe}' not found in release {latest_tag}. Available: {}",
            release.assets.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
        )
    })?;

    //=-- 2. Version check (only in version/both mode)
    let (local_version, version_match) = if mode.wants_version() {
        if let Some(exe) = local_exe {
            match get_local_version(exe) {
                Ok(v) => {
                    let matches = v.starts_with(&latest_version);
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

    //=-- 3. Decide if download is needed
    //=--    hash mode → always download
    //=--    version/both → download only if version mismatch or unknown
    let needs_download = match &mode {
        CheckMode::Hash => true,
        CheckMode::Version | CheckMode::Both => !version_match.unwrap_or(false),
    };

    //=-- 4. Download and hash (if needed)
    let (downloaded_hash, local_hash, hash_match, expected_hash_ok) = if needs_download {
        let save_path = local_exe;
        let dl_hash = download_and_hash(dl_url, save_path).await?;

        //=-- Check against --hash (expected hash) if provided
        let exp_ok = expected_hash.map(|eh| {
            let eh = strip_hash_prefix(eh);
            dl_hash == eh
        });

        //=-- Local vs download comparison (only in hash/both mode)
        let (local_h, h_match) = if mode.wants_hash() {
            if let Some(exe) = local_exe {
                match hash_local_file(exe) {
                    Ok(h) => {
                        let matches = h == dl_hash;
                        (Some(h), Some(matches))
                    }
                    Err(e) => {
                        eprintln!("Warning: could not hash local file: {e}");
                        (None, None)
                    }
                }
            } else {
                (None, None)
            }
        } else {
            //=-- version-only mode: still hash the local file if it exists
            //=-- for informational purposes, but no comparison needed
            let local_h = local_exe.and_then(|exe| hash_local_file(exe).ok());
            (local_h, None)
        };

        (Some(dl_hash), local_h, h_match, exp_ok)
    } else {
        (None, None, None, None)
    };

    //=-- Strip sha256: prefix from expected_hash for display
    let expected_hash_display = expected_hash.map(strip_hash_prefix).map(String::from);

    Ok(CheckResult {
        mode,
        latest_tag,
        latest_version,
        download_performed: needs_download,
        downloaded_hash,
        local_version,
        version_match,
        local_hash,
        hash_match,
        expected_hash: expected_hash_display,
        expected_hash_ok,
    })
}

/// Prints a human-readable summary of the check result.
pub fn print_result(result: &CheckResult) {
    println!();
    println!("═══════════════════════════════════════");
    println!("  Release Check Results");
    println!("═══════════════════════════════════════");
    println!("  Release tag:     {}", result.latest_tag);

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
        println!("    release: {}", result.latest_version);
    }

    //=-- Hash check (local vs downloaded)
    if result.mode.wants_hash() && result.download_performed {
        print!("  Hash check:      ");
        match result.hash_match {
            Some(true) => println!("✓ MATCH"),
            Some(false) => println!("✗ MISMATCH ⇒ update needed"),
            None => println!("? no local file to compare"),
        }
        if let Some(ref lh) = result.local_hash {
            println!("    local:    {lh}");
        }
        if let Some(ref dh) = result.downloaded_hash {
            println!("    remote:   {dh}");
        }
    }

    //=-- Download info
    if result.download_performed {
        if let Some(ref dh) = result.downloaded_hash {
            println!("  Download hash:   {dh}");
        }
    } else {
        println!("  Download:        ✗ not performed (already current)");
    }

    //=-- Expected hash verification
    if let Some(ref eh) = result.expected_hash {
        print!("  Expected hash:   ");
        match result.expected_hash_ok {
            Some(true) => println!("✓ MATCH"),
            Some(false) => println!("✗ MISMATCH"),
            None => println!("? not checked"),
        }
        println!("    hash: {eh}");
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
        if result.download_performed && all_good {
            println!("  Result: ✓ up-to-date (download matches)");
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
    fn test_clean_version_string_no_plus() {
        assert_eq!(clean_version_string("1.2.3.0"), "1.2.3.0");
    }

    #[test]
    fn test_clean_version_string_with_plus() {
        assert_eq!(clean_version_string("1.2.3+build1"), "1.2.3");
    }

    #[test]
    fn test_clean_version_string_internal_plus() {
        assert_eq!(clean_version_string("1.2+3.0"), "1.2");
    }

    #[test]
    fn test_clean_version_string_empty() {
        assert_eq!(clean_version_string(""), "");
    }

    #[test]
    fn test_clean_version_string_only_plus() {
        assert_eq!(clean_version_string("+"), "");
    }

    #[test]
    fn test_strip_hash_prefix_sha256_lower() {
        assert_eq!(strip_hash_prefix("sha256:abc123"), "abc123");
    }

    #[test]
    fn test_strip_hash_prefix_sha256_upper() {
        assert_eq!(strip_hash_prefix("SHA256:abc123"), "abc123");
    }

    #[test]
    fn test_strip_hash_prefix_no_prefix() {
        assert_eq!(strip_hash_prefix("abc123"), "abc123");
    }

    #[test]
    fn test_strip_hash_prefix_empty() {
        assert_eq!(strip_hash_prefix(""), "");
    }
}
