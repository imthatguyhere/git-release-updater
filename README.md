# git-release-updater

Check GitHub release versions and file hashes. Downloads and verifies release assets, comparing against local executables to determine if an update is needed.

## Prerequisites

- Rust toolchain (latest stable)
- Windows (required for PE version extraction; hash mode works cross-platform)

## Building

```powershell
cargo build --release
```

## Configuration

Configuration is resolved in priority order: **CLI arg > `.env` > default**.

| Variable | CLI flag | Description | Default |
| - | - | - | - |
| `REPO_URL` | `--repo` | GitHub repository URL | `https://github.com/microsoft/winget-create` |
| `TARGET_EXE` | `--target` | Executable name in release assets | `wingetcreate.exe` |
| `VERSION` | `--version` | Release version tag to check (`latest` or tag) | `latest` |
| `MODE` | `--mode` | Check mode: `hash`, `version`, or `both` | `both` |
| `EXE_PATH` | `--exe` | Path to local executable (dir or full path) | _(none — download only)_ |
| `EXPECTED_HASH` | `--hash` | Expected SHA-256 hash for download verification | _(none — no extra check)_ |

## Usage

```powershell
# Check defaults (microsoft/winget-create, wingetcreate.exe)
cargo run

# Version mode — compare local exe version vs release tag
cargo run -- --mode version --exe "C:\tools\myapp.exe"

# Hash mode — compare local hash vs downloaded hash
cargo run -- --mode hash --exe "C:\tools\myapp.exe"

# Both modes — version check first, hash compare if different
cargo run -- --mode both --exe "C:\tools\wingetcreate.exe"

# With expected hash integrity check
cargo run -- --hash "sha256:9f56bb326b852a699296e936c7b40dadfaf3ccff01c8e84ecff89871ecff8e5c"

# Using a directory path for the exe (appends target filename)
cargo run -- --exe "C:\Programs\MyApp\"
# resolves to: C:\Programs\MyApp\wingetcreate.exe

# Custom repo + target
cargo run -- --repo "https://github.com/my-org/my-tool" --target "mytool.exe"
```

### Modes

| Mode | Behavior |
| - | - |
| `version` | Checks local exe's FileVersion/ProductVersion against the release tag. Downloads only if version differs. |
| `hash` | Downloads the release asset, computes SHA-256, compares against local exe hash. Always downloads. |
| `both` | Version check first (cheap). Downloads + hash compare only if version mismatches. |

### `.env` example

```env
REPO_URL=https://github.com/my-org/my-tool
TARGET_EXE=mytool.exe
MODE=version
EXE_PATH=C:\tools\mytool.exe
```

## Output

- **stdout:** Repository info, release tag, version/hash check results, download status, summary line.
- **Exit code:** `0` on success, `1` on fatal error (API failure, file I/O error, etc.).
- **Reports:** Printed to stdout as a formatted table.
- **Log files:** None (output is console-only).

### Example output

```md
Repository:  https://github.com/microsoft/winget-create
Target exe:  wingetcreate.exe
Version:     latest
Mode:        both
Local exe:   C:\tools\wingetcreate.exe

Latest release: 1.12.8.0  (tag: v1.12.8.0)

═══════════════════════════════════════
  Release Check Results
═══════════════════════════════════════
  Release tag:     v1.12.8.0
  Version check:   ✓ MATCH
    local:   1.12.8.0
    release: 1.12.8.0
  Download:        ✗ not performed (already current)
═══════════════════════════════════════
  Result: ✓ up-to-date
═══════════════════════════════════════
```

## Architecture

```md
lib (crate root — defines all modules)
  ├── release (check orchestration)
  │       ├── GitHub API helpers
  │       ├── download + SHA-256 hashing
  │       └── PE version extraction (Windows)
  ├── request (HTTP client)
  └── util (helpers, timestamps)

main (thin binary — imports from lib)
  └── calls lib modules, parses CLI, prints results
```

## Modules

| Module | Crate | Purpose |
| - | - | - |
| `lib` | library | Crate root. Re-exports `release`, `request`, `util` as public API. |
| `main` | binary | Entry point, `.env` loading, CLI parsing. Thin wrapper over the library. |
| `release` | library | GitHub API types, check modes, version/hash logic, download, file save |
| `request` | library | HTTP client wrapper (raw GET, JSON GET, binary GET) |
| `util` | library | Reusable utility functions (timestamps, string helpers, date math) |

## Tracked Software

### Default: `wingetcreate.exe` (microsoft/winget-create)

- **Repo:** `https://github.com/microsoft/winget-create`
- **Target:** `wingetcreate.exe`
- **Checks:** Version + hash

## Developer Credits

Imthatguyhere (ITGH | Tyler)
