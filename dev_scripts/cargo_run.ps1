# cargo_run.ps1 - Run CursorFX
# Usage:
#   .\cargo_run.ps1               - Run in debug mode (console visible, logs printed)
#   .\cargo_run.ps1 -Release      - Run optimised release binary
#   .\cargo_run.ps1 -Log trace    - Set log level (trace|debug|info|warn|error)

param (
    [switch]$Release = $false,
    [string]$Log     = "info"
)

$ErrorActionPreference = "Stop"

$ESC   = [char]27
$RESET = "$ESC[0m"
$CYAN  = "$ESC[96m"
$GREEN = "$ESC[92m"
$BOLD  = "$ESC[1m"

$mode = if ($Release) { "release" } else { "debug" }

Write-Host ""
Write-Host "${BOLD}${CYAN}=== CursorFX - Launching ===${RESET}"
Write-Host "Mode  : ${BOLD}$mode${RESET}"
Write-Host "Log   : ${BOLD}RUST_LOG=$Log${RESET}"
Write-Host ""

$env:RUST_LOG = $Log

Push-Location "$PSScriptRoot/project_cursor"

try {
    if ($Release) {
        cargo run --release
    } else {
        cargo run
    }

    Write-Host ""
    Write-Host "${BOLD}${GREEN}=== CursorFX exited cleanly ===${RESET}"
} finally {
    Remove-Item Env:RUST_LOG -ErrorAction SilentlyContinue
    Pop-Location
}
