use clap::Parser;
use git_release_updater::release;
use std::path::PathBuf;

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
    //=-- Load .env — ignore if missing
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    //=-- Resolve values: CLI arg > .env > default
    let repo = cli
        .repo
        .unwrap_or_else(|| "https://github.com/microsoft/winget-create".into());
    let target = cli.target.unwrap_or_else(|| "wingetcreate.exe".into());
    let version = cli.version.unwrap_or_else(|| "latest".into());
    let mode_str = cli.mode.unwrap_or_else(|| "both".into());
    let expected_hash = cli.hash;

    //=-- Parse mode
    let mode = match release::CheckMode::from_str(&mode_str) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    };

    //=-- Resolve exe path (directory → dir + target filename)
    let local_exe = cli.exe.map(|p| release::resolve_exe_path(&p, &target));

    //=-- Resolve output path (directory → dir + target filename)
    let output_path = cli.output.map(|p| release::resolve_exe_path(&p, &target));

    println!("Repository:  {repo}");
    println!("Target exe:  {target}");
    println!("Version:     {version}");
    println!("Mode:        {mode_str}");
    if let Some(ref exe) = local_exe {
        println!("Local exe:   {}", exe.display());
    }
    if let Some(ref op) = output_path {
        println!("Output path: {}", op.display());
    }
    if let Some(ref h) = expected_hash {
        println!("Expected hash: {h}");
    }
    println!();

    match release::run_check(
        &repo,
        &target,
        &version,
        mode,
        local_exe.as_deref(),
        expected_hash.as_deref(),
        output_path.as_deref(),
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
