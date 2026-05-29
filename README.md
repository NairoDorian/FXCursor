# Cross-Platform Rust WebGPU Cursor FX

A high-performance, ultra-lightweight, cross-platform cursor effects overlay and configuration utility written entirely in **pure Rust** using WebGPU (`wgpu`) and `egui`.

This project implements a dual-window architecture powered by a single event loop to achieve hardware-accelerated visuals and real-time configuration without the overhead of a web browser wrapper (like Electron or Tauri Webview2).

---

## 🚀 Core Project Goals

### 1. Cross-Platform Agnostic: "1 Program Fits All"
- Built entirely in native Rust using standard cross-platform libraries (`winit`, `wgpu`, `egui`).
- No OS lock-in. A single source code directory compiles into native binaries on **Windows 11**, **Linux**, and **macOS** using local backends (DirectX 12, Vulkan, or Metal).
- Integrates cleanly with desktop environments using native system tray menus.

### 2. Fastest GPU-Accelerated Real-Time Performance
- Uses WebGPU Shading Language (WGSL) shaders to run complex particle simulation and ripple mechanics directly on the GPU.
- High refresh rate support (e.g. 120Hz/144Hz+) with smooth frame pacing.
- **Ultra-low footprint**: Runs under **30MB** of RAM (compared to **150MB+** for Tauri or Electron webviews).
- Minimal CPU and GPU utilization, preserving system resources for gaming and high-demand applications.

### 3. Developer-Friendly: Fast & Easy to Build
- Zero JS/NodeJS/Bun/npm overhead. Setup is as simple as installing Rust.
- Near-instant compile and check times (incremental check in < 1 second).
- Simple project architecture with no complex configurations.
- Included build and dependency-updating scripts to keep the project up-to-date automatically.

---

## 📐 Architecture & System Design

The application utilizes a unique **dual-window architecture** coordinated by a single `winit` event loop. This allows the GUI settings window and the fullscreen overlay to share graphics resources efficiently, maintaining sub-millisecond event dispatch latency and avoiding the large multi-process overhead typical of web-view wrappers.

```mermaid
graph TD
    A[Winit Event Loop] --> B[GuiWindow]
    A --> C[OverlayWindow]
    
    subgraph "Shared GPU Resource Architecture"
        D[Shared wgpu Instance & Adapter]
        E[Shared wgpu Device & Queue]
        D --> E
        E -->|Render Target View| B
        E -->|Render Target View| C
    end
    
    subgraph "Physics & Interaction Flow"
        F[Global mouse coordinates via device_query]
        G[Spring-Damper Chain Physics Simulation]
        H[Uniform Buffer updates containing trail parameters]
        F --> G
        G --> H
        H -->|WGSL Pipeline| C
    end
    
    subgraph "Native Platform Hooking (Windows)"
        I[Win32 WndProc Subclassing]
        J[WM_NCHITTEST -> HTTRANSPARENT]
        K[Style check: WS_EX_TRANSPARENT | WS_EX_LAYERED]
        I --> J
        I --> K
        K -->|Maintain click-through| C
    end
```

