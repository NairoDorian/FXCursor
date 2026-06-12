# Architectural Memory & Decisions Log

This document tracks design decisions, hardware interactions, crate evaluations, and system resource budgets for the Cross-Platform CursorFX application.

---

## 1. Architectural History

| Version | Shell | Frontend | Graphics | RAM | Startup | Notes |
|---------|-------|----------|----------|-----|---------|-------|
| **V1** (Legacy) | Tauri V1 | SolidJS + Bun | GDI+, D3D11 | ~150–250 MB | ~1.2–2.0s | Windows-only, Windhawk DLLs |
| **V2** (Pure Rust) | winit 0.29 | egui 0.26 | wgpu 0.19 | <30 MB | ~0.1–0.2s | Cross-platform, no webview |
| **V3** (Current) | Tauri V2 | React 19 + TW4 | wgpu 24 | ~80–120 MB | ~0.5–1.0s | Best DX, polished UI, cross-platform |

### Why V3 (Tauri V2 + React)?

1. **Developer Experience**: Hot Module Replacement, TypeScript, React DevTools, browser-based debugging make iteration dramatically faster than egui.
2. **UI Polish**: TailwindCSS 4 enables pixel-perfect design with dark theme, responsive layout, and animated transitions that egui cannot match.
3. **Tauri V2 Maturity**: V2 features native tray, multi-window, IPC streaming, plugin system, and reduced webview overhead compared to V1.
4. **Ecosystem**: Access to the full npm/Bun ecosystem for utilities, state management, and testing. The React component model maps naturally to settings panels.
5. **WebGPU Integration**: wgpu 24 brings improved DX12, Vulkan 1.3, and Metal 3 support. The Rust backend keeps the same WGSL shaders unchanged.

---

## 2. Resource & Performance Budgets (V3)

| Metric | Target | Notes |
|--------|--------|-------|
| **RAM (total process)** | <120 MB | Webview ~60MB + GPU resources ~30MB + Rust ~10MB |
| **CPU (idle)** | <1% | Vite dev server excluded (dev only) |
| **CPU (active rendering)** | <2% | Frame pacing at display refresh rate |
| **GPU (active)** | <3% | Instanced rendering, adaptive LOD |
| **Binary size (release)** | ~15–20 MB | LTO + strip + panic=abort |
| **Startup time** | <1.0s | Tauri V2 cold start |
| **Frame pacing** | VSync-aligned | PresentMode::Fifo |

---

## 3. Crate Evaluation & Risk Matrix

### A. Window Management (Tauri V2)
- **Chosen**: Tauri V2 (`tauri = "2"`)
- **Rationale**: Built on `tao` (successor to winit). Multi-window support. Native window handle access for wgpu. System tray, global shortcuts, and menu APIs built-in.
- **Risk**: Webview adds ~60MB overhead. `Tauri 2.x` is less battle-tested than winit. Some platform-specific transparency quirks (Wayland).

### B. Graphics API (wgpu 24)
- **Chosen**: wgpu 24
- **Rationale**: Latest stable release. Improved DX12 backend. Synchronous adapter/device creation. WGSL shaders compile at build time.
- **Changes from 0.19**: `DeviceDescriptor` gained `memory_hints`. `ShaderModuleDescriptor` gained `compilation_options`. Surface creation uses `SurfaceTarget`.

### C. Configuration GUI (React 19 + TailwindCSS 4)
- **Chosen**: React 19 + Vite 6 + TailwindCSS 4
- **Rationale**: Industry standard. Tailwind v4 uses CSS-based configuration (`@theme`) eliminating `tailwind.config.ts`. The `@tailwindcss/vite` plugin provides zero-config integration.
- **Risk**: Webview rendering overhead. IPC latency for config updates (mitigated by optimistic UI updates).

### D. Global Input (device_query 2)
- **Chosen**: `device_query = "2"`
- **Rationale**: Same proven approach as V2. Polls global mouse position without hooking OS events. Works on all platforms (Wayland requires compositor cooperation).
- **Risk**: Linux Wayland blocks global mouse polling. Fallback: overlay window mouse events (but click-through prevents this).

---

## 4. Platform-Specific Design

### Windows 11 (Primary)
- DX12 backend via wgpu 24
- Overlay window: `WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE`
- WndProc subclassing: `WM_NCHITTEST → HTTRANSPARENT` for click-through
- NVIDIA NVAPI fix: Sets Vulkan present method to "Prefer Native" to prevent DXGI swapchain wrapping
- Config saved to `%APPDATA%/CursorFX/CursorFX/config.ron`

### macOS (Metal)
- Metal 3 backend via wgpu 24
- Transparency via `NSWindow` properties
- Accessibility permissions required for `device_query` global mouse polling
- Config saved to `~/Library/Application Support/com.CursorFX.CursorFX/config.ron`

### Linux (Vulkan)
- Vulkan 1.3 backend via wgpu 24
- X11: Click-through works via X11 window properties
- Wayland: Requires compositor support for `ext-window-input-v1` or similar protocols
- Config saved to `~/.config/CursorFX/config.ron`

---

## 5. Build & Dependency Infrastructure

### Tauri V2 Build Pipeline
1. `bun run dev` starts Vite dev server on port 1420
2. `bun run tauri dev` starts Tauri dev mode (compiles Rust, launches Vite)
3. `bun run tauri build` creates production release binary

### Dependency Upgrades
- Rust: `cargo upgrade --to-latest` + `cargo update`
- Bun: `bun update` (updates package.json + lockfile)

### Dev Scripts
Located in `dev_scripts/`:
- `build.ps1`/`build.bat` - Cargo build automation
- `cargo_check.ps1`/`cargo_check.bat` - Fast cargo check + clippy
- `update_dependencies.ps1`/`update_dependencies.bat` - Automated Rust dep upgrades
- See `dev_scripts/build_instructions.md` for full documentation

---

## 6. Known Issues & Mitigations

| Issue | Platform | Mitigation |
|-------|----------|------------|
| Webview RAM overhead | All | Acceptable trade-off for dev experience. Release builds minimize with LTO |
| Wayland global mouse | Linux | Fallback to relative overlay window coordinates |
| NVIDIA DXGI wrapping | Windows | NVAPI fix in `overlay/mod.rs` |
| wgpu surface style resets | Windows | Guard loop re-applies styles after each configure |
| macOS accessibility prompt | macOS | Required for device_query; user must grant in System Preferences |
