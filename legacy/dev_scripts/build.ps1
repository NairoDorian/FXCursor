# FXCursor Build Script
# Usage:
#   .\build.ps1                     - Builds for native host in release mode
#   .\build.ps1 -Mode debug          - Builds for native host in debug mode
#   .\build.ps1 -Target win          - Builds for Windows in release mode
#   .\build.ps1 -Target win -Mode debug - Builds for Windows in debug mode

param (
    [string]$Target = "native",
    [string]$Mode = "release"
)

$ErrorActionPreference = "Stop"

Write-Host "=== Starting FXCursor Build Process ===" -ForegroundColor Cyan

# Configure cargo flags based on mode
$cargoFlags = @()
if ($Mode -eq "release") {
    $cargoFlags += "--release"
}

# Change directory to project folder
Push-Location "$PSScriptRoot/../project_cursor/src-tauri"

try {
    if ($Target -eq "native") {
        Write-Host "Building for native host in $Mode mode..." -ForegroundColor Yellow
        cargo build $cargoFlags
    }
    elseif ($Target -eq "win") {
        Write-Host "Building for Windows (x86_64-pc-windows-msvc) in $Mode mode..." -ForegroundColor Yellow
        rustup target add x86_64-pc-windows-msvc
        cargo build $cargoFlags --target x86_64-pc-windows-msvc
    }
    elseif ($Target -eq "linux") {
        Write-Host "Building for Linux (x86_64-unknown-linux-gnu) in $Mode mode..." -ForegroundColor Yellow
        Write-Host "Note: Cross-compiling to Linux from Windows requires a cross-linker (e.g. WSL, 'cross', or 'cargo-zigbuild')." -ForegroundColor Cyan
        rustup target add x86_64-unknown-linux-gnu
        cargo build $cargoFlags --target x86_64-unknown-linux-gnu
    }
    elseif ($Target -eq "mac") {
        Write-Host "Building for macOS (x86_64-apple-darwin) in $Mode mode..." -ForegroundColor Yellow
        Write-Host "Note: Cross-compiling to macOS from Windows requires a cross-linker or macOS SDK." -ForegroundColor Cyan
        rustup target add x86_64-apple-darwin
        cargo build $cargoFlags --target x86_64-apple-darwin
    }
    else {
        Write-Error "Invalid target '$Target'. Choose from: native, win, linux, mac"
    }

    Write-Host "=== Build Completed Successfully! ===" -ForegroundColor Green
}
finally {
    Pop-Location
}
