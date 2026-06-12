# Cross-Platform CursorFX - Tauri V2 + Bun + React + TailwindCSS + wgpu

A high-performance, cross-platform cursor effects overlay application built with **Tauri V2**, **Bun**, **Vite**, **React**, and **TailwindCSS** for the configuration UI, with a **Rust + wgpu 24** backend for GPU-accelerated cursor effects.

The application creates a transparent, fullscreen overlay window that renders particle trails, click ripples, orbiting satellites, and glow effects centered around the cursor — all rendered in real-time via WebGPU (WGSL) shaders. A separate React-based configuration panel provides real-time settings control.

---

## Core Project Goals

### 1. Cross-Platform: "1 Program Fits All"
- **Tauri V2** provides platform-native windowing on **Windows 11**, **macOS**, and **Linux**
- **wgpu 24** renders via Metal (macOS), Vulkan (Linux/Windows), or DX12 (Windows)
- **Bun** provides fast package management and dev server across all platforms
- Single source tree compiles to native binaries for all three OS targets

### 2. GPU-Accelerated Real-Time Performance
- WGSL shaders run particle simulation, ripple mechanics, and ribbon trails on the GPU
- VSync-aligned rendering at display refresh rate (60–144Hz+)
- **Ultra-low GPU footprint** via instanced rendering and spline-adaptive vertex counts

### 3. Developer-Friendly: Fast & Easy
- **Bun** replaces npm for faster installs and dev server startup
- **Vite 6** with HMR for instant frontend reloads
- **TailwindCSS 4** with Vite plugin for zero-config styling
- **TypeScript 5** with strict mode for type safety

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Tauri V2 Process                      │
│                                                         │
│  ┌─────────────────┐    ┌───────────────────────────┐   │
│  │  React Frontend │    │    Rust Backend (wgpu)     │   │
│  │  (Vite + TW4)   │◄──►│    ┌───────────────────┐  │   │
│  │  Settings Panel │ IPC│    │  Overlay Window    │  │   │
│  │  Port 1420      │    │    │  Transparent       │  │   │
│  └─────────────────┘    │    │  Always-on-Top     │  │   │
│                         │    │  Click-through     │  │   │
│                         │    │  Fullscreen        │  │   │
│                         │    ├───────────────────┤  │   │
│                         │    │  WGSL Shaders:     │  │   │
│                         │    │  - Ribbon Trail    │  │   │
│                         │    │  - SDF Ripples     │  │   │
│                         │    │  - Circle Instances│  │   │
│                         │    │  - Glow Aura       │  │   │
│                         │    └───────────────────┘  │   │
│                         └───────────────────────────┘   │
└─────────────────────────────────────────────────────────┘
```

### Component Breakdown

| Layer | Technology | Purpose |
|-------|-----------|---------|
| **Application Shell** | Tauri V2 | Cross-platform window management, system tray, IPC |
| **Config UI** | React 19 + TailwindCSS 4 | Settings panel with color pickers, sliders, toggles |
| **Frontend Bundler** | Vite 6 + Bun | Fast builds, HMR, TypeScript compilation |
| **Rust Backend** | wgpu 24 + device_query 2 | GPU rendering, mouse tracking, physics simulation |
| **Config Storage** | RON (serde) | Serialized to `~/.config/CursorFX/config.ron` |
| **System Tray** | Tauri V2 tray API | Quick access to show/hide config, quit |

---

## Implemented Features

- **Ribbon Trail**: Spring-damper chain simulation with Catmull-Rom spline interpolation, multi-layer rendering with round caps, adaptive vertex quality
- **Click Ripples**: SDF-based expanding rings on mouse click with configurable duration and colors per button
- **Glow Aura**: Soft radial glow with exponential falloff centered on cursor
- **Squishy Cursor Head**: Velocity-responsive squish/stretch ellipse with motion blur
- **Particle Bursts**: Click-triggered particle system with gravity, friction, and lifetime
- **Orbiting Satellites**: Configurable count, speed, dual-ring mode, with orbit ring visualization
- **Rainbow Mode**: HSL hue cycling across trail layers
- **Adaptive Quality**: Dynamic vertex density based on path curvature and cursor speed

### Settings Panel (React)
- **General**: Enable/disable FX, choose effect mode (Ribbon/Ripple/Glow)
- **Trail Physics**: Length, size, spring/damping, position skip, spline smoothness, fade curve
- **Trail Layers**: 4 independent layers with per-layer color, width, opacity, blur
- **Cursor Head**: Squishy ellipse with size, intensity, smoothing, fill mode
- **Ripples & Particles**: Per-button colors, ripple expansion/duration, particle parameters
- **Satellites**: Count, orbit diameter, size, speed, dual-ring, orbit ring display

---

## Directory Structure

```
project_cursor/
├── src-tauri/                    # Rust backend (Tauri V2 + wgpu 24)
│   ├── Cargo.toml                # Rust dependencies
│   ├── build.rs                  # Tauri build script
│   ├── tauri.conf.json           # Tauri V2 configuration
│   ├── capabilities/
│   │   └── default.json          # Tauri V2 permission capabilities
│   ├── icons/                    # Application icons
│   └── src/
│       ├── main.rs               # Entry point
│       ├── lib.rs                # Tauri commands & state management
│       ├── config.rs             # AppConfig struct + RON serialization
│       ├── tracker.rs            # Global mouse position tracking
│       └── overlay/
│           ├── mod.rs            # Overlay window creation + render loop
│           ├── renderer.rs       # wgpu pipelines, physics, rendering
│           └── shader.wgsl       # WGSL vertex/fragment shaders
├── src/                          # React frontend (Vite + TailwindCSS)
│   ├── main.tsx                  # React entry point
│   ├── App.tsx                   # Main application shell
│   ├── App.css                   # TailwindCSS import
│   ├── lib/
│   │   ├── bindings.ts           # TypeScript type definitions
│   │   └── config.ts             # Default config values
│   └── components/
│       ├── GeneralSettings.tsx   # Enabled/effect type toggles
│       ├── TrailPhysics.tsx      # Spring-damper chain settings
│       ├── TrailLayers.tsx       # 4-layer color & blur config
│       ├── CursorHead.tsx        # Squishy head parameters
│       ├── RipplesParticles.tsx  # Ripple & particle settings
│       ├── Satellites.tsx        # Orbital satellite config
│       └── ColorPicker.tsx       # RGBA color input component
├── index.html                    # Vite entry HTML
├── package.json                  # Bun dependencies
├── vite.config.ts                # Vite + React + TailwindCSS config
├── tsconfig.json                 # TypeScript configuration
├── config.ron                    # Default runtime config template
└── run.bat                       # Windows quick-launch helper
```

---

## Getting Started

### Prerequisites
- **Rust** (MSRV 1.75+, latest stable recommended)
- **Bun** (latest, for package management and dev server)
- **Platform SDKs**: Windows MSVC, macOS Xcode, Linux build-essential

### Quick Start

```bash
# Navigate to project directory
cd project_cursor

