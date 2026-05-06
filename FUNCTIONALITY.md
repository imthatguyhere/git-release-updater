# Functionality

## Data Flow

```md
main (binary entry: cli.parse + dotenv)
  │
  ├── calls git_release_updater::release::resolve_exe_path()
  │
  └── calls git_release_updater::release::run_check()
        │
        ├── release::parse_repo_url()
        ├── request::get_json()     → GitHub release API
        ├── release::find_asset()   → asset (url + digest)
        │
        ├── [pre-download checks]
        │     ├── [version check]
        │     │     ├── release::get_local_version()
        │     │     │     └── win_version::get_file_version / get_product_version
        │     │     └── release::clean_tag() + release::clean_version_string()
        │     └── [local hash taken BEFORE download]
        │           └── release::hash_local_file()
        │
        ├── [download to memory only]
        │     ├── request::get_bytes()     → Vec<u8>
        │     └── release::sha256_bytes()  → hex hash
        │
        ├── [verify download against hash source]
        │     ├── release asset `digest` (priority) ↛ release::strip_hash_prefix()
        │     ├── --hash CLI arg (fallback) ↛ release::strip_hash_prefix()
        │     ├── both absent ↛ Err (discard, don't save)
        │     └── mismatch ↛ Err (discard, don't save)
        │
        ├── [compare local vs download (hash mode)]
        │     └── skip save if hashes match
        │
        └── [save to disk (only after verification)]
              └── release::save_bytes(path)
```

## Configuration

Configuration sources, in priority order:

