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
        ├── release::find_asset_url()
        │
        ├── [version check]
        │     ├── release::get_local_version()
        │     │     └── win_version::get_file_version / get_product_version
        │     └── release::clean_tag() + release::clean_version_string()
        │
        ├── [download]
        │     ├── request::get_bytes()  → binary download
        │     ├── sha2::Sha256          → hash computation
        │     └── std::fs::write        → save to exe path
        │
        ├── [hash comparison]
        │     └── release::hash_local_file() + downloaded hash
        │
        └── release::print_result()  → formatted output
```

## Configuration

Configuration sources, in priority order:

1. CLI flags (highest)
2. `.env` file (loaded via `dotenvy::dotenv()`)
3. Built-in defaults

| Variable | CLI flag | Default | Description |
| - | - | - | - |
| `REPO_URL` | `--repo` | `https://github.com/microsoft/winget-create` | GitHub repo to check |
| `TARGET_EXE` | `--target` | `wingetcreate.exe` | Asset filename within the release |
| `VERSION` | `--version` | `latest` | Release tag filter |
| `MODE` | `--mode` | `both` | `hash` / `version` / `both` |
| `EXE_PATH` | `--exe` | _(none)_ | Local exe path (dir or full) |
| `EXPECTED_HASH` | `--hash` | _(none)_ | SHA-256 for download integrity |

The `--exe` path is resolved by `resolve_exe_path()`: if it's a directory (trailing separator or existing dir), the `TARGET_EXE` filename is appended.

## Build and Release Packaging

Standard `cargo build --release`. No custom profile settings.

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
  - `CheckResult` — holds all comparison results and download metadata
  - `GitHubAsset` — deserialized from GitHub API: `name`, `browser_download_url`
  - `GitHubRelease` — deserialized from GitHub API: `tag_name`, `assets`
- **Public functions:**
  - `resolve_exe_path(base, asset_name) -> PathBuf` — resolves a path that may be a directory or a full file path
  - `parse_repo_url(url) -> Result<(String, String)>` — extracts owner/repo from a GitHub URL
  - `get_latest_release(repo_url) -> Result<GitHubRelease>` — fetches the latest release from GitHub API
  - `find_asset_url(release, target_exe) -> Option<&str>` — finds a release asset by name (case-insensitive)
  - `clean_tag(tag) -> &str` — strips leading `v`/`V` prefix
  - `download_and_hash(url, save_path) -> Result<String>` — downloads a file, returns its SHA-256 hex string, optionally saves to disk
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
  - **Hash comparison:** SHA-256 of local file vs SHA-256 of downloaded bytes. Always performs hash verification on any download (integrity check). Optional `--hash` provides an additional expected-hash verification.
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

### _(All public items are documented in their module sections above.)_
