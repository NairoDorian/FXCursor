@echo off
setlocal enabledelayedexpansion

echo === Starting CursorFX Build Process ===

cd %~dp0\..\project_cursor\src-tauri

:: Defaults
set MODE=release
set CARGO_FLAGS=--release
set TARGET=native

:: Check arguments
:: Usage: build.bat [target] [mode]
:: Examples:
::   build.bat           -> native release
::   build.bat debug     -> native debug
::   build.bat win       -> win release
::   build.bat win debug -> win debug

if "%1"=="debug" (
    set MODE=debug
    set CARGO_FLAGS=
    set TARGET=native
) else (
    if not "%1"=="" (
        set TARGET=%1
    )
    if "%2"=="debug" (
        set MODE=debug
        set CARGO_FLAGS=
    )
)

if "%TARGET%"=="win" (
    echo Building for Windows in !MODE! mode...
    rustup target add x86_64-pc-windows-msvc
    cargo build !CARGO_FLAGS! --target x86_64-pc-windows-msvc
) else if "%TARGET%"=="linux" (
    echo Building for Linux in !MODE! mode...
    echo Note: Cross-compiling to Linux from Windows requires a cross-linker (e.g. WSL, 'cross', or 'cargo-zigbuild').
    rustup target add x86_64-unknown-linux-gnu
    cargo build !CARGO_FLAGS! --target x86_64-unknown-linux-gnu
) else if "%TARGET%"=="mac" (
    echo Building for macOS in !MODE! mode...
    echo Note: Cross-compiling to macOS from Windows requires a cross-linker or macOS SDK.
    rustup target add x86_64-apple-darwin
    cargo build !CARGO_FLAGS! --target x86_64-apple-darwin
) else if "%TARGET%"=="native" (
    echo Building for native host in !MODE! mode...
    cargo build !CARGO_FLAGS!
) else (
    echo Invalid target "%TARGET%". Choose from: win, linux, mac, or leave empty for native.
    exit /b 1
)

cd %~dp0
echo === Build Completed Successfully! ===
