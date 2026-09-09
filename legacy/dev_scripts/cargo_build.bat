@echo off
setlocal enabledelayedexpansion

:: cargo_build.bat — Build FXCursor binary
:: Runs full verification first, then compiles.
:: Usage:
::   cargo_build.bat                  - Native release build
::   cargo_build.bat debug            - Native debug build
::   cargo_build.bat win              - Windows release build
::   cargo_build.bat win debug        - Windows debug build
::   cargo_build.bat linux            - Linux release build
::   cargo_build.bat mac              - macOS release build
::   cargo_build.bat skipcheck        - Skip pre-build checks (faster iteration)
::   cargo_build.bat skipcheck debug  - Skip checks, debug build

set TARGET=native
set MODE=release
set CARGO_FLAGS=--release
set SKIP_CHECK=0

:: Parse args
if "%1"=="skipcheck" (
    set SKIP_CHECK=1
    if "%2"=="debug" (
        set MODE=debug
        set CARGO_FLAGS=
    )
    goto :build
)
if "%1"=="debug" (
    set MODE=debug
    set CARGO_FLAGS=
    goto :build
)
if not "%1"=="" (
    set TARGET=%1
    if "%2"=="debug" (
        set MODE=debug
        set CARGO_FLAGS=
    )
)

:build
echo.
echo === FXCursor Build ===
echo Target : !TARGET!
echo Mode   : !MODE!
echo.

cd %~dp0\..\project_cursor\src-tauri

:: ── Pre-build verification ─────────────────────────────────────────────────
if "!SKIP_CHECK!"=="0" (
    echo ^>^>^> Pre-build — cargo check --all-targets
    cargo check --all-targets
    if !ERRORLEVEL! NEQ 0 (
        echo [FAIL] cargo check failed — fix errors before building.
        cd %~dp0
        exit /b 1
    )
    echo [OK]  cargo check passed

    echo.
    echo ^>^>^> Pre-build — cargo clippy -W clippy::all
    cargo clippy -- -W clippy::all
    if !ERRORLEVEL! NEQ 0 (
        echo [FAIL] clippy warnings — fix before building.
        cd %~dp0
        exit /b 1
    )
    echo [OK]  clippy passed
    echo.
) else (
    echo [SkipCheck] Skipping pre-build verification.
    echo.
)

:: ── Compile ───────────────────────────────────────────────────────────────
if "!TARGET!"=="native" (
    echo ^>^>^> Compiling for native host in !MODE! mode...
    cargo build !CARGO_FLAGS!
) else if "!TARGET!"=="win" (
    echo ^>^>^> Compiling for Windows in !MODE! mode...
    rustup target add x86_64-pc-windows-msvc
    cargo build !CARGO_FLAGS! --target x86_64-pc-windows-msvc
) else if "!TARGET!"=="linux" (
    echo ^>^>^> Compiling for Linux in !MODE! mode...
    echo Note: Requires a Linux cross-linker.
    rustup target add x86_64-unknown-linux-gnu
    cargo build !CARGO_FLAGS! --target x86_64-unknown-linux-gnu
) else if "!TARGET!"=="mac" (
    echo ^>^>^> Compiling for macOS in !MODE! mode...
    echo Note: Requires macOS SDK or cargo-zigbuild.
    rustup target add x86_64-apple-darwin
    cargo build !CARGO_FLAGS! --target x86_64-apple-darwin
) else (
    echo Invalid target "!TARGET!". Choose from: native, win, linux, mac
    cd %~dp0
    exit /b 1
)

if !ERRORLEVEL! NEQ 0 (
    echo [FAIL] Build failed — see errors above.
    cd %~dp0
    exit /b 1
)

cd %~dp0
echo.
echo === Build Completed Successfully! ===
echo Binary : project_cursor\target\!MODE!\project_cursor.exe
echo.
