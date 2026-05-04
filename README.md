# git-release-updater

Deploy or update a specific executable from the latest (or a specific) release on popular Git hosting websites like GitHub.

## Prerequisites

- Rust toolchain (latest stable)
- Windows (current target)

## Building

```powershell
cargo build --release
```

## Configuration

| Variable | Description | Default |
| - | - | - |
| _(none yet)_ | | |

## Usage

```powershell
cargo run
```

CLI flags: _(none yet — skeleton stage)_

## Output

- **stdout:** Status messages, release info
- **Log files:** _(none yet)_
- **Reports:** _(none yet)_

## Architecture

```md
main (entry point) ──→ request (HTTP client)
                    ──→ util (helpers)
```

## Modules

| Module | Purpose |
| - | - |
| `main` | Entry point, async runtime init |
| `request` | HTTP request wrappers (GET/POST/PUT/DELETE) |
| `util` | Reusable utility functions |

## Tracked Software

### _(none yet — skeleton stage)_

## Developer Credits

Imthatguyhere (ITGH | Tyler)
