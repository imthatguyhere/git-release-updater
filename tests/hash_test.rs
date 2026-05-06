use git_release_updater::hash;

//=-- ---------------------------------------------------------------------------
//=-- sha256_bytes
//=-- ---------------------------------------------------------------------------

#[test]
fn test_sha256_bytes_known() {
    let result = hash::sha256_bytes(b"hello world");
    assert_eq!(
        result,
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}

#[test]
fn test_sha256_bytes_empty() {
    let result = hash::sha256_bytes(b"");
    assert_eq!(
        result,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

//=-- ---------------------------------------------------------------------------
//=-- hash_local_file
//=-- ---------------------------------------------------------------------------

#[test]
fn test_hash_local_file_nonexistent() {
    let result = hash::hash_local_file(std::path::Path::new("C:\\does_not_exist_12345.exe"));
    assert!(result.is_err());
}

#[test]
fn test_hash_local_file_empty() {
    let dir = std::env::temp_dir();
    let path = dir.join("gru_hash_test_empty.bin");
    std::fs::write(&path, []).unwrap();
    let result = hash::hash_local_file(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        result.unwrap(),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn test_hash_local_file_known() {
    let dir = std::env::temp_dir();
    let path = dir.join("gru_hash_test_known.bin");
    std::fs::write(&path, b"hello world").unwrap();
    let result = hash::hash_local_file(&path);
    std::fs::remove_file(&path).unwrap();
    assert_eq!(
        result.unwrap(),
        "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
    );
}