# Install frontend dependencies (Bun)
bun install

# Run in development mode (HMR + hot reload)
bun run tauri dev

# Build for production
bun run tauri build
```

### Build Commands (via Dev Scripts)

| Script | Purpose |
|--------|---------|
| `bun run dev` | Start Vite dev server (port 1420) |
| `bun run build` | TypeScript check + Vite production build |
| `bun run tauri dev` | Tauri dev mode (frontend + Rust) |
| `bun run tauri build` | Tauri production release build |

For the legacy scripts in `dev_scripts/`, see [dev_scripts/build_instructions.md](dev_scripts/build_instructions.md).

---

## Configuration

The application stores settings in RON format at the platform-specific config directory:
- **Windows**: `%APPDATA%/CursorFX/CursorFX/config.ron`
- **macOS**: `~/Library/Application Support/com.CursorFX.CursorFX/config.ron`
- **Linux**: `~/.config/CursorFX/config.ron`

Settings are auto-saved when clicking "Save Settings" in the config panel, or via the Reset Defaults button.

---

## Comparison: V3 (Tauri V2) vs V2 (Pure Rust/winit)

| Aspect | V3 (Current) | V2 (Previous) |
|--------|-------------|---------------|
| **App Shell** | Tauri V2 | winit 0.29 event loop |
| **Config UI** | React 19 + TailwindCSS 4 | egui 0.26 immediate mode |
| **Sys Tray** | Tauri V2 tray API | tray-icon 0.14 |
| **Overlay** | Tauri window + wgpu 24 | winit window + wgpu 0.19 |
| **JS Runtime** | Bun | None |
| **Dev Experience** | HMR, TypeScript, browser devtools | Rust-only, recompile |
| **UI Customization** | TailwindCSS utility classes | egui Visuals dark theme |
| **Build** | `bun run tauri build` | `cargo build --release` |
| **RAM (idle)** | ~80–120 MB (webview overhead) | <30 MB |
| **GPU** | wgpu 24 (DX12/Vulkan/Metal) | wgpu 0.19 |

---

## Git Setup

```bash
git init
git add .
git commit -m "V3: Tauri V2 + Bun + React + TailwindCSS + wgpu 24"
git remote add origin https://github.com/NairoDorian/Cross_Platform_Rust_WebGPU_CursorFX.git
git branch -M main
git push -u origin main
```
