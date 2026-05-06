param(
    [string]$Configuration = "release",
    [string]$OutputDirectory = "dist",
    [switch]$NoUPX
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$targetExe = Join-Path $repoRoot "target\$Configuration\git-release-updater.exe"
$distDir = Join-Path $repoRoot $OutputDirectory
$distExe = Join-Path $distDir "git-release-updater.exe"

Push-Location $repoRoot
try {
    cargo build --release

    New-Item -ItemType Directory -Force -Path $distDir | Out-Null
    Copy-Item -LiteralPath $targetExe -Destination $distExe -Force

    if (-not $NoUPX) {
        $upx = Get-Command upx -ErrorAction SilentlyContinue
        if ($null -eq $upx) {
            Write-Warning "UPX not found on PATH; leaving executable uncompressed."
        }
        else {
            & $upx.Source --lzma --best --all-filters $distExe
            if ($LASTEXITCODE -ne 0) {
                Write-Warning "UPX compression failed with exit code $LASTEXITCODE; leaving executable uncompressed."
                Copy-Item -LiteralPath $targetExe -Destination $distExe -Force
            }
        }
    }

    $size = (Get-Item -LiteralPath $distExe).Length
    Write-Host "Release executable written to $distExe ($([Math]::Round($size / 1MB, 2)) MB)"
}
finally {
    Pop-Location
}
