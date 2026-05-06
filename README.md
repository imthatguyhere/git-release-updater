# git-release-updater

Check GitHub release versions and file hashes. Downloads and verifies release assets, comparing against local executables to determine if an update is needed.

## Prerequisites

- Rust toolchain (latest stable)
- Windows (required for PE version extraction; hash mode works cross-platform)

## Building

```powershell
cargo build --release
```

### Repo-root build entrypoint

Run the release build from the repo root with:

```powershell
.\scripts\build-release.ps1
```

That script builds the release binary and copies it into `dist/`.

### VS Code build task

VS Code uses the default build task named `Build release script` in [.vscode/tasks.json](.vscode/tasks.json).

Use `Terminal > Run Build Task...` or the built-in build command `Ctrl+Shift+B` when the workspace is open.

## Configuration

Configuration is resolved in priority order: **CLI arg > `.env` > default**. Windows paths in `.env` may be written unquoted, including directory paths ending in `\`.

| Variable | CLI flag | Description | Default |
| - | - | - | - |
| `REPO_URL` | `--repo` | GitHub repository URL | `https://github.com/microsoft/winget-create` |
| `TARGET_EXE` | `--target` | Executable name in release assets | `wingetcreate.exe` |
| `VERSION` | `--version` | Release version tag to check (`latest` or tag) | `latest` |
| `MODE` | `--mode` | Check mode: `hash`, `version`, or `both` | `both` |
| `EXE_PATH` | `--exe` | Path to local executable (dir or full path) | `C:\ProgramData\ITGH\git-release-updater` |
| `OUTPUT_PATH` | `--output` | Where to save downloaded file (dir or full path). Falls back to `EXE_PATH` | _(uses EXE_PATH)_ |
| `EXPECTED_HASH` | `--hash` | Expected SHA-256 hash — fails fatally if mismatch. Falls back to GitHub asset `digest` if available. | _(none — uses GitHub digest)_ |

> **Hash verification priority:** GitHub release asset `digest` field (if present) → `--hash` CLI arg. If neither is available, the download is **not saved** — requires `--hash` to proceed.

## Usage

```powershell
# Check defaults (microsoft/winget-create, wingetcreate.exe)
cargo run

# Version mode — compare local exe version vs release tag
cargo run -- --mode version --exe "C:\tools\myapp.exe" --output "C:\tools\myapp.exe"

# Hash mode — compare local hash vs downloaded hash, save only if different
cargo run -- --mode hash --exe "C:\tools\myapp.exe" --output "C:\tools\myapp.exe"

# Both modes — version check first, hash compare if different
cargo run -- --mode both --exe "C:\tools\wingetcreate.exe"

# With expected hash integrity check (fatal if mismatch, download discarded)
cargo run -- --hash "sha256:9f56bb326b852a699296e936c7b40dadfaf3ccff01c8e84ecff89871ecff8e5c"

# Using a directory path for the exe (appends target filename)
cargo run -- --exe "C:\Programs\MyApp\"
# resolves to: C:\Programs\MyApp\wingetcreate.exe

# Extensionless paths are treated as directories too
cargo run -- --exe "C:\Programs\MyApp"
# resolves to: C:\Programs\MyApp\wingetcreate.exe

# Separate output path from exe path
cargo run -- --exe "C:\tools\current.exe" --output "C:\tools\updated.exe"

# Specific release tag instead of latest
cargo run -- --version "v1.10.3.0" --exe "C:\tools\myapp.exe"

# Custom repo + target
cargo run -- --repo "https://github.com/my-org/my-tool" --target "mytool.exe"
```

### Modes

| Mode | Behavior |
| - | - |
| `version` | Checks local exe's FileVersion/ProductVersion against the release tag. Downloads only if version differs. Saves to output path. |
| `hash` | Hashes local exe, downloads release to memory, compares hashes. Saves to output path **only if hashes differ**. Skips save if already current. |
| `both` | Checks version and hash. If the local hash already matches the GitHub digest or `--hash`, skips download. Otherwise downloads, verifies integrity, and saves only if the downloaded hash differs from the local file. |

> **All modes** verify download integrity against GitHub asset `digest` or `--hash` before saving. Without either, the download is discarded and an error is returned.

### `.env` example

Copy `.env.example` to `.env` and adjust. Default configuration:

```env
REPO_URL=https://github.com/microsoft/winget-create
TARGET_EXE=wingetcreate.exe
VERSION=latest
MODE=both
EXE_PATH=C:\ProgramData\ITGH\git-release-updater
OUTPUT_PATH=C:\ProgramData\ITGH\git-release-updater
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
  GitHub digest:   ✓ MATCH
    hash: 8bd738851b524885410112678e3771b341c5c716de60fbbecb88ab0a363ed85d
  Download:        not performed (already current)
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
