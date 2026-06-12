# Project Context & AI Instructions

## Project Overview
**Cross-Platform CursorFX** is a GPU-accelerated cursor effects overlay and configuration utility built with **Tauri V2**, **Bun**, **React 19**, **TailwindCSS 4**, and **wgpu 24**. It renders particle trails, click ripples, orbiting satellites, and glow effects in a transparent fullscreen overlay via WGSL shaders.

## Architecture Summary
- **App Shell**: Tauri V2 process manages two windows: a React webview (config panel) and a transparent overlay window (wgpu rendering)
- **GPU Pipeline**: WGSL shaders in `src-tauri/src/overlay/shader.wgsl` render ribbon trails, SDF circles/rings, and glow effects via instanced drawing
- **IPC**: React frontend communicates with Rust backend via Tauri V2 `invoke()` commands (`get_config`, `update_config`, `save_config`, `reset_defaults`)
- **Config Persistence**: `AppConfig` struct in `src-tauri/src/config.rs` serializes to RON format at platform config directory
- **Render Loop**: Background thread in `src-tauri/src/overlay/mod.rs` polls global mouse via `device_query`, updates physics, renders via wgpu
- **System Tray**: Tauri V2 tray API with show/hide and quit actions

## Key Technical Constraints
1. **Click-through Overlay**: Windows uses `WS_EX_LAYERED | WS_EX_TRANSPARENT` with WndProc subclassing (`WM_NCHITTEST → HTTRANSPARENT`). Styles must survive wgpu surface reconfiguration.
2. **Taskbar Hiding**: `WS_EX_TOOLWINDOW` + `skip_taskbar(true)` with guard loop against DWM resets.
3. **Surface Recovery**: If wgpu surface is lost (`SurfaceError::Lost`), the render loop recreates it automatically.
4. **NVIDIA Fix**: NVAPI sets Vulkan presentation to "Prefer Native" on Windows to prevent DXGI wrapping.

## Coding Style & Conventions
- **Rust**: edition 2021, `#[cfg(target_os)]` for platform code, `unsafe` for Win32 calls
- **TypeScript**: Strict mode, `noUnusedLocals`, `noUnusedParameters`
- **React**: Functional components with hooks, optimistic UI updates before IPC
- **Styling**: TailwindCSS 4 utility classes, dark theme (gray-950 base)
- **Shaders**: WGSL only, no GLSL/HLSL
- **Config**: RON format via `serde` + `ron` crate

## Directory Layout
- `project_cursor/src-tauri/` — Rust backend (Tauri V2 + wgpu 24)
- `project_cursor/src/` — React frontend (Vite + TailwindCSS 4)
- `dev_scripts/` — Build automation (PowerShell + CMD)
- `original_mods/` — Legacy C++ reference implementations
- `memory.md` — Architectural decisions log
