# Architectural Memory & Decisions Log

This document tracks design decisions, hardware interactions, crate evaluations, and system resource budgets for the **Cross-Platform Rust WebGPU Cursor FX** application.

---

## 1. Architectural History: From Tauri to Pure Rust

| Architecture | Front-end Tech | Graphics Tech | RAM Profile | Startup Latency | Key Constraints / Issues |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Old V1/V2** | Tauri, Bun, SolidJS, HTML/CSS | GDI+, D3D11 | ~150 - 250 MB | ~1.2s - 2.0s | Platform-locked (Windows 11), heavy Webview memory footprint, complex node modules, build lag. |
| **New V3 (Selected)** | `egui` (Immediate Rust UI) | `wgpu` (WGSL Shaders) | **< 30 MB** | **~0.1s - 0.2s** | Global input permission (macOS), Wayland sandboxing restrictions (Linux). |

### Key Reasons for Choosing Pure Rust (`winit` + `wgpu` + `egui`):
1. **WebGPU Driver Performance**: Renders natively on Metal (macOS), Vulkan (Linux/Windows), or DX12 (Windows) via `wgpu` without being limited by Webview browser sandboxing or lack of modern graphics feature flags in system webviews.
2. **RAM Footprint**: Webview-based configurations require spawning Chromium (Webview2) render processes. Moving UI rendering into an `egui` layer running on the same graphics context as the overlay eliminates Webview2 completely.
3. **Packaging Simplicity**: Generates a single compact binary (~10-15MB after optimizations like `panic = "abort"` and `opt-level = "z"`), removing Node/Bun dependencies.

---

## 2. Resource & Performance Budgets

- **Memory Usage (RAM)**: `< 30 MB` (Goal: `< 20 MB` at idle).
- **CPU Utilization**: `< 1%` under active rendering (using frame-pacing and lazy event-loop handling for the Config Panel).
- **GPU Utilization**: `< 2%` on typical dedicated or modern integrated GPUs (e.g., Intel Iris Xe, Apple M-series, Nvidia GTX/RTX).
- **Frame Pacing**: Lock rendering of Overlay to target display refresh rate (e.g., 60Hz, 120Hz, 144Hz) using `wgpu`'s VSync-aware presenter (`PresentMode::AutoFifo` or `PresentMode::Fifo`).

---

## 3. Crate Evaluation & Risk Matrix

### A. Window Management (`winit` vs. `sdl2` vs. `glfw`)
- **Chosen**: `winit`
- **Rationale**: De-facto standard in the Rust ecosystem. Native support for multi-window setups, raw-window-handle bindings for `wgpu`, and advanced OS window attributes (`set_cursor_hittest` for click-through, transparency flags).
- **Risk**: Minor platform-specific quirks (e.g., Linux transparency under Wayland requires compositor cooperation).

### B. Graphics API (`wgpu` vs. `glow` / `OpenGL` vs. `ash` / `Vulkan`)
- **Chosen**: `wgpu`
- **Rationale**: Offers a modern, unified WebGPU API. Safer than raw Vulkan (`ash`) or OpenGL (`glow`), with compile-time shader validation (WGSL). Excellent performance across Metal (macOS), Vulkan, and DirectX 12.

### C. Configuration GUI (`egui` vs. `iced` vs. `slint`)
- **Chosen**: `egui`
- **Rationale**: Extremely fast development cycle with immediate-mode design. Direct integration with `wgpu` via `egui-wgpu`. Does not require complex state machines for simple slider-driven settings.

### D. Global Input (`device_query` vs. `rdev` vs. native API hooks)
- **Chosen**: `device_query`
- **Rationale**: Simple API, does not require starting complex event loops or hooking keypresses, minimizing the security footprint (does not require keylogger permissions on Windows, though macOS accessibility may still prompt).
- **Risk**: On Linux Wayland, global mouse coordinates are blocked by the OS security model. Wayland users may require fallback mechanisms.

---

## 4. Platform-Specific Design Patterns

### Windows 11 (Primary Target)
- Uses `WindowAttributes::with_transparent(true)` and `window.set_cursor_hittest(false)`.
- DirectX 12 backend used by `wgpu`.
- System tray runs via `tray-icon` linked into the standard winit event loop.

### macOS (Metal)
- Requires enabling `NSApplication` activation policy to handle menu bar and tray behavior correctly.
- Transparency and click-through are supported using Cocoa window styling flags.

### Linux (Vulkan / Wayland & X11)
- Under X11, click-through works out of the box using X11 window properties.
- Under Wayland, transparent overlays require the compositor to support `ext-window-input-v1` or similar protocols. A fallback configuration to run with window borders or hover-only tracking is planned if click-through is rejected by the window manager.

---

## 5. Overlay Click-Through & Taskbar Hiding Troubleshooting

The Windows 11 click-through overlay and taskbar-hiding functionality required solving three hidden platform-level conflicts:

