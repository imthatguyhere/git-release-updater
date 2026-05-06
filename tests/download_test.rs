use git_release_updater::download;

//=-- ---------------------------------------------------------------------------
//=-- save_bytes
//=-- ---------------------------------------------------------------------------

#[test]
fn test_save_bytes_roundtrip() {
    let dir = std::env::temp_dir().join("gru_download_save_test");
    let path = dir.join("test.bin");
    let data = b"hello from download save test";
    download::save_bytes(data, &path).unwrap();
    let read_back = std::fs::read(&path).unwrap();
    assert_eq!(read_back, data);
    std::fs::remove_dir_all(&dir).unwrap();
}

//=-- ---------------------------------------------------------------------------
//=-- download_bytes / download_and_hash are network-dependent and covered by
//=-- functional testing against GitHub API.
//=-- ---------------------------------------------------------------------------
