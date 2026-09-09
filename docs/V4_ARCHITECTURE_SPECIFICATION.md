# FXCursor V4: Next-Generation Cross-Platform Architecture Specification

> **Status:** Comprehensive V4 Production Blueprint (Hardened Edition)  
> **Target Systems:** Windows 10/11 (x64/ARM64), macOS 12+ (Apple Silicon & Intel), Linux (X11 & Wayland)  
> **Key Performance Goals:** < 12 MB Idle RAM, 0.0% Idle CPU/GPU, < 3 ms Frame Latency, 100% Cross-Platform

---

## Table of Contents

1. [System Topology & High-Level Architecture](#1-system-topology--high-level-architecture)
2. [Forensic Lessons Learned from Historical & Existing Projects](#2-forensic-lessons-learned-from-historical--existing-projects)
3. [The Seven Pillars of the Hardened V4 Architecture](#3-the-seven-pillars-of-the-hardened-v4-architecture)
   - [Pillar 1: Decoupled Micro-Daemon + SolidJS 2.0 Transient UI](#pillar-1-decoupled-micro-daemon--solidjs-20-transient-ui)
   - [Pillar 2: GPU Compute-Driven Physics & Spline Evaluation](#pillar-2-gpu-compute-driven-physics--spline-evaluation)
   - [Pillar 3: Multi-Monitor Virtual Coordinate Projection Matrix](#pillar-3-multi-monitor-virtual-coordinate-projection-matrix)
   - [Pillar 4: Zero-Overhead Hardware Event Sleep & UIPI Fallback](#pillar-4-zero-overhead-hardware-event-sleep--uipi-fallback)
   - [Pillar 5: Deterministic Sub-Stepping Physics & Normal Stabilization](#pillar-5-deterministic-sub-stepping-physics--normal-stabilization)
   - [Pillar 6: Triple-Platform Direct Surface Compositing](#pillar-6-triple-platform-direct-surface-compositing)
   - [Pillar 7: Multi-Buffer Flush Engine & Ghost Elimination](#pillar-7-multi-buffer-flush-engine--ghost-elimination)
4. [Zero-Copy IPC & State Engine (`rkyv`)](#4-zero-copy-ipc--state-engine-rkyv)
5. [Complete Module & Directory Structure](#5-complete-module--directory-structure)
6. [Detailed Phased Migration Roadmap](#6-detailed-phased-migration-roadmap)
7. [Target Performance Benchmark Table](#7-target-performance-benchmark-table)

---

## 1. System Topology & High-Level Architecture

FXCursor V4 completely decouples the **GPU Render Daemon** from the **User Configuration Interface**.

```
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   FXCURSOR V4 SYSTEM TOPOLOGY                                    │
└──────────────────────────────────────────────────────────────────────────────────────────────────┘

                                    [ TRANSIENT SETTINGS UI ]
                                 SolidJS 2.0 + Modern Native CSS
                                  (Vite + Native Webview Shell)
                                 *Runs ONLY when user opens UI*
                                                │
                                                │ Zero-Copy IPC (Named Pipe / Unix Domain Socket)
                                                ▼
┌──────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               FXCURSOR CORE MICRO-DAEMON (Rust)                                  │
│                                (Steady-State RAM: < 12 MB)                                       │
│                                                                                                  │
│  ┌──────────────────────┐   ┌──────────────────────────────┐   ┌──────────────────────────────┐  │
│  │   Native Input Bus   │   │     State & Settings Core    │   │      System Tray Controller  │  │
│  │  - Win32 Raw Input   │──►│   - rkyv Zero-Copy State     │   │   - muda / tray-icon         │  │
│  │  - macOS CGEventTap  │   │   - Lock-free ArcSwap Config │   │   - Context menu & Hotkeys   │  │
│  │  - Linux evdev/wlr   │   │   - SQLite WAL / RON Storage │   │   - Spawn/Kill UI on demand  │  │
│  │  - UIPI Fallback     │   │   - Seqlock Multi-Thread Sync│   │   - Single-instance mutex    │  │
│  └──────────────────────┘   └──────────────┬───────────────┘   └──────────────────────────────┘  │
│                                            │                                                     │
│                                            ▼                                                     │
│  ┌────────────────────────────────────────────────────────────────────────────────────────────┐  │
│  │                             GPU COMPUTE & RENDER PIPELINE (wgpu)                           │  │
│  │                                                                                            │  │
│  │  [Stage 1: Input Upload] ──► [Stage 2: WGSL Compute Pass] ──► [Stage 3: Graphics Render]   │  │
│  │   48-byte Uniform Packet      - Spring-Damper Chain (240Hz)   - Instanced SDF Billboards   │  │
│  │   (Mouse, Delta, Buttons)     - Catmull-Rom Spline Curve      - Ribbon Triangle Strips     │  │
│  │                               - Normal Vector Stabilization   - Pre-Multiplied Alpha Pass  │  │
│  │                               - 2,048 Kinematic Particles     - Swapchain Presentation     │  │
│  └─────────────────────────────────────────┬──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────┼─────────────────────────────────────────────────────┘
                                             │
                                             ▼
                             [ HARDWARE DISPLAY SURFACE PRESENT ]
                       - Windows 10/11: D3D12 Flip-Discard Swapchain
                       - macOS: Metal CAMetalLayer (presentsWithTransaction = false)
                       - Linux: Vulkan Surface / wlr-layer-shell overlay
```

---

## 2. Forensic Lessons Learned from Historical & Existing Projects

By analyzing the production history of `D3D_CURSOR` (C++20 Direct3D 11) and `FXCursor` (Rust wgpu V3), V4 resolves all known edge cases:

| Failure Mode / Edge Case                | Cause in Earlier Architectures                                                                                                                                                       | Solution Implemented in V4                                                                                                                                                            |
| :-------------------------------------- | :----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **1. UIPI Elevated Window Freeze**      | In Windows, `WH_MOUSE_LL` and `WM_INPUT` are blocked by User Interface Privilege Isolation when an Administrator window is in focus. Infinite waits caused trails to freeze mid-air. | **Bounded Sleep + Fallback Polling:** Kernel wait timeout is capped at 15ms with automatic fallback to unprivileged `GetCursorPos` queries.                                           |
| **2. Wake-Token Race Condition**        | Clearing the wake token _before_ draining the message queue allowed a hook message arriving in-between to be ignored, parking the thread indefinitely.                               | **Post-Drain Token Atomic Swap:** The wake token is reset atomically immediately before the kernel park call (`MsgWaitForMultipleObjectsEx`).                                         |
| **3. Multi-Monitor Negative Coords**    | Secondary monitors placed to the left or top of the primary monitor have negative desktop coordinates (`SM_XVIRTUALSCREEN < 0`), breaking standard $[0, 1]$ projection math.         | **Virtual Space Orthographic Matrix:** Compute and Vertex shaders apply an offset-aware transform: $x_{\text{ndc}} = \frac{x - x_{\text{origin}}}{w_{\text{virt}}} \cdot 2.0 - 1.0$.  |
| **4. Physics Explosion on Frame Stall** | During UAC prompts or window drag pauses, frame $\Delta t$ spiked to 500ms+, causing explicit Euler spring equations to overflow to `NaN` / infinity.                                | **Sub-Stepping with Max Delta Clamp:** Fixed 240Hz physics tick ($dt = 4.16\text{ms}$) with $\Delta t_{\text{max}} = 33\text{ms}$ clamp to make explosions mathematically impossible. |
| **5. Ribbon Mesh Twisting (Bowties)**   | Sharp 180° hairpin turns cause instantaneous normal vector sign inversions, generating self-intersecting degenerate triangles.                                                       | **Normal Stabilization Pass:** Step-by-step dot-product evaluation in compute shader: if $\vec{n}_i \cdot \vec{n}_{i-1} < 0$, flip $\vec{n}_i = -\vec{n}_i$.                          |
| **6. NVIDIA Vulkan Transparency Bug**   | NVIDIA drivers wrap Vulkan desktop surfaces in opaque DWM swapchains unless forced via driver profiles.                                                                              | **Direct3D 12 Default on Windows:** Bypasses Vulkan DWM quirks completely on Windows, while keeping NVAPI profile automation as a fallback.                                           |
| **7. Multi-Buffer Ghost Trails**        | Flip-discard swapchains retain old frame geometry in inactive backbuffers when rendering stops.                                                                                      | **Clean Flush State Machine:** Renders exactly $N = \text{BufferCount}$ clear frames $[0, 0, 0, 0]$ before entering deep sleep.                                                       |
| **8. Multi-Touch Finger Slot Swapping** | Lifting fingers from a digitizer shifted array indices, causing trails to jump erratically to other fingers.                                                                         | **Dedicated 32-Slot State Tracker:** Each touch ID is mapped to a static slot with independent exponential opacity decay (`fadeAlpha`).                                               |

---

## 3. The Seven Pillars of the Hardened V4 Architecture

### Pillar 1: Decoupled Micro-Daemon + SolidJS 2.0 Transient UI

- **Core Daemon:** A pure Rust binary (`fxcursor-daemon`) with **zero webview dependencies**. Steady-state memory footprint is **< 12 MB**.
- **SolidJS 2.0 + Modern Native CSS Frontend:**
  - **Zero Virtual DOM:** Signals bind directly to DOM attributes, allowing continuous 120Hz slider dragging without component re-renders or GC pauses.
  - **Modern Native CSS:** Eliminates Tailwind build-time post-processors. Uses CSS Nesting, OKLCH color spaces, and Dynamic CSS Variables (`var(--accent-color)`).
  - **Transient Lifecycle:** The UI is launched on demand and **completely killed (`process::exit`)** on window close, releasing all memory back to the OS.

---

### Pillar 2: GPU Compute-Driven Physics & Spline Evaluation

In V1–V3, the CPU computed splines and physics on every frame. In V4, **100% of physics runs inside a WGSL Compute Shader (`@compute`)**:

```rust
#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct FrameUniforms {
    pub virtual_rect: [f32; 4], // [x_origin, y_origin, total_width, total_height]
    pub cursor_pos: [f32; 2],
    pub cursor_vel: [f32; 2],
    pub buttons_mask: u32,
    pub delta_time: f32,
    pub total_time: f32,
    pub pad: u32,
}
```

#### WGSL Compute Pass Pipeline:

1. **Pass 1: Spring-Mass Chain Kinematics (`physics.wgsl`):**
   Evaluates spring forces and damping across 100 physics nodes in parallel workgroups.
2. **Pass 2: Catmull-Rom Spline & Normal Stabilization:**
   Upsamples discrete nodes into continuous ribbons, evaluates curvature density, and stabilizes normal vectors.
3. **Pass 3: Kinematic Particle Simulation:**
   Updates 2,048 particle billboards (air drag, gravity vectors, bounce reflections, alpha decay).
4. **Direct Storage Buffer Binding:** The render pass binds the compute storage buffer directly as its vertex buffer—**zero CPU-to-GPU data roundtrips**.

---

### Pillar 3: Multi-Monitor Virtual Coordinate Projection Matrix

To support multi-monitor setups with mixed DPI scales and negative virtual coordinates:

$$\begin{bmatrix} X_{\text{ndc}} \\ Y_{\text{ndc}} \end{bmatrix} = \begin{bmatrix} \frac{2.0}{W_{\text{virtual}}} & 0 \\ 0 & -\frac{2.0}{H_{\text{virtual}}} \end{bmatrix} \begin{bmatrix} X_{\text{world}} - X_{\text{origin}} \\ Y_{\text{world}} - Y_{\text{origin}} \end{bmatrix} + \begin{bmatrix} -1.0 \\ 1.0 \end{bmatrix}$$

- Automatically recalculates on `WM_DISPLAYCHANGE` (Windows), `NSApplicationDidChangeScreenParametersNotification` (macOS), or `wl_output` events (Linux) without crashing or dropping surfaces.

---

### Pillar 4: Zero-Overhead Hardware Event Sleep & UIPI Fallback

- **Hardware Interrupt Sleep:**
  - Windows: `MsgWaitForMultipleObjectsEx(..., MWMO_INPUTAVAILABLE)` tied to `WM_INPUT`.
  - macOS: `CGEventTap` on a dedicated Mach runloop.
  - Linux: `epoll` monitoring `/dev/input/event*` or Wayland seat.
- **UIPI Fallback:** If an elevated window is active on Windows, the sleep timeout automatically caps at 15ms with `GetCursorPos` polling fallback, guaranteeing zero trail freezing.
- **Result:** **0.00% CPU and 0.00% GPU utilization when idle.**

---

### Pillar 5: Deterministic Sub-Stepping Physics & Normal Stabilization

- **Fixed Physics Sub-Ticks:** Integrates at a constant 240Hz ($dt = 4.16\text{ms}$) with an accumulator loop.
- **Delta Clamp:** $\Delta t$ is capped at $33\text{ms}$ to prevent spring divergence during OS hangs.
- **Normal Vector Dot-Product Flip Pass:**
  ```wgsl
  if (dot(current_normal, prev_normal) < 0.0) {
      current_normal = -current_normal;
  }
  ```

---

### Pillar 6: Triple-Platform Direct Surface Compositing

- **Windows 10/11:** Native Direct3D 12 swapchain with `DXGI_SWAP_EFFECT_FLIP_DISCARD` + `DXGI_ALPHA_MODE_PREMULTIPLIED` over a borderless window with `WS_EX_NOREDIRECTIONBITMAP`.
- **macOS:** `CAMetalLayer` (`presentsWithTransaction = false`) attached to an `NSWindow` (`level = .screenSaver`, `ignoresMouseEvents = true`).
- **Linux (Wayland):** `wlr-layer-shell` (`ZWLR_LAYER_SHELL_V1_LAYER_OVERLAY`) with Vulkan 1.3 presentation.
- **Linux (X11):** `XCompositeGetOverlayWindow` with 32-bit ARGB visual.

---

### Pillar 7: Multi-Buffer Flush Engine & Ghost Elimination

When cursor movement ceases, the engine transitions through a 3-state lifecycle:

```
[ State 0: ACTIVE ] ──(No input & particles done)──► [ State 1: FLUSHING ] ──(N clear frames)──► [ State 2: DEEP SLEEP ]
  Render effects                                      Clear backbuffers to [0,0,0,0]              0.00% CPU / 0.00% GPU
```

- Renders exactly $N = \text{BufferCount}$ clear frames to ensure all flip-discard backbuffers are emptied before the thread enters a kernel wait.

---

## 4. Zero-Copy IPC, Self-Healing State, & Production Hardening

FXCursor V4 incorporates the battle-tested resilience patterns from **[Minimalistic_App](file:///C:/Users/Z/Downloads/PROJECTS/Minimalistic_App)**:

### 1. Zero-Copy & Type-Safe IPC (`rkyv` + `specta`)

- **Transport:** Windows Named Pipe (`\\.\pipe\fxcursor-v4`) / Unix Domain Socket (`/tmp/fxcursor-v4.sock`).
- **Protocol:** Serialized with `rkyv` for 0 ns deserialization.
- **Auto-Generated Bindings:** Uses `specta` and `tauri-specta` to generate type-safe TypeScript interfaces (`bindings.ts`) automatically at compile time. Zero manual type synchronization.
- **Lock-Free State:** The daemon render loop accesses configuration snapshots via `ArcSwap<AppConfig>`, eliminating mutex locks on the hot render path.

### 2. Field-Level Self-Healing Settings Engine (`serde_path_to_error`)

- **The Problem:** Hand-editing a settings file or a botched version migration with a single invalid value (e.g. `"trail_length": "long"` instead of an integer) causes standard `serde_json::from_str` to fail for the whole document, throwing away the user's colors, hotkeys, and window preferences.
- **The Self-Healing Solution:**
  1. Parses as untyped `serde_json::Value` and merges over default values.
  2. Deserializes through `serde_path_to_error` to identify the exact JSON path of any broken value.
  3. Resets **only that specific broken field** to its default, records a repair log, and writes the healed file to disk.

### 3. Portable Execution Mode (`portable` Marker)

- When an empty file named `portable` is placed next to the executable, all configuration, databases, and logs automatically redirect to `<exe_dir>/Data/` instead of OS app data directories (`%APPDATA%`, `~/Library/Application Support`, `~/.config`).
- Resolved once at process start via `OnceLock<Option<PathBuf>>`. Leaves zero trace on host machines when run from USB sticks.

### 4. WebView Hardening & Accelerator Disabling

- Disables built-in browser accelerator keys (`F5`, `Ctrl+R`, `Ctrl+P`, `Ctrl+F`, `Ctrl+U`, `Ctrl+±`) at the native engine level (`ICoreWebView2Settings3::SetAreBrowserAcceleratorKeysEnabled(false)`) and DOM level (`preventDefault`).
- Prevents accidental app reloads that would erase transient state or break layout zooms.

### 5. Thread-Safe Diagnostic Crash Logging (`panic_log.rs`)

- Installs a panic hook before process start that writes panic payloads, thread IDs (e.g. `gpu-render-thread`, `mouse-hook-thread`), file names, and line numbers to a rotating log file before dying under `panic = "abort"`.

### 6. Developer Experience & Fast Linkers (`.cargo/config.toml` + `dev-fast.ts`)

- Uses `lld-link` (Windows) and `mold` (Linux) via `scripts/dev-fast.ts` with `CARGO_PROFILE_DEV_DEBUG=limited` to achieve **2.6× faster incremental dev builds**.
- Automated dual-ecosystem upgrade pipeline (`bun run update-deps`) for synchronized NPM + Crates.io upgrades.

---

## 5. Complete Module & Directory Structure

```
FXCursor_V4/
├── Cargo.toml                      # Workspace manifest (LTO, opt-level=3, panic=abort)
├── crates/
│   ├── fxcursor-daemon/            # Pure native background daemon (< 12 MB RAM)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs             # Daemon entry, single-instance lock, tray init
│   │       ├── input/              # Platform-native event taps
│   │       │   ├── mod.rs          # Unified InputEvent stream
│   │       │   ├── windows.rs      # Win32 Raw Input sink (WM_INPUT) + UIPI fallback
│   │       │   ├── macos.rs        # macOS CGEventTap Mach runloop
│   │       │   └── linux.rs        # Linux evdev / wayland seat
│   │       ├── gpu/                # wgpu compute + render pipelines
│   │       │   ├── mod.rs          # GPU context & swapchain manager
│   │       │   ├── compute.rs      # WGSL compute pass (physics, splines, particles)
│   │       │   ├── renderer.rs     # WGSL render pass (instanced SDF & ribbons)
│   │       │   └── shaders/
│   │       │       ├── physics.wgsl# Compute shader (mass-damper + particles)
│   │       │       └── render.wgsl # Vertex & fragment shader (SDF + blend)
│   │       ├── overlay/            # Platform transparent click-through windows
│   │       │   ├── mod.rs
│   │       │   ├── windows.rs      # D3D12 / WS_EX_NOREDIRECTIONBITMAP window styles
│   │       │   ├── macos.rs        # NSWindow + CAMetalLayer
│   │       │   └── linux.rs        # wlr-layer-shell / X11 overlay
│   │       ├── ipc/                # rkyv IPC server (named pipe / domain socket)
│   │       └── state.rs            # Lock-free ArcSwap configuration container
│   │
│   ├── fxcursor-protocol/          # Shared rkyv types & schema definitions
│   │   ├── Cargo.toml
│   │   └── src/
│   │       └── lib.rs              # Configuration structs & validation
│   │
│   └── fxcursor/            # Settings UI Application (SolidJS 2.0 + Native CSS)
│       ├── Cargo.toml
│       ├── src-tauri/              # Thin IPC client connecting to daemon
│       └── src/                    # SolidJS 2.0 + Modern Native CSS Frontend
│           ├── App.tsx             # Reactive Settings Dashboard (Signals)
│           ├── styles/             # Modern Native CSS (Nesting, Variables, OKLCH)
│           └── components/         # Reactive sliders, knobs, color pickers
```

---

## 6. Detailed Phased Migration Roadmap

```
┌──────────────────────────────────────────────────────────────────────────────────────┐
│                               PHASED V4 MIGRATION PLAN                               │
├──────────────────────────────────────────────────────────────────────────────────────┤
│ Phase 1: Decouple Core Daemon & Zero-Copy IPC                                        │
│          - Extract overlay and wgpu renderer into standalone headless crate.          │
│          - Implement named pipe / domain socket IPC with rkyv.                       │
│                                                                                      │
│ Phase 2: Compute Shader Physics & Sub-Stepping Migration                             │
│          - Implement physics.wgsl compute pass (Spring-Damper, Splines, Particles).  │
│          - Add 240Hz sub-stepping accumulator and normal stabilization pass.         │
│                                                                                      │
│ Phase 3: Platform Native Event Taps & Flush State Machine                            │
│          - Replace device_query polling with Win32 Raw Input and macOS CGEventTap.   │
│          - Wire MsgWaitForMultipleObjectsEx / Mach port wait for 0.00% idle sleep.   │
│          - Implement N-frame clean flush engine for flip-discard swapchains.          │
│                                                                                      │
│ Phase 4: SolidJS 2.0 + Native CSS Dashboard                                          │
│          - Build reactive SolidJS 2.0 UI connecting to daemon via IPC.               │
│          - Style with native CSS (nesting, OKLCH colors, CSS custom properties).     │
│          - Wire tray "Configure" action to spawn/close UI on demand.                 │
└──────────────────────────────────────────────────────────────────────────────────────┘
```

---

## 7. Target Performance Benchmark Table

| Metric                             |  V3 (Current Architecture)  |      V4 (Hardened Specification)      |            Improvement             |
| :--------------------------------- | :-------------------------: | :-----------------------------------: | :--------------------------------: |
| **Steady-State RAM (Idle)**        |          80–120 MB          |              **< 12 MB**              |         **~88% Reduction**         |
| **Idle CPU Utilization**           |       ~0.5% (Polling)       |    **0.00% (Kernel Event Sleep)**     |         **Zero Watt Idle**         |
| **Active CPU Utilization (240Hz)** |     ~2.5% (CPU Physics)     |       **< 0.05% (GPU Compute)**       |         **~98% Reduction**         |
| **PCIe Bandwidth per Frame**       |  ~250 KB (Vertex Uploads)   |     **48 Bytes (Uniform Buffer)**     |        **~99.9% Reduction**        |
| **Input Capture Latency**          | ~4–8 ms (Polling interval)  |   **< 0.5 ms (Hardware Interrupt)**   | **Sub-Pixel Microsecond Response** |
| **Binary Size (Daemon)**           |           ~18 MB            |               **~6 MB**               |         **~66% Reduction**         |
| **Cross-Platform Compatibility**   |   Win / Mac / Linux (X11)   | **Win / Mac / Linux (X11 + Wayland)** |   **Complete Native OS Support**   |
| **Elevated Window Handling**       |    Can stall on Windows     |       **UIPI Fallback Polling**       |      **Zero Trail Freezing**       |
| **Multi-Monitor Projection**       | Can clip on negative coords |      **Virtual Desktop Matrix**       |  **Flawless Multi-DPI Spanning**   |