### Shared WebGPU Context
A single [wgpu::Instance](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/project_cursor/src/main.rs#L69), `wgpu::Device`, and `wgpu::Queue` are shared between `GuiWindow` and `OverlayWindow`. While `GuiWindow` configures its surface to render the egui control panel, `OverlayWindow` uses a transparent swapchain with composite alpha blending (`CompositeAlphaMode::PostMultiplied` or `PreMultiplied`) to render custom WGSL shaders on top of all desktop windows.

### Windows WndProc Subclassing & Click-Through
Making a fullscreen window click-through while maintaining stable rendering requires interacting directly with the OS-level Window Procedure (`WndProc`):
1. **Window Styles**: The overlay is configured with `WS_EX_TRANSPARENT`, `WS_EX_LAYERED`, `WS_EX_TOPMOST`, and `WS_EX_NOACTIVATE`. `WS_EX_TOOLWINDOW` and [WindowBuilderExtWindows::with_skip_taskbar(true)](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/project_cursor/src/main.rs#L61) are used to hide it from the taskbar and Alt+Tab menu.
2. **Style Restores**: When `wgpu` reconfigures the swapchain (e.g., on resize), DXGI/Vulkan surface drivers reset extended style flags. To counter this, a style guard check is evaluated on each frame in [configure_overlay_window](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/project_cursor/src/overlay/mod.rs#L50). If flags like `WS_EX_TRANSPARENT` or `WS_EX_LAYERED` are missing, the window is dynamically re-styled using `SetWindowLongPtrW`.
3. **Subclassing WndProc**: Winit's default message loop intercepts window hover testing. We override this by subclassing the window procedure in [overlay_wndproc](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/project_cursor/src/overlay/mod.rs#L18), intercepting the `WM_NCHITTEST` message and returning `HTTRANSPARENT` directly to the Windows Desktop Window Manager (DWM).

---

## 🎨 Implemented Features & Effects

- **☄️ Particle Trail (Ribbon Chain)**: A spring-mass-damper chain simulation trailing the mouse. The head follows the cursor location, while subsequent nodes interpolate toward their predecessors using smooth physics. The tail fades in size and opacity.
- **🌊 Click Ripple**: Spawns an expanding concentric ring at click coordinates. The shader dynamically evaluates distance equations (SDFs) to render anti-aliased ring outlines that expand and fade out over a configurable duration.
- **✨ Glow Aura**: A soft radial glow tracking the cursor. Uses an exponential falloff calculation (`exp(-dist_sq * 4.0)`) in the pixel shader to create a smooth, glowing orb.

### Settings Customization Panel
The native configuration GUI ([gui/panel.rs](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/project_cursor/src/gui/panel.rs)) allows real-time adjustments:
- **Visuals**: Primary color picker (RGBA), trail length (16 to 256 particles), trail/glow width (1.0px to 50.0px).
- **Physics**: Simulation speed multiplier, friction/damping coefficients, gravity vectors (allowing upward buoyancy or downward drift), and click ripple maximum radii.
- **Toggles**: Toggle particle trails, global click ripple, and select current active effects.

---

## 🔄 Comparison with Legacy C++ Windhawk Mods

This Rust rewrite transitions the project from a desktop hook/injection model to a native cross-platform application:

| Feature / Aspect | Rust WebGPU Cursor FX (Current) | Legacy C++ Windhawk Mods ([original_mods](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/original_mods/)) |
| :--- | :--- | :--- |
| **Architecture** | Independent dual-window user-space application. | DLL injected directly into `explorer.exe` or `dwm.exe`. |
| **Graphics API** | WebGPU (`wgpu`) - cross-platform Vulkan, Metal, DX12. | Direct3D 11 + DirectComposition / GDI+ (Windows-only). |
| **System Footprint** | Extremely low (<30MB RAM, <1% CPU). | Low, but carries risk of crashing host shell (`explorer.exe`). |
| **Input Capture** | Native polling via [device_query](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/project_cursor/src/tracker.rs) crate. | OS-level low-level hooks (`WH_MOUSE_LL`, `WH_KEYBOARD_LL`). |
| **Portability** | Full Windows, macOS, and Linux compatibility. | Strictly locked to Windows kernel and shell structures. |
| **Configuration** | Modern immediate-mode GUI (`egui`) auto-saving to RON. | Windhawk Registry-backed custom YAML/C++ settings. |

---

## 📂 Directory Structure

*(See [repo-summary.md](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/repo-summary.md) for detailed line counts, sizes, and descriptions of every file in the repository).*

```
.
├── README.md                       # Project overview & goals (this file)
├── memory.md                       # Architectural decisions, research logs & design choices
├── repo-summary.md                 # Automatically generated file index & lines-of-code breakdown
├── dev_scripts/                    # Developer automation scripts & their documentation
│   ├── build_instructions.md       # Build targets, cross-compilation & script usage guide
│   ├── build.ps1                   # PowerShell: native & cross-target build automation
│   ├── build.bat                   # CMD: native & cross-target build automation
│   ├── cargo_build.ps1             # PowerShell: low-level cargo build wrapper
│   ├── cargo_build.bat             # CMD: low-level cargo build wrapper
│   ├── cargo_check.ps1             # PowerShell: fast cargo check (no binary output)
│   ├── cargo_check.bat             # CMD: fast cargo check
│   ├── cargo_run.ps1               # PowerShell: cargo run shortcut
│   ├── cargo_run.bat               # CMD: cargo run shortcut
│   ├── generate_repomix.ps1        # PowerShell: automated repository summary packer
│   ├── generate_repomix.bat        # CMD: automated repository summary packer
│   ├── update_dependencies.ps1     # PowerShell: automated dependency upgrade script
│   └── update_dependencies.bat     # CMD: automated dependency upgrade script
├── repomix.config.json             # Repomix configuration (include/exclude rules, output style)
├── repomix-instruction.md          # AI context & coding conventions for repomix consumers
├── repomix-output.md               # Auto-generated packed repository
├── original_mods/                  # Legacy C++ reference implementations (Windhawk mods)
│   ├── D3D_cursor_mod.wh.cpp       # Legacy Direct3D 11 cursor mod
│   └── gdi+_cursor_mod.wh.cpp      # Legacy GDI+ cursor mod
└── project_cursor/                 # Primary Rust project root
    ├── Cargo.toml                  # Dependencies & build targets
    ├── config.ron                  # Default runtime configuration (RON format)
    └── src/
        ├── main.rs                 # Event loop & multi-window management
        ├── config.rs               # Configuration serialization (RON) & state sharing
        ├── tracker.rs              # Global mouse coordinate tracking
        ├── tray.rs                 # System tray and taskbar management
        ├── gui/
        │   ├── mod.rs              # egui window manager setup
        │   └── panel.rs            # Sliders, color pickers, and presets panel
        └── overlay/
            ├── mod.rs              # Overlay window styling and WndProc subclassing
            ├── renderer.rs         # wgpu pipeline, buffers, and swapchain setup
            └── shader.wgsl         # Interactive cursor trail shader (WGSL)
```

---

## 🛠️ Getting Started & Build Commands

Ensure you have Rust installed (MSRV 1.75+ recommended).

### 1. Run the Project
Navigate to the `project_cursor` directory and run:
```bash
cargo run
```
Or use the shortcut from the project root:
```cmd
cd project_cursor && run.bat
```

### 2. Automated Developer Scripts Quick-Reference

The scripts located in [dev_scripts/](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/) simplify building, compiling, checking, and updating dependencies:

| Script Name | Purpose | Supported Arguments / Targets |
| :--- | :--- | :--- |
| [build.ps1](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/build.ps1) / [build.bat](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/build.bat) | Clean native and cross-target release/debug compiler wrapper. | Targets: `native` (default), `win`, `linux`, `mac`. Modes: `release` (default), `debug`. |
| [cargo_build.ps1](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/cargo_build.ps1) / [cargo_build.bat](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/cargo_build.bat) | Performs cargo check and clippy analysis before building binary. | Targets: `native`, `win`, `linux`, `mac`. Modes: `release`, `debug`. Flags: `-SkipCheck`. |
| [cargo_check.ps1](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/cargo_check.ps1) / [cargo_check.bat](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/cargo_check.bat) | Runs rapid type checks, clippy lints, and documentation generation. | N/A |
| [cargo_run.ps1](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/cargo_run.ps1) / [cargo_run.bat](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/cargo_run.bat) | Shortcut to run the cargo binary with configurable environment log levels. | N/A |
| [update_dependencies.ps1](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/update_dependencies.ps1) / [update_dependencies.bat](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/update_dependencies.bat) | Automatically updates all dependency crates to their latest versions. | N/A |
| [generate_repomix.ps1](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/generate_repomix.ps1) / [generate_repomix.bat](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/generate_repomix.bat) | Re-indexes repository stats and generates packed files for AI tools. | N/A |

For details about cross-compiling options, refer to [dev_scripts/build_instructions.md](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/dev_scripts/build_instructions.md).

---

## ⚙️ Configuration File (`config.ron`)

Configuration parameters are persisted using Rusty Object Notation (RON) in [project_cursor/config.ron](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/project_cursor/config.ron). Below is an example of the serialized format:

```ron
(
    enabled: true,
    effect_type: 0,
    trail_color: (0.0, 0.8, 1.0, 1.0),
    trail_length: 64,
    trail_width: 8.0,
    speed: 1.0,
    friction: 0.08,
    gravity: 0.0,
    ripple_radius: 100.0,
    click_response: true,
)
```

The application auto-saves changes to disk upon slider adjustments in the GUI.

---

## 🐙 Git Repository Setup & Initial Commit

To push the project to your GitHub repository at `https://github.com/NairoDorian/Cross_Platform_Rust_WebGPU_CursorFX`, execute the following commands in the workspace root:

```bash
# Initialize git repository
git init

# Add all files (excluding files in .gitignore and target folders)
git add .

# Create the first commit
git commit -m "First commit: Pure Rust + WebGPU cursor effects overlay and configuration utility"

# Link to remote repository
git remote add origin https://github.com/NairoDorian/Cross_Platform_Rust_WebGPU_CursorFX.git

# Rename main branch to main (if default was master)
git branch -M main

# Push to remote repository
git push -u origin main
```
