@echo off
setlocal enabledelayedexpansion

:: cargo_check.bat — Full Verification Suite for CursorFX
:: Usage:
::   cargo_check.bat          - Full verification (check + clippy + doc)
::   cargo_check.bat quick    - cargo check only (fastest)
::   cargo_check.bat fix      - Auto-apply safe clippy fixes

set QUICK=0
set FIX=0
set ALL_PASSED=1

if "%1"=="quick" set QUICK=1
if "%1"=="fix"   set FIX=1

echo.
echo === CursorFX Verification Suite ===
echo.

cd %~dp0\project_cursor

:: ── Step 1: cargo check ──────────────────────────────────────────────────────
echo ^>^>^> Step 1/3 — cargo check --all-targets
cargo check --all-targets
if !ERRORLEVEL! NEQ 0 (
    echo [FAIL] cargo check failed
    set ALL_PASSED=0
) else (
    echo [OK]  cargo check passed
)
echo.

if "%QUICK%"=="1" (
    echo [Quick mode] Skipping clippy and doc checks.
    goto :done
)

:: ── Step 2: cargo clippy ─────────────────────────────────────────────────────
echo ^>^>^> Step 2/3 — cargo clippy -W clippy::all
if "%FIX%"=="1" (
    echo [Fix mode] Applying safe auto-fixes...
    cargo clippy --fix --allow-staged -- -W clippy::all
) else (
    cargo clippy -- -W clippy::all
)
if !ERRORLEVEL! NEQ 0 (
    echo [FAIL] clippy found issues
    set ALL_PASSED=0
) else (
    echo [OK]  clippy passed ^(0 warnings^)
)
echo.

:: ── Step 3: cargo doc ────────────────────────────────────────────────────────
echo ^>^>^> Step 3/3 — cargo doc --no-deps
cargo doc --no-deps --quiet
if !ERRORLEVEL! NEQ 0 (
    echo [FAIL] cargo doc failed
    set ALL_PASSED=0
) else (
    echo [OK]  cargo doc passed
)
echo.

:done
cd %~dp0
echo.
if "!ALL_PASSED!"=="1" (
    echo === All checks PASSED ===
) else (
    echo === Some checks FAILED — see above ===
    exit /b 1
)
