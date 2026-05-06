use crate::{hash, request};
use std::path::Path;

//=-- ---------------------------------------------------------------------------
//=-- Download + save helpers
//=-- ---------------------------------------------------------------------------

/// Downloads bytes from `url` into memory. Validates HTTP 2xx.
pub async fn download_bytes(url: &str) -> Result<Vec<u8>, String> {
    let resp = request::get_bytes(url).await?;
    if !(200..300).contains(&resp.status) {
        return Err(format!("download failed: HTTP {} from {url}", resp.status));
    }
    Ok(resp.body)
}

/// Saves bytes to `path`, creating parent dirs.
pub fn save_bytes(bytes: &[u8], path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("cannot create directory '{}': {e}", parent.display()))?;
    }
    std::fs::write(path, bytes).map_err(|e| format!("cannot write '{}': {e}", path.display()))?;
    println!("  Saved: {}", path.display());
    Ok(())
}

/// Downloads a file from `url` and returns its SHA-256 hash + bytes.
/// Does NOT save to disk — caller decides.
pub async fn download_and_hash(url: &str) -> Result<(String, Vec<u8>), String> {
    let bytes = download_bytes(url).await?;
    let hash = hash::sha256_bytes(&bytes);
    Ok((hash, bytes))
}
