@echo off
setlocal enabledelayedexpansion

:: generate_repomix.bat — Automates repository summary generation and packs with Repomix.
:: Runs the PowerShell generator script.

echo.
echo === Launcher: Repomix Metadata Generation Workflow ===
echo.

powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0\generate_repomix.ps1"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [FAIL] Workflow failed. See errors above.
    exit /b 1
)

echo.
echo === Workflow Completed Successfully! ===
echo.
exit /b 0
