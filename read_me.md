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

## 📂 Directory Structure

```
.
├── read_me.md                  # Project overview & goals
├── memory.md                   # State memory, research logs, and design choices
├── build_instructions.md       # Target build and script documentation
├── build.ps1 / build.bat       # Native & target build automation scripts
├── update_dependencies.ps1/bat # Automated dependency upgrade scripts
├── repomix-summary.md          # Packed repository structure metadata
└── project_cursor/             # Primary Rust project root
    ├── Cargo.toml              # Dependencies & build targets
    └── src/
        ├── main.rs             # Event loop & multi-window management
        ├── config.rs           # Configuration serialization (RON) & state sharing
        ├── tracker.rs          # Global mouse coordinate tracking
        ├── tray.rs             # System tray and taskbar management
        ├── gui/
        │   ├── mod.rs          # egui window manager setup
        │   └── panel.rs        # Sliders, color pickers, and presets panel
        └── overlay/
            ├── mod.rs          # Overlay window styling and WndProc subclassing
            ├── renderer.rs     # wgpu pipeline, buffers, and swapchain setup
            └── shader.wgsl     # Interactive cursor trail shader (WGSL)
```

---

## 🛠️ Getting Started & Build Commands

Ensure you have Rust installed (MSRV 1.75+ recommended).

### 1. Run the Project
Navigate to the `project_cursor` directory and run:
```bash
cargo run
```

### 2. Build Release Binaries
You can build native binaries using the build scripts at the root of the repository:
```powershell
# PowerShell
.\build.ps1
```
```cmd
:: CMD Command Prompt
build.bat
```
*(For cross-compilation targeting Linux or macOS from Windows, please refer to [build_instructions.md](file:///c:/Users/Z/Downloads/PROJECTS/Cross_Platform_Rust_WebGPU_CursorFX/build_instructions.md).)*

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
