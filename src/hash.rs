use sha2::{Digest, Sha256};
use std::io::Read;
use std::path::Path;

//=-- ---------------------------------------------------------------------------
//=-- SHA-256 helpers
//=-- ---------------------------------------------------------------------------

/// Computes SHA-256 of bytes.
pub fn sha256_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Computes the SHA-256 hash of a local file.
pub fn hash_local_file(path: &Path) -> Result<String, String> {
    let mut file =
        std::fs::File::open(path).map_err(|e| format!("cannot open '{}': {e}", path.display()))?;

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
//=-- Hash comparison helpers
//=-- ---------------------------------------------------------------------------

/// Strips "sha256:" or "SHA256:" prefix from a hash string, if present.
pub(crate) fn strip_hash_prefix(h: &str) -> &str {
    h.strip_prefix("sha256:")
        .or_else(|| h.strip_prefix("SHA256:"))
        .unwrap_or(h)
}

/// Compares two SHA-256 strings after normalizing optional `sha256:` prefixes.
pub(crate) fn hash_eq(left: &str, right: &str) -> bool {
    strip_hash_prefix(left).eq_ignore_ascii_case(strip_hash_prefix(right))
}

/// Compares the local file hash against the best available expected release hash.
///
/// GitHub asset digest takes priority over a CLI `--hash` value so the check
/// matches the integrity verification priority used before saving downloads.
pub(crate) fn local_hash_matches_expected(
    local_hash: Option<&String>,
    github_digest: Option<&String>,
    expected_hash: Option<&str>,
) -> Option<bool> {
    let expected = github_digest
        .map(String::as_str)
        .or_else(|| expected_hash.map(strip_hash_prefix));

    match (local_hash, expected) {
        (Some(local), Some(expected)) => Some(hash_eq(local, expected)),
        _ => None,
    }
}

//=-- ---------------------------------------------------------------------------
//=-- Inline tests (private fn coverage only; public API tested via tests/)
//=-- ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn test_hash_eq_ignores_prefix_and_case() {
        assert!(hash_eq("sha256:ABC123", "abc123"));
        assert!(hash_eq("ABC123", "SHA256:abc123"));
        assert!(!hash_eq("abc123", "def456"));
    }

    #[test]
    fn test_local_hash_matches_expected_uses_github_digest_first() {
        let local_hash = "abc123".to_string();
        let github_digest = "sha256:abc123".to_string();
        assert_eq!(
            local_hash_matches_expected(Some(&local_hash), Some(&github_digest), Some("def456")),
            Some(true)
        );
    }

    #[test]
    fn test_local_hash_matches_expected_uses_cli_hash_fallback() {
        let local_hash = "abc123".to_string();
        assert_eq!(
            local_hash_matches_expected(Some(&local_hash), None, Some("sha256:abc123")),
            Some(true)
        );
    }

    #[test]
    fn test_local_hash_matches_expected_unknown_without_hash_source() {
        let local_hash = "abc123".to_string();
        assert_eq!(
            local_hash_matches_expected(Some(&local_hash), None, None),
            None
        );
    }
}