### A. wgpu Surface Configuration Style Resets
- **The Problem**: The `WS_EX_TRANSPARENT` and `WS_EX_LAYERED` styles were initially applied once at raw window creation. However, when `wgpu` configures the render swapchain (`surface.configure`), it re-initializes DXGI/Vulkan surface descriptors, which silently resets the window's extended style styles (`GWL_EXSTYLE`) back to default.
- **The Resolution**: We moved the styling overrides directly into `new` and `resize` routines of `OverlayWindow` and `GuiWindow`, ensuring they are reapplied immediately *after* any `wgpu` surface configuring completes.

### B. winit WndProc Message Interception
- **The Problem**: Even with the correct extended styles, `winit`'s internal window procedure intercepts mouse events. When Windows queries the cursor position using `WM_NCHITTEST`, `winit`'s procedure caught it and returned `HTCLIENT` (client area hit) because the internal `cursor_hittest` flag was set to true. This overrode the OS-level `WS_EX_TRANSPARENT` behavior.
- **The Resolution**: We combined two solutions:
  1. Call `window.set_cursor_hittest(false)` to update `winit`'s internal hit-test flag, preventing it from returning `HTCLIENT`.
  2. Use `SetWindowLongPtrW` with `GWLP_WNDPROC` to subclass the window procedure natively, intercepting `WM_NCHITTEST` and returning `HTTRANSPARENT` directly to the OS, bypassing winit's pipeline.

### C. Persistent Taskbar & Alt+Tab Visibility
- **The Problem**: In Windows 11, clearing `WS_EX_APPWINDOW` and applying `WS_EX_TOOLWINDOW` at window creation wasn't completely persistent, nor did it always succeed if the window was shown before styles were applied. The window manager would keep registering the window's presence in Alt+Tab and the taskbar, or DXGI surface swaps would reset extended style bits.
- **The Resolution**:
  1. Used the `WindowBuilderExtWindows::with_skip_taskbar(true)` extension trait to prevent the window from ever entering the taskbar registry during its creation phase.
  2. Set `WindowExtWindows::set_skip_taskbar(true)` dynamically.
  3. Created a guard checks structure inside the main rendering loop (`OverlayWindow::render` calling `configure_overlay_window`), which checks `style != new_style` via `GetWindowLongPtrW` and dynamically re-applies style adjustments if any driver/compositor-level event clears them.

### D. Modern DXGI Swapchain & Click-Through Coexistence
- **The Problem**: Making the overlay window click-through to *other* applications (processes) requires the `WS_EX_LAYERED` style bit. Without `WS_EX_LAYERED`, returning `HTTRANSPARENT` from `WM_NCHITTEST` only clicks through to parent/owner windows of the same process. However, calling `SetWindowLongPtrW` and `SetWindowPos` on every frame to ensure these styles are present forces the Windows DWM to constantly invalidate the window frame/composition buffers, breaking DXGI composition presenting and making the rendering disappear.
- **The Resolution**:
  1. **Bitmask Checks**: Implemented a precise style diff checks mask in `configure_overlay_window` (`style & required_flags == required_flags` and checking `WS_EX_APPWINDOW` is absent). This ensures `SetWindowLong` and `SetWindowPos` are only called *once* during start/resize, and *never* on normal frames.
  2. **WndProc Monitor**: Monitored the window's current procedure (`GWLP_WNDPROC`). If winit/DXGI resets it to winit's default during surface recreation, the code automatically re-subclasses it, storing the new winit pointer in `PREV_WNDPROC`.
  3. **Bypassing Focus Event Throttling**: Moved the overlay rendering calls directly into the 120Hz update tick under `Event::AboutToWait` in `main.rs`, instead of relying on `request_redraw()`. This prevents the OS from throttling or discarding paint events when the transparent overlay window loses input focus.

---

## 6. Build & Dependency Update Infrastructure

### A. Automation Scripts (`build.ps1/bat`, `cargo_build.ps1/bat`, `cargo_check.ps1/bat`, `cargo_run.ps1/bat`)
- **Location**: All scripts have been consolidated under `dev_scripts/`.
- **Decision**: Implemented native Windows batch script wrappers and PowerShell scripts.
- **Why**: Navigates into `project_cursor` automatically and compiles via cargo. Supports target-specific toolchain configurations (`native`, `win`, `linux`, `mac`) to prepare the codebase for multi-platform distribution.

### B. Dependency Updater (`update_dependencies.ps1/bat`)
- **Location**: `dev_scripts/update_dependencies.ps1` and `dev_scripts/update_dependencies.bat`.
- **Decision**: Leverages `cargo-edit`'s `cargo upgrade --to-latest` followed by `cargo update`.
- **Why**: Restricting version constraints to hardcoded strings prevents automated security and API performance patches from downstream libraries. Bumping constraints in `Cargo.toml` automatically keeps the program up-to-date with upstream changes on the next build.
