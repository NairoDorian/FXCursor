# Build and Update Instructions

> **Scope note (2026-09-09)**: the PowerShell/CMD scripts in this folder were written for the legacy V3 app in `project_cursor/` and still hard-code that path. The active V4 app lives in **``** and is driven entirely by Bun scripts (section 1). Treat sections 2–4 as legacy until the scripts are re-pointed.

---

## 1. V4 Build Pipeline (FXCursor — Bun + Tauri 2)

```powershell
cd FXCursor
bun install                 # dependencies (never npm/yarn/pnpm)
bun run tauri dev           # Studio window + transparent overlay (primary test command)
bun run typecheck           # tsc -b
bun run lint                # oxlint
bun test                    # bun unit tests
bun run build               # Vite production bundle → dist/
rtk cargo check --workspace
rtk cargo test --workspace
bun run before-commit       # 7-gate validation (typecheck, lint, tests, build, cargo check/test, version sync)
bun run arch                # regenerate ARCHITECTURE.md
bun run update-deps         # dual-ecosystem pre-release upgrade + full verification
bun run tauri build         # release binary + NSIS installer (target/release/bundle/nsis)
bun run package:portable    # portable zip next to the installer (exe + portable marker + Data/)
```

- Vite dev server: `http://localhost:1420` (HMR for the Studio only; Rust changes trigger a cargo rebuild).
- Debug builds keep the console for `env_logger` output (`RUST_LOG=info` recommended); release builds use `windows_subsystem = "windows"`.
- Configuration is written to `%APPDATA%\com.fxcursor.app\config.json` (Windows) or `<exe dir>\Data\config.json` when a file named `portable` sits next to the executable.
- Launch with `--minimized` to start hidden in the tray (this is what autostart uses).

---

## 2. Legacy Rust Build Scripts (dev_scripts/, target `project_cursor/`)

| Script                                            | Purpose                                 | Arguments                                            |
| ------------------------------------------------- | --------------------------------------- | ---------------------------------------------------- |
| `build.ps1` / `build.bat`                         | Clean release/debug build               | `-Target native/win/linux/mac`, `-Mode release/debug` |
| `cargo_build.ps1` / `cargo_build.bat`             | Check + clippy + build                  | Targets: `native`, `win`, `linux`, `mac`             |
| `cargo_check.ps1` / `cargo_check.bat`             | Type check + clippy + docs              | N/A                                                  |
| `cargo_run.ps1` / `cargo_run.bat`                 | Cargo run shortcut                      | N/A                                                  |
| `update_dependencies.ps1` / `update_dependencies.bat` | Upgrade Cargo.toml deps to latest   | N/A                                                  |
| `generate_repomix.ps1` / `generate_repomix.bat`   | Generate AI-ready repo pack             | N/A                                                  |

To reuse them for V4, change their working directory from `project_cursor/src-tauri` to `FXCursor` (workspace root) — `cargo` commands then cover `src-tauri`, `fxcursor-protocol` and `fxcursor-daemon` together.

```powershell
.\dev_scripts\build.ps1                 # native release (legacy path)
.\dev_scripts\build.ps1 -Mode debug
.\dev_scripts\build.ps1 -Target linux
```

```cmd
dev_scripts\build.bat
dev_scripts\build.bat debug
```

---

## 3. Cross-Compilation Targets

By default, scripts invoke `rustup target add <target>` to install toolchains.

- **Windows** (`x86_64-pc-windows-msvc`): requires MSVC build tools.
- **Linux** (`x86_64-unknown-linux-gnu`): use WSL2, or `cargo install cross --locked && cross build --target x86_64-unknown-linux-gnu --release`.
- **macOS** (`x86_64-apple-darwin` / `aarch64-apple-darwin`): compile natively, or `cargo install cargo-zigbuild --locked && cargo zigbuild --target aarch64-apple-darwin --release`.

Tauri release bundles must be built on the target OS (`bun run tauri build`); cross builds only produce the Rust binary. The overlay and global-mouse paths are verified on Windows only (see `PROGRESS.md`).

---

## 4. Dependency Updates

### V4 (recommended)
```bash
cd FXCursor
bun run update-deps
```
Probes NPM pre-release dist-tags and Crates.io `newest_version`, rewrites `package.json` / `Cargo.toml`, runs `bun update --latest` and `cargo update`, then typecheck, build, `cargo check` and `cargo test`.

### Legacy
```powershell
.\dev_scripts\update_dependencies.ps1   # cargo upgrade --to-latest (needs cargo-edit) + cargo update, project_cursor only
bun update                               # project_cursor frontend
```

---

## 5. Windows 11 Notes

- Overlay click-through uses Tauri's `set_ignore_cursor_events(true)`; no WndProc subclassing.
- The overlay spans the full virtual desktop (`SM_XVIRTUALSCREEN`…), including monitors at negative coordinates.
- wgpu enumerates DX12 and Vulkan; the `HighPerformance` adapter wins (Vulkan on the NVIDIA dev machine). The V3-era NVAPI "Prefer Native" workaround was not carried over.
- Global hotkey conflicts (another app owns the shortcut) are logged with `RUST_LOG=info` and do not stop the app.
