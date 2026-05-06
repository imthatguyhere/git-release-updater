use clap::Parser;
use git_release_updater::release;
use std::collections::HashMap;
use std::path::PathBuf;

const DEFAULT_EXE_PATH: &str = r"C:\ProgramData\ITGH\git-release-updater";

fn read_dotenv_lossy() -> HashMap<String, String> {
    let mut values = HashMap::new();
    let Ok(mut dir) = std::env::current_dir() else {
        return values;
    };

    loop {
        let path = dir.join(".env");
        if let Ok(contents) = std::fs::read_to_string(&path) {
            for line in contents.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                let Some((key, value)) = line.split_once('=') else {
                    continue;
                };
                let key = key.trim().strip_prefix("export ").unwrap_or(key.trim());
                if key.is_empty() {
                    continue;
                }

                let value = value.trim();
                let value = value
                    .strip_prefix('"')
                    .and_then(|v| v.strip_suffix('"'))
                    .or_else(|| value.strip_prefix('\'').and_then(|v| v.strip_suffix('\'')))
                    .unwrap_or(value);
                values
                    .entry(key.to_string())
                    .or_insert_with(|| value.to_string());
            }
            return values;
        }

        if !dir.pop() {
            return values;
        }
    }
}

// ---------------------------------------------------------------------------
//=-- CLI
// ---------------------------------------------------------------------------

/// Check GitHub release versions and file hashes.
///
/// Loads configuration from a .env file (if present) and CLI arguments.
/// CLI arguments take precedence over .env values.
#[derive(Parser, Debug)]
#[command(name = "git-release-updater", disable_version_flag = true)]
struct Cli {
    /// GitHub repository URL
    ///   (default: https://github.com/microsoft/winget-create)
    #[arg(long, env = "REPO_URL")]
    repo: Option<String>,

    /// Target executable name in the release assets
    ///   (default: wingetcreate.exe)
    #[arg(long, env = "TARGET_EXE")]
    target: Option<String>,

    /// Release version tag to check — use "latest" for the latest release
    ///   (default: latest)
    #[arg(long, env = "VERSION")]
    version: Option<String>,

    /// Check mode: hash, version, or both
    ///   (default: both)
    #[arg(long, env = "MODE")]
    mode: Option<String>,

    /// Path to the local executable to inspect
    #[arg(long, env = "EXE_PATH")]
    exe: Option<PathBuf>,

    /// Path where to save the downloaded file (dir or full path).
    /// Falls back to EXE_PATH if not set.
    #[arg(long, env = "OUTPUT_PATH")]
    output: Option<PathBuf>,

    /// Expected SHA-256 hash for the release file
    ///   (fails fatally if mismatch)
    #[arg(long, env = "EXPECTED_HASH")]
    hash: Option<String>,
}

// ---------------------------------------------------------------------------
//=-- Entry point
// ---------------------------------------------------------------------------

#[tokio::main]
async fn main() {
    let dotenv_values = read_dotenv_lossy();

    //=-- Load .env — ignore if missing
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    //=-- Resolve values: CLI arg > .env > default
    let repo = cli
        .repo
        .or_else(|| dotenv_values.get("REPO_URL").cloned())
        .unwrap_or_else(|| "https://github.com/microsoft/winget-create".into());
    let target = cli
        .target
        .or_else(|| dotenv_values.get("TARGET_EXE").cloned())
        .unwrap_or_else(|| "wingetcreate.exe".into());
    let version = cli
        .version
        .or_else(|| dotenv_values.get("VERSION").cloned())
        .unwrap_or_else(|| "latest".into());
    let mode_str = cli
        .mode
        .or_else(|| dotenv_values.get("MODE").cloned())
        .unwrap_or_else(|| "both".into());
    let expected_hash = cli
        .hash
        .or_else(|| dotenv_values.get("EXPECTED_HASH").cloned());

    //=-- Parse mode
    let mode = match release::CheckMode::from_str(&mode_str) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    //=-- Resolve exe path (directory -> dir + target filename)
    let local_exe_base = cli
        .exe
        .or_else(|| dotenv_values.get("EXE_PATH").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from(DEFAULT_EXE_PATH));
    let local_exe = release::resolve_exe_path(&local_exe_base, &target);

    //=-- Resolve output path (directory -> dir + target filename)
    let output_path = cli
        .output
        .or_else(|| dotenv_values.get("OUTPUT_PATH").map(PathBuf::from))
        .map(|p| release::resolve_exe_path(&p, &target))
        .unwrap_or_else(|| local_exe.clone());

    println!("Repository:  {repo}");
    println!("Target exe:  {target}");
    println!("Version:     {version}");
    println!("Mode:        {mode_str}");
    println!("Local exe:   {}", local_exe.display());
    println!("Output path: {}", output_path.display());
    if let Some(ref h) = expected_hash {
        println!("Expected hash: {h}");
    }
    println!();

    match release::run_check(
        &repo,
        &target,
        &version,
        mode,
        Some(local_exe.as_path()),
        expected_hash.as_deref(),
        Some(output_path.as_path()),
    )
    .await
    {
        Ok(result) => release::print_result(&result),
        Err(e) => {
            eprintln!("Fatal error: {e}");
            std::process::exit(1);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_read_dotenv_lossy_preserves_trailing_backslash() {
        let dir = std::env::temp_dir().join(format!(
            "gru_dotenv_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let previous_dir = std::env::current_dir().unwrap();
        std::fs::write(
            dir.join(".env"),
            "REPO_URL=https://github.com/PAG-IT/cdk-drive-updater\nEXE_PATH=C:\\kworking\\cdk-drive-updater\\\nOUTPUT_PATH=C:\\kworking\\cdk-drive-updater\\\n",
        )
        .unwrap();

        std::env::set_current_dir(&dir).unwrap();
        let values = super::read_dotenv_lossy();
        std::env::set_current_dir(previous_dir).unwrap();
        std::fs::remove_dir_all(&dir).unwrap();

        assert_eq!(
            values.get("EXE_PATH").map(String::as_str),
            Some(r"C:\kworking\cdk-drive-updater\")
        );
        assert_eq!(
            values.get("OUTPUT_PATH").map(String::as_str),
            Some(r"C:\kworking\cdk-drive-updater\")
        );
    }
}
