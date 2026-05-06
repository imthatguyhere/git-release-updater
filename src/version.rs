use std::path::Path;

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

    /// Converts an OS string into a null-terminated UTF-16 buffer for Win32 APIs.
    fn wide(s: &OsStr) -> Vec<u16> {
        let mut v: Vec<u16> = s.encode_wide().collect();
        v.push(0);
        v
    }

    /// Reads a version resource string value from a PE file.
    ///
    /// The function first queries the file's language/codepage translation table,
    /// then reads the requested key from that localized string table.
    fn get_string_version(path: &Path, key: &str) -> Option<String> {
        let path_wide = wide(path.as_os_str());

        let mut dummy: u32 = 0;
        let info_size = unsafe { GetFileVersionInfoSizeW(path_wide.as_ptr(), &mut dummy) };

        if info_size == 0 {
            return None;
        }

        let mut buf: Vec<u8> = vec![0u8; info_size as usize];
        if unsafe {
            GetFileVersionInfoW(
                path_wide.as_ptr(),
                0,
                info_size,
                buf.as_mut_ptr() as *mut std::ffi::c_void,
            )
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

        let sub_block = format!("\\StringFileInfo\\{lang:04x}{codepage:04x}\\{key}");
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
        let slice = unsafe { std::slice::from_raw_parts(str_ptr as *const u16, str_len as usize) };
        String::from_utf16(slice).ok()
    }
}

//=-- ---------------------------------------------------------------------------
//=-- Public version helpers
//=-- ---------------------------------------------------------------------------

/// Strips the leading 'v' / 'V' from a tag name and returns the version string.
pub fn clean_tag(tag: &str) -> &str {
    tag.strip_prefix('v')
        .or_else(|| tag.strip_prefix('V'))
        .unwrap_or(tag)
}

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
        .ok_or_else(|| format!("no version info found in '{}'", path.display()))?;

    Ok(clean_version_string(&raw))
}

#[cfg(not(windows))]
pub fn get_local_version(_path: &Path) -> Result<String, String> {
    Err("file version extraction is only supported on Windows".to_string())
}

/// Cleans a raw version string:
/// - Split at '+' -> keep left side
/// - Remove any remaining '+' characters
fn clean_version_string(raw: &str) -> String {
    raw.split('+').next().unwrap_or("").replace('+', "")
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
}