1. CLI flags (highest)
2. `.env` file (loaded via `dotenvy::dotenv()` plus a line-based fallback that preserves unquoted Windows paths ending in `\` — copy `.env.example` to `.env`)
3. Built-in defaults

| Variable | CLI flag | Default | Description |
| - | - | - | - |
| `REPO_URL` | `--repo` | `https://github.com/microsoft/winget-create` | GitHub repo to check |
| `TARGET_EXE` | `--target` | `wingetcreate.exe` | Asset filename within the release |
| `VERSION` | `--version` | `latest` | Release tag filter |
| `MODE` | `--mode` | `both` | `hash` / `version` / `both` |
| `EXE_PATH` | `--exe` | `C:\ProgramData\ITGH\git-release-updater` *(dir — appends TARGET_EXE)* | Local exe path (dir or full) |
| `OUTPUT_PATH` | `--output` | *(uses EXE_PATH)* | Save destination (dir or full) |
| `EXPECTED_HASH` | `--hash` | *(none — uses GitHub digest)* | SHA-256 for download integrity — fatal if mismatch. Falls back to GitHub asset `digest` when present. |

> **Hash source priority:** GitHub release asset `digest` field (auto-detected) → `--hash` CLI arg. If neither is available and a save is needed, the download is discarded with an error — `--hash` must be provided to proceed.

The `--exe` / `--output` paths are resolved by `resolve_exe_path()`: if it's a directory (trailing separator, existing dir, or extensionless path), the `TARGET_EXE` filename is appended. With the default `EXE_PATH`, the effective path becomes `C:\ProgramData\ITGH\git-release-updater\wingetcreate.exe`.

A template configuration file is provided at `.env.example`.

## Build and Release Packaging

Standard `cargo build --release` for the Rust build. Release packaging also has a repo-root PowerShell entrypoint at [scripts/build-release.ps1](scripts/build-release.ps1), which runs the release build and copies the binary into `dist/`.

The workspace’s VS Code task configuration marks `Build release script` as the default build task, so `Ctrl+Shift+B` and `Terminal > Run Build Task...` invoke that script when the workspace is open.

## Modules

### lib

- **Purpose:** Library crate root. Declares and re-exports all public modules so they can be consumed by both the `main` binary and integration tests in `tests/`.
- **Declaration:**

  ```rust
  pub mod release;
  pub mod request;
  pub mod util;
  ```

- **Public:** Re-exports everything from `release`, `request`, `util` under the `git_release_updater` crate namespace.

### main

- **Purpose:** Binary entry point. Thin wrapper over the `git_release_updater` library crate.
- **Public:** `fn main()` — async entry via `#[tokio::main]`.
- **Imports:** `use git_release_updater::release;` — does **not** declare sub-modules directly (they live in `lib.rs`).
- **Key algorithms:** Configuration prioritization (CLI > `.env` > default).

### release

- **Purpose:** Core release-checking logic. Fetches GitHub release metadata, downloads release assets, computes SHA-256 hashes, extracts PE version info, compares local vs remote, and formats results.
- **Types:**
  - `CheckMode` — enum: `Hash`, `Version`, `Both`
  - `CheckResult` — holds all comparison results and download metadata. Fields include `github_digest` (from GitHub asset metadata), `cli_expected_hash` (from `--hash` arg), `actual_save_path` (effective save location), `save_skipped_reason` (why save was skipped), `expected_hash_ok` (whether the hash check passed).
  - `GitHubAsset` — deserialized from GitHub API: `name`, `browser_download_url`, `digest` (optional `sha256:` hex string from GitHub release listing).
  - `GitHubRelease` — deserialized from GitHub API: `tag_name`, `assets`
- **Public functions:**
  - `resolve_exe_path(base, asset_name) -> PathBuf` — resolves a path that may be a directory or a full file path. Directory intent is detected from trailing separators, existing directories, or extensionless paths.
  - `parse_repo_url(url) -> Result<(String, String)>` — extracts owner/repo from a GitHub URL
  - `get_latest_release(repo_url) -> Result<GitHubRelease>` — fetches latest release from GitHub API
  - `get_release_by_tag(repo_url, tag) -> Result<GitHubRelease>` — fetches a specific release by tag from GitHub API
  - `find_asset_url(release, target_exe) -> Option<&str>` — finds a release asset's download URL by name
  - `find_asset(release, target_exe) -> Option<&GitHubAsset>` — returns the full asset struct (including `digest`)
  - `clean_tag(tag) -> &str` — strips leading `v`/`V` prefix
  - `download_bytes(url) -> Result<Vec<u8>>` — downloads to memory only, validates HTTP 2xx
  - `sha256_bytes(bytes) -> String` — computes SHA-256 of a byte slice
  - `save_bytes(bytes, path)` — writes bytes to disk, creates parent dirs
  - `download_and_hash(url) -> Result<(String, Vec<u8>)>` — downloads to memory, returns (hash, bytes)
  - `hash_local_file(path) -> Result<String>` — computes SHA-256 of a local file (streaming, 8KB buffer)
  - `get_local_version(path) -> Result<String>` — extracts version from a PE file via Win32 API
  - `run_check(...) -> Result<CheckResult>` — main orchestration function
  - `print_result(result)` — formats the check result to stdout
  - `CheckMode::from_str(s) -> Result<Self>` — parses mode string
  - `CheckMode::wants_hash() -> bool`
  - `CheckMode::wants_version() -> bool`
- **Key algorithms:**
  - **Version comparison:** Local PE version extracted via `GetFileVersionInfoSizeW` / `GetFileVersionInfoW` / `VerQueryValueW`. Falls back FileVersion → ProductVersion. Cleans metadata: split at `+`, keep left, strip remaining `+`. Compared via `starts_with` against the cleaned release tag.
  - **Download decision:** `hash` mode always downloads. `version`/`both` mode skips download if local version already matches the release tag.
  - **Safety sequence:** Local hash taken **before** download. Bytes downloaded to memory only. Hash verified before save (GitHub digest priority → `--hash` fallback). If neither hash source is available, download is discarded with error — no file written. Hash mismatch returns `Err`.
  - **Hash source priority:** GitHub release asset `digest` field (auto-detected from API response) → `--hash` CLI arg. If a save is needed and neither is available, the operation fails.
  - **Hash mode save logic:** Save only if local hash differs from download hash. If hashes match, the local file is already current — no overwrite needed.
- **Module-level functions (private):** `clean_version_string(raw) -> String` — splits at `+`, removes all `+` characters.
- **Sub-modules:**
  - `win_version` (Windows only): `get_file_version(path)`, `get_product_version(path)` — wraps Win32 version API calls.

### request

- **Purpose:** HTTP client wrapper. Abstracts `reqwest` for common methods.
- **Types:**
  - `Response` — status code + body string
  - `BytesResponse` — status code + raw byte vector (for binary downloads)
  - `Method` — enum: Get, Post, Put, Delete
- **Public functions:**
  - `get_bytes(url)` — raw binary GET (used by `download_and_hash`)
  - `request()` — generic HTTP call with optional JSON body
  - `request_json()` — HTTP call + JSON deserialization
  - `get()` — raw GET string response
  - `get_json()` — GET with JSON deserialization
  - `post()` — raw POST
  - `post_json()` — POST with JSON deserialization

### util

- **Purpose:** General-purpose helpers.
- **Public functions:**
  - `current_timestamp()` — Unix timestamp in seconds
  - `format_timestamp()` — human-readable date-time string
  - `truncate()` — string truncation with ellipsis
  - `is_valid_url()` — URL prefix check
- **Private functions:**
  - `days_to_date(days) -> (year, month, day)` — converts days since Unix epoch to calendar date (Howard Hinnant algorithm)

## Public API

### *(All public items are documented in their module sections above.)*
