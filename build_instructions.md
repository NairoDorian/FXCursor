# Build and Update Instructions

This repository contains scripts to automate building, testing, and updating dependencies for the Cross-Platform Rust WebGPU Cursor FX application.

## 1. Build Scripts (`build.ps1` and `build.bat`)

The build scripts automate native compilation and target configuration. They support both **Development/Debug** (keeps debug console windows, includes symbols) and **Production/Release** (fastest execution speed, hides debug consoles) modes.

### Usage in PowerShell:
```powershell
# Build for native host in release mode (default)
.\build.ps1

# Build for native host in debug mode (development)
.\build.ps1 -Mode debug

# Target Windows in release mode
.\build.ps1 -Target win

# Target Windows in debug mode
.\build.ps1 -Target win -Mode debug

# Target Linux in release mode
.\build.ps1 -Target linux
```

### Usage in Command Prompt (CMD):
```cmd
:: Build for native host in release mode
build.bat

:: Build for native host in debug mode
build.bat debug

:: Target Windows in release mode
build.bat win

:: Target Windows in debug mode
build.bat win debug
```

---

## 2. Windows 11 Shell Visibility Behavior (Debug vs. Release)

The project leverages conditional compilation tags to handle debug console outputs dynamically:
- **Debug Build**: Spawns a standard Windows console. You will see real-time logger outputs (`INFO`, `WARN`, etc.) in the terminal.
- **Release Build**: Completely hides the console window (`windows_subsystem = "windows"`). The program runs entirely in the background and is only visible in the system tray and overlay.

---

## 3. Cross-Compilation Target Setup

By default, the scripts will invoke `rustup target add <target>` to install target toolchains.

> [!NOTE]
> Cross-compiling from a Windows host to other operating systems requires installing target-specific linkers and SDKs.

### A. Windows Target (`x86_64-pc-windows-msvc`)
- Requires MSVC build tools (installed automatically with Visual Studio or Rust's Windows build tools).

### B. Linux Target (`x86_64-unknown-linux-gnu`)
- Requires a GCC compiler and linker targeting Linux.
- **Recommended Setup**:
  1. Use Windows Subsystem for Linux (WSL) to compile natively inside a Linux environment.
  2. Or use the `cross` crate (a docker-based toolchain wrapper):
     ```bash
     cargo install cross --locked
     cross build --target x86_64-unknown-linux-gnu --release
     ```

### C. macOS Target (`x86_64-apple-darwin` / `aarch64-apple-darwin`)
- Requires Xcode SDK and macOS linker toolchains.
- **Recommended Setup**:
  - Compile natively on a macOS machine.
  - Or use a cross-compiler toolchain like `cargo-zigbuild` with Zig as the linker:
    ```bash
    cargo install cargo-zigbuild --locked
    cargo zigbuild --target x86_64-apple-darwin --release
    ```

---

## 4. Dependencies Update Scripts (`update_dependencies.ps1` and `update_dependencies.bat`)

The updater scripts dynamically fetch the latest available versions of all crates in `Cargo.toml` without hardcoding constraint versions.

### How it works:
1. Verifies if `cargo-edit` (which provides `cargo-upgrade`) is installed. If missing, it installs it automatically via `cargo install cargo-edit --locked`.
2. Runs `cargo upgrade --to-latest` which modifies `Cargo.toml` to bump all dependencies to their newest versions.
3. Runs `cargo update` to update `Cargo.lock`.

### Usage:
```powershell
.\update_dependencies.ps1
```
```cmd
update_dependencies.bat
```
