# cargo_build.ps1 - Build CursorFX binary
# Runs full verification first, then compiles.
# Usage:
#   .\cargo_build.ps1                         - Native release build (with pre-check)
#   .\cargo_build.ps1 -Mode debug             - Native debug build
#   .\cargo_build.ps1 -Target win             - Windows release build
#   .\cargo_build.ps1 -Target win -Mode debug - Windows debug build
#   .\cargo_build.ps1 -Target linux           - Linux release build
#   .\cargo_build.ps1 -Target mac             - macOS release build
#   .\cargo_build.ps1 -SkipCheck             - Skip pre-check (faster iteration)

param (
    [string]$Target    = "native",
    [string]$Mode      = "release",
    [switch]$SkipCheck = $false
)

$ErrorActionPreference = "Stop"

$ESC    = [char]27
$RESET  = "$ESC[0m"
$CYAN   = "$ESC[96m"
$YELLOW = "$ESC[93m"
$GREEN  = "$ESC[92m"
$RED    = "$ESC[91m"
$BOLD   = "$ESC[1m"

function Write-Step { param($msg) Write-Host "${BOLD}${CYAN}>>> $msg${RESET}" }
function Write-Ok   { param($msg) Write-Host "${GREEN}[OK]  $msg${RESET}" }

Write-Host ""
Write-Host "${BOLD}${CYAN}=== CursorFX Build ===${RESET}"
Write-Host "Target : ${BOLD}$Target${RESET}"
Write-Host "Mode   : ${BOLD}$Mode${RESET}"
Write-Host ""

Push-Location "$PSScriptRoot/../project_cursor/src-tauri"
$buildPassed = $true

try {
    # -- Pre-build verification ------------------------------------------------
    if (-not $SkipCheck) {
        Write-Step "Pre-build - cargo check --all-targets"
        cargo check --all-targets
        if ($LASTEXITCODE -ne 0) {
            $buildPassed = $false
            Write-Host "${RED}[FAIL] cargo check failed - fix errors before building.${RESET}"
        } else {
            Write-Ok "cargo check passed"
        }

        if ($buildPassed) {
            Write-Host ""
            Write-Step "Pre-build - cargo clippy -W clippy::all"
            cargo clippy -- -W clippy::all
            if ($LASTEXITCODE -ne 0) {
                $buildPassed = $false
                Write-Host "${RED}[FAIL] clippy warnings found - fix before building.${RESET}"
            } else {
                Write-Ok "clippy passed"
            }
        }
        Write-Host ""
    } else {
        Write-Host "${YELLOW}[SkipCheck] Skipping pre-build verification.${RESET}"
        Write-Host ""
    }

    # -- Compile ---------------------------------------------------------------
    if ($buildPassed) {
        $cargoFlags = @()
        if ($Mode -eq "release") { $cargoFlags += "--release" }

        switch ($Target) {
            "native" {
                Write-Step "Compiling for native host in $Mode mode..."
                cargo build $cargoFlags
            }
            "win" {
                Write-Step "Compiling for Windows (x86_64-pc-windows-msvc) in $Mode mode..."
                rustup target add x86_64-pc-windows-msvc
                cargo build $cargoFlags --target x86_64-pc-windows-msvc
            }
            "linux" {
                Write-Step "Compiling for Linux (x86_64-unknown-linux-gnu) in $Mode mode..."
                Write-Host "${YELLOW}Note: Requires a Linux cross-linker (WSL, 'cross', or 'cargo-zigbuild').${RESET}"
                rustup target add x86_64-unknown-linux-gnu
                cargo build $cargoFlags --target x86_64-unknown-linux-gnu
            }
            "mac" {
                Write-Step "Compiling for macOS (x86_64-apple-darwin) in $Mode mode..."
                Write-Host "${YELLOW}Note: Requires macOS SDK or 'cargo-zigbuild'.${RESET}"
                rustup target add x86_64-apple-darwin
                cargo build $cargoFlags --target x86_64-apple-darwin
            }
            default {
                Write-Host "${RED}Invalid target '$Target'. Choose from: native, win, linux, mac${RESET}"
                $buildPassed = $false
            }
        }

        if ($LASTEXITCODE -ne 0) { $buildPassed = $false }
    }
} finally {
    Pop-Location
}

# -- Final result --------------------------------------------------------------
Write-Host ""
if ($buildPassed) {
    Write-Host "${BOLD}${GREEN}=== Build Completed Successfully! ===${RESET}"
    $profileDir = if ($Mode -eq "release") { "release" } else { "debug" }
    $outDir = Join-Path $PSScriptRoot "project_cursor\target\$profileDir"
    Write-Host "Binary : ${BOLD}$outDir\project_cursor.exe${RESET}"
    Write-Host ""
    exit 0
} else {
    Write-Host "${BOLD}${RED}=== Build FAILED - see above ===${RESET}"
    Write-Host ""
    exit 1
}
