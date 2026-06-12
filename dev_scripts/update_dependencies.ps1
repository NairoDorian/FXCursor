# Cross_Platform_Rust_WebGPU_CursorFX Dependencies Update Script
# Updates all cargo dependencies to the latest possible version.

$ErrorActionPreference = "Stop"

Write-Host "=== Starting Dependencies Update Process ===" -ForegroundColor Cyan

# Ensure cargo-edit is installed for cargo upgrade command
if (-not (Get-Command "cargo-upgrade" -ErrorAction SilentlyContinue)) {
    Write-Host "cargo-edit not found. Installing cargo-edit (this might take a minute)..." -ForegroundColor Yellow
    cargo install cargo-edit --locked
}

# Navigate to project_cursor
Push-Location "$PSScriptRoot/../project_cursor/src-tauri"

try {
    Write-Host "Upgrading all dependencies in Cargo.toml to latest compatible versions..." -ForegroundColor Yellow
    cargo upgrade --compatible
    
    Write-Host "Running cargo update to update Cargo.lock..." -ForegroundColor Yellow
    cargo update
    
    Write-Host "=== Dependencies Update Completed Successfully! ===" -ForegroundColor Green
}
finally {
    Pop-Location
}
