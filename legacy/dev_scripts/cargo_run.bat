@echo off
setlocal enabledelayedexpansion

:: cargo_run.bat — Run FXCursor
:: Usage:
::   cargo_run.bat              - Run debug build (console visible, logs printed)
::   cargo_run.bat release      - Run optimised release binary
::   cargo_run.bat release info - Run release with specific log level (trace/debug/info/warn/error)

set MODE=debug
set CARGO_FLAGS=
set RUST_LOG=info

if "%1"=="release" (
    set MODE=release
    set CARGO_FLAGS=--release
)
if not "%2"=="" set RUST_LOG=%2

echo.
echo === FXCursor - Launching ===
echo Mode  : !MODE!
echo Log   : RUST_LOG=!RUST_LOG!
echo.

cd %~dp0\..\project_cursor\src-tauri

set RUST_LOG=!RUST_LOG!
cargo run !CARGO_FLAGS!

cd %~dp0
echo.
echo === FXCursor exited ===
