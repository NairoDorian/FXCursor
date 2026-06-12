# Changelog

## [0.2.0] - 2026-06-11

### Architecture Rewrite (V3)
- **Replaced** pure Rust `winit 0.29` + `egui 0.26` with **Tauri V2** + **React 19** + **TailwindCSS 4**
- **Replaced** `tray-icon` crate with Tauri V2 native tray API
- **Replaced** egui immediate-mode config panel with React webview (7 components)
- **Kept** wgpu-based overlay rendering engine (WGSL shaders unchanged)

### Dependency Upgrades (All Latest)
- `wgpu` 0.19 → **29.0.3** (Metal 3, Vulkan 1.3, DX12, WGSL+)
- `ron` 0.8 → **0.12.1**
- `device_query` 1.1 → **4.0.1**
- `windows-sys` 0.52 → **0.61**
- `directories` 5.0 → **6.0**
- `tauri` → **2.11.2**
- `react` → **19.2.7**
- `vite` → **6.4.3**
- `tailwindcss` → **4.3.0**
- `typescript` → **5.9.3**

### Added
- React config panel with 7 settings sections (General, Trail Physics, Layers, Cursor Head, Ripples/Particles, Satellites)
- TailwindCSS 4 dark theme with cyan accent
- TypeScript strict mode with full type coverage
- Bun package manager support
- Tauri V2 IPC commands: `get_config`, `update_config`, `save_config`, `reset_defaults`, `toggle_overlay`, `get_overlay_status`
- Programmatic overlay window creation via Tauri WebviewWindowBuilder
- Fullscreen overlay with primary monitor detection
- `pollster` re-added for async wgpu device/adapter requests

### Fixed
- **wgpu 29 API migration**: `CurrentSurfaceTexture` enum, `multiview` → `multiview_mask`, `PipelineLayoutDescriptor` restructure, `InstanceDescriptor` full field specification, `DeviceDescriptor` with `experimental_features` and `trace`
- **Click-through regression**: WndProc styles now re-applied every frame (was broken to once-only)
- **windows-sys 0.61 API**: `SetWindowPos` pointer types, `LoadLibraryW` return type, `HWND` type changes
- **Surface creation**: Uses `SurfaceTargetUnsafe::RawHandle` for Tauri window compatibility
- **Tauri Manager import**: Added `use tauri::Manager` for `get_webview_window`
- **NVAPI fix**: Updated to windows-sys 0.59/0.61 APIs with proper pointer types

### Removed
- Pure Rust `main.rs` with winit event loop
- `tray.rs` (tray-icon crate)
- `nvapi.rs` (merged into overlay/mod.rs)
- `gui/mod.rs` and `gui/panel.rs` (egui-based)
- Old `overlay/mod.rs` (winit-based window creation)
- Legacy `project_cursor/Cargo.toml` (now in `src-tauri/`)

### Documentation
- README.md: Full rewrite for V3 architecture with mermaid diagrams
- memory.md: Added V3 architecture decision log, updated budgets and platform notes
- build_instructions.md: Added Bun + Tauri workflow
- repomix-instruction.md: Updated for V3
- repomix.config.json: Updated include/exclude patterns for new structure
- dev_scripts: All paths updated to `project_cursor/src-tauri/`

### Known Issues
- DX12 backend fails with `OutOfMemory` on RTX 4070 for transparent overlay (Vulkan is default, works perfectly)
- 14 NVAPI naming convention warnings (harmless, Windows-only)
- `surface_format` unused assignment warning (values set dynamically in surface creation)
