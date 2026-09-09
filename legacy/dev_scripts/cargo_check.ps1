# cargo_check.ps1 - Full Verification Suite for FXCursor
# Runs all quality checks in sequence: type check, clippy lints, doc check.
# Usage:
#   .\cargo_check.ps1            - Full verification (default)
#   .\cargo_check.ps1 -Quick     - cargo check only (fastest, skips clippy)
#   .\cargo_check.ps1 -Fix       - Auto-apply safe clippy fixes

param (
    [switch]$Quick = $false,
    [switch]$Fix   = $false
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
function Write-Fail { param($msg) Write-Host "${RED}[FAIL] $msg${RESET}" }

Write-Host ""
Write-Host "${BOLD}${CYAN}=== FXCursor Verification Suite ===${RESET}"
Write-Host ""

Push-Location "$PSScriptRoot/../project_cursor/src-tauri"
$allPassed = $true

try {
    # -- Step 1: cargo check --------------------------------------------------
    Write-Step "Step 1/3 - cargo check --all-targets"
    cargo check --all-targets
    if ($LASTEXITCODE -ne 0) {
        $allPassed = $false
        Write-Fail "cargo check failed"
    } else {
        Write-Ok "cargo check passed"
    }
    Write-Host ""

    if ($Quick) {
        Write-Host "${YELLOW}[Quick mode] Skipping clippy and doc checks.${RESET}"
    } else {
        # -- Step 2: cargo clippy ---------------------------------------------
        Write-Step "Step 2/3 - cargo clippy -W clippy::all"
        if ($Fix) {
            Write-Host "${YELLOW}[Fix mode] Applying safe auto-fixes...${RESET}"
            cargo clippy --fix --allow-staged -- -W clippy::all
        } else {
            cargo clippy -- -W clippy::all
        }
        if ($LASTEXITCODE -ne 0) {
            $allPassed = $false
            Write-Fail "clippy found issues"
        } else {
            Write-Ok "clippy passed (0 warnings)"
        }
        Write-Host ""

        # -- Step 3: cargo doc ------------------------------------------------
        Write-Step "Step 3/3 - cargo doc --no-deps"
        cargo doc --no-deps --quiet
        if ($LASTEXITCODE -ne 0) {
            $allPassed = $false
            Write-Fail "cargo doc failed"
        } else {
            Write-Ok "cargo doc passed"
        }
        Write-Host ""
    }
} finally {
    Pop-Location
}

# -- Final result --------------------------------------------------------------
Write-Host ""
if ($allPassed) {
    Write-Host "${BOLD}${GREEN}=== All checks PASSED ===${RESET}"
    exit 0
} else {
    Write-Host "${BOLD}${RED}=== Some checks FAILED - see above ===${RESET}"
    exit 1
}
