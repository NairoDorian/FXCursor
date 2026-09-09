# Architectural Memory & Decisions Log

This document tracks design decisions, hardware interactions, crate evaluations, and resource budgets for FXCursor. Updated 2026-09-09 after a full code audit of the V4 codebase (now the repository root).

---

## 1. Architectural History

| Version          | Shell        | Frontend             | Graphics            | Physics | RAM (measured/target)          | Notes                                                                                   |
| ---------------- | ------------ | -------------------- | ------------------- | ------- | ------------------------------ | --------------------------------------------------------------------------------------- |
| **V0** (mods)    | Windhawk     | none                 | GDI+, D3D11         | CPU     | in-process in explorer.exe     | Windows-only C++ reference for the 4-layer visual design (`legacy/original_mods/`)             |
| **V1** (Legacy)  | Tauri V1     | SolidJS + Bun        | GDI+, D3D11 DLL     | CPU     | ~150–250 MB                    | Windows-only, injected DLL                                                              |
| **V2** (Rust)    | winit 0.29   | egui 0.26            | wgpu 0.19           | CPU     | < 30 MB                        | Cross-platform, no webview                                                              |
| **V3**           | Tauri V2     | React 19 + TW4       | wgpu 29             | CPU     | ~80–120 MB                     | `legacy/project_cursor/`, frozen                                                               |
| **V4 (current)** | Tauri V2     | SolidJS 2 + native CSS | wgpu 30           | CPU     | ~330 MB RSS in debug (2 webviews); release target < 120 MB | ``. Overlay is a Tauri window + wgpu surface. Daemon is a prototype only. |
| **V4 (spec)**    | Rust daemon  | SolidJS 2 (transient) | wgpu 30            | GPU compute | < 12 MB daemon             | `docs/V4_ARCHITECTURE_SPECIFICATION.md`; Pillars 1, 2, 4, 6 (non-Windows), 7 not built  |

### Why V4 kept the Tauri single-process model for now

1. It already works end to end: overlay, rendering, tray, persistence, hotkey, autostart on Windows.
2. The daemon path needs a native overlay window, an IPC client in the Studio and a process supervisor before it delivers its RAM win; none exist yet.
3. Extracting a shared `fxcursor-render` crate first lets both paths use one renderer instead of the current ~900-line fork.

---

## 2. Resource & Performance Budgets (V4 targets, release build)

| Metric                   | Target             | Status 2026-09-09                                                         |
| ------------------------ | ------------------ | ------------------------------------------------------------------------- |
| RAM (total process)      | < 120 MB           | Not measured in release; debug ~330 MB because the overlay is a webview   |
| CPU (idle, cursor still) | < 0.5 %            | Loop wakes every 15 ms, does physics on 80 nodes, skips GPU work           |
| CPU (active)             | < 2 %              | 4 ms loop, CPU ribbon build for 4 layers                                  |
| GPU (idle)               | 0 %                | ✅ no submissions after 3 settle frames                                    |
| Frame pacing             | Mailbox, ≤ 2 frames latency | ✅                                                                |
| Input latency            | < 1 frame          | Polling (4 ms) — clicks shorter than a poll can be missed                 |
| Startup                  | < 1.0 s            | ~1 s dev; not measured release                                            |

---

## 3. Crate Evaluation & Decisions

