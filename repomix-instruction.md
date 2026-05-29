# Project Context & AI Instructions

## Project Overview
**Cross-Platform Rust WebGPU Cursor FX** is an ultra-lightweight, GPU-accelerated cursor effects overlay and configuration utility written entirely in pure Rust. It replaces earlier Tauri/Electron-based prototypes with a native dual-window architecture using `winit` + `wgpu` + `egui`.

## Architecture Summary
- **Dual-window system**: A transparent fullscreen overlay (renders GPU particle effects) and a configuration panel (egui settings UI), sharing a single `winit` event loop.
- **GPU pipeline**: WGSL shaders run particle simulation, cursor trails, and ripple effects directly on the GPU via `wgpu`. Rendering targets the display refresh rate (60–144Hz+) using VSync.
- **Config persistence**: Settings are serialized to `config.ron` (RON format) and shared between windows via `Arc<Mutex<Config>>`.
- **System tray**: `tray-icon` crate provides a native tray icon with show/hide/quit actions.
- **Input tracking**: `device_query` polls global mouse coordinates each frame without hooking OS input events.

## Key Technical Constraints
1. **Click-through overlay**: On Windows, the overlay uses `WS_EX_LAYERED | WS_EX_TRANSPARENT` with WndProc subclassing (`WM_NCHITTEST → HTTRANSPARENT`) to pass all clicks to underlying windows. Styles must be reapplied after every `wgpu` surface reconfiguration.
2. **Taskbar hiding**: Overlay is hidden from Alt+Tab and taskbar via `WS_EX_TOOLWINDOW` + `skip_taskbar(true)`, with a guard loop to re-apply if DWM resets styles.
3. **Performance budget**: < 30MB RAM, < 1% CPU, < 2% GPU. Frame rendering bypasses `request_redraw()` to avoid focus-throttling by the OS.

## Coding Style & Conventions
- **Language**: Rust (edition 2021, MSRV 1.75+)
- **Error handling**: Use `anyhow` for application errors, `log` + `env_logger` for diagnostics.
- **Platform code**: Guard OS-specific code with `#[cfg(target_os = "...")]` attributes. Windows-specific Win32 calls use the `windows` crate with raw `unsafe` blocks.
- **Shader language**: WGSL (WebGPU Shading Language) — no GLSL or HLSL.
- **Config format**: RON (Rusty Object Notation), not JSON or TOML.
- **Build profiles**: Debug keeps console visible; Release uses `#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]` to hide it.

## Directory Layout
- `project_cursor/` — The Rust project root (contains `Cargo.toml` and `src/`)
- `dev_scripts/` — Build automation scripts (PowerShell + CMD) and their documentation
- `original_mods/` — Legacy C++ reference implementations (Windhawk mods, read-only reference)
- `memory.md` — Architectural decisions log (not code, but critical project context)
