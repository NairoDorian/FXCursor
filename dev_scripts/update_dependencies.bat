@echo off
echo === Starting Dependencies Update Process ===

cd %~dp0\project_cursor

where cargo-upgrade >nul 2>nul
if %errorlevel% neq 0 (
    echo cargo-edit not found. Installing cargo-edit...
    cargo install cargo-edit --locked
)

echo Upgrading all dependencies in Cargo.toml to latest compatible versions...
cargo upgrade --compatible

echo Running cargo update...
cargo update

cd %~dp0
echo === Dependencies Update Completed Successfully! ===