### Windowing: Tauri 2 (`tao`)
- Overlay = `WebviewWindowBuilder` with `transparent`, `decorations(false)`, `always_on_top`, `skip_taskbar`, `shadow(false)`, `focused(false)`, then `set_ignore_cursor_events(true)` for click-through. No WndProc subclassing (V3's crash source).
- Cost: an extra WebView2 process for a window that never shows HTML. Decision: accept for now; revisit with a raw HWND/winit overlay (roadmap Phase C).

### Graphics: wgpu 30
- Backends `DX12 | VULKAN` on Windows (adapter choice left to `HighPerformance` preference; Vulkan was picked on the dev machine), Metal on macOS, Vulkan on Linux.
- Pre-multiplied alpha composite mode when available, `Mailbox` presentation, `desired_maximum_frame_latency: 2`.
- Surface loss/outdated → reconfigure in place.

### Input: `device_query` 4
- Chosen for simplicity and cross-platform coverage. Known limits: polling only, no sub-poll click detection, Wayland unsupported, macOS needs Accessibility permission.
- Decision: replace on Windows with Raw Input + `MsgWaitForMultipleObjectsEx` (roadmap Phase B); keep `device_query` as the fallback.

### Configuration: JSON + `serde_path_to_error` self-healing
- `fxcursor-protocol::deserialize_with_self_healing` merges missing keys from defaults and resets only broken fields (up to 64 repair rounds).
- Storage path: `app_config_dir()/config.json` or `<exe>/Data/config.json` in portable mode. Atomic temp+rename writes. Debounced (400 ms) autosave thread.
- rkyv / SQLite from the spec were rejected for now: JSON is human-editable, which the self-healing design exists to support.

### OS integrations
- `tauri-plugin-global-shortcut` 2.3: shortcut string parsed from `general.global_hotkey`; `unregister_all` then `on_shortcut`. Conflicts are logged, not fatal.
- `tauri-plugin-autostart` 2.5: `LaunchAgent` on macOS, registry Run key on Windows, passes `--minimized`.
- `tauri-plugin-single-instance`: second launch focuses the Studio window.

### Presets
- Rust `get_builtin_presets()` is the source of truth; the TS mirror exists only for `bun run dev` browser preview. Roadmap: generate bindings with `tauri-specta` (already a dependency) and drop the mirror.

---

## 4. Platform-Specific Design

### Windows 11 (primary, verified)
- Virtual desktop bounds from `GetSystemMetrics(SM_*VIRTUALSCREEN)`; shaders subtract the virtual origin so negative monitor coordinates work.
- Config: `%APPDATA%\com.nairodorian.fxcursor\config.json`.
- Historical NVIDIA/DXGI Vulkan wrapping issue (V3) has not reproduced with the Tauri transparent window; NVAPI workaround not carried over.

### macOS (unverified)
- Metal via wgpu; transparency relies on Tauri's `transparent(true)` (may need the `macos-private-api` feature for a truly transparent window).
- `device_query` requires Accessibility permission.
- Virtual bounds currently hardcoded to 1920×1080 — must use `NSScreen` (roadmap Phase E).

### Linux (unverified)
- X11: transparency needs a compositor and an ARGB visual; click-through needs an input shape.
- Wayland: no global mouse polling; layer-shell overlay is the intended path.

---

## 5. Build & Dependency Infrastructure

- `bun run tauri dev` (Vite on port 1420 + `cargo run`), `bun run build`, `bun run tauri build` (bundling currently disabled in `tauri.conf.json`).
- `bun run update-deps` probes NPM pre-release dist-tags and Crates.io `newest_version` and then runs typecheck, build, `cargo check`, `cargo test`.
- `bun run before-commit`: 7 gates (typecheck, lint, tests, build, cargo check, cargo test, version sync).
- `bun run arch`: regenerates `ARCHITECTURE.md` via Repomix `pack()`.
- `legacy/dev_scripts/` PowerShell/CMD helpers still point at `legacy/project_cursor/` (legacy).

---

## 6. Known Issues & Mitigations

| Issue                                        | Platform | Mitigation / Plan                                                                   |
| -------------------------------------------- | -------- | ----------------------------------------------------------------------------------- |
| Overlay webview RAM overhead                 | All      | Accept for now; native overlay window on the roadmap                                |
| Missed sub-poll clicks                       | All      | Raw Input / low-level hook (Phase B)                                                |
| Daemon renderer fork drifts from Tauri copy  | —        | Extract `fxcursor-render` crate (Phase C)                                           |
| Daemon pipe handle double-close, buffer caps | Windows  | Fix when the daemon becomes a real target                                           |
| `effect_mode`, `fps_counter` unused          | —        | Implement or remove (Phase A)                                                       |
| Wayland global mouse                         | Linux    | Layer-shell + seat events (Phase E)                                                 |
| macOS accessibility prompt                   | macOS    | Document; consider `CGEventTap` in the daemon                                       |
