# Build and Update Instructions

This repository contains scripts to automate building, testing, and updating dependencies for the CursorFX application.

---

## 1. V3 Build Pipeline (Bun + Tauri V2)

### Development
```powershell
# Install Bun dependencies
bun install

# Start Vite dev server (frontend only)
bun run dev

# Start Tauri dev mode (frontend + Rust backend)
bun run tauri dev
```

### Production Build
```powershell
# TypeScript check + Vite build
bun run build

# Tauri release build (bundled binary)
bun run tauri build
```

The Tauri dev server runs Vite on `http://localhost:1420` with HMR enabled.

---

## 2. Rust Build Scripts (dev_scripts/)

The scripts in `dev_scripts/` provide pure-Rust build commands for the `src-tauri/` directory:

| Script | Purpose | Arguments |
|--------|---------|-----------|
| `build.ps1` / `build.bat` | Clean release/debug build | `-Target native/win/linux/mac`, `-Mode release/debug` |
| `cargo_build.ps1` / `cargo_build.bat` | Check + clippy + build | Targets: `native`, `win`, `linux`, `mac` |
| `cargo_check.ps1` / `cargo_check.bat` | Type check + clippy + docs | N/A |
| `cargo_run.ps1` / `cargo_run.bat` | Cargo run shortcut | N/A |
| `update_dependencies.ps1` / `update_dependencies.bat` | Upgrade Cargo.toml deps to latest | N/A |
| `generate_repomix.ps1` / `generate_repomix.bat` | Generate AI-ready repo pack | N/A |

### PowerShell Usage
```powershell
# Build for native host in release mode (default)
.\dev_scripts\build.ps1

# Build for native host in debug mode
.\dev_scripts\build.ps1 -Mode debug

# Target Linux in release mode
.\dev_scripts\build.ps1 -Target linux
```

### CMD Usage
```cmd
:: Build for native host
dev_scripts\build.bat

:: Build for native host in debug mode
dev_scripts\build.bat debug
```

---

## 3. Cross-Compilation Targets

By default, scripts invoke `rustup target add <target>` to install toolchains.

### A. Windows (`x86_64-pc-windows-msvc`)
- Requires MSVC build tools (Visual Studio or Rust's Windows build tools)

### B. Linux (`x86_64-unknown-linux-gnu`)
- Use WSL2 for native Linux compilation
- Or use the `cross` crate: `cargo install cross --locked && cross build --target x86_64-unknown-linux-gnu --release`

### C. macOS (`x86_64-apple-darwin` / `aarch64-apple-darwin`)
- Compile natively on macOS hardware
- Or use `cargo-zigbuild`: `cargo install cargo-zigbuild --locked && cargo zigbuild --target x86_64-apple-darwin --release`

---

## 4. Dependency Updates

### Rust (Cargo)
```powershell
.\dev_scripts\update_dependencies.ps1
```
This runs `cargo upgrade --to-latest` (requires `cargo-edit`) followed by `cargo update`.

### Bun (Frontend)
```bash
bun update
```
This updates all packages in `package.json` to their latest semver-compatible versions and regenerates `bun.lock`.

---

## 5. Windows 11 Notes

- **Release builds** use `windows_subsystem = "windows"` (no console window)
- **Debug builds** show the console for `env_logger` output
- Tauri V2 handles the subsystem attribute automatically via `tauri.conf.json`
- If running on NVIDIA GPU, the NVAPI fix in `src-tauri/src/overlay/mod.rs` programmatically sets Vulkan presentation mode to "Prefer Native" (requires admin privileges once)
