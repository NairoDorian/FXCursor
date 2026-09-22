# Changelog

## [Unreleased] - 2026-09-17

### Fixed

- **Reactive Toast State Synchronization**: Fixed inverted notification state in `src/App.tsx` when toggling effects with `Ctrl + E` by binding directly to the calculated next state.
- **Vite Bundle Cleanliness**: Replaced dynamic module imports in `src/lib/console.ts` with static imports, eliminating `[INEFFECTIVE_DYNAMIC_IMPORT]` warnings and optimizing chunking.
- **GPU Device Texture Dimension Clamping**: Clamped depth texture dimensions in `crates/fxcursor-render/src/renderer.rs` against `device.limits().max_texture_dimension_2d` to prevent driver panics on ultra-wide multi-monitor configurations.
- **Daemon Named Pipe Disconnect Safety**: Added graceful disconnect cleanup on handle clone errors in `crates/fxcursor-daemon/src/ipc/mod.rs`.
- **Theme Accent Token Parity**: Replaced legacy variable reference with `var(--accent-primary)` in `src/components/Tabs/HotkeysTab.tsx`.
- **Named Capture Worker Threads**: Explicitly named capture worker threads `"capture-png-worker"` in `src-tauri/src/overlay/mod.rs` for enhanced observability.
- **Daemon IPC Deadlock**: resolved re-entrant `state.config.write()` lock acquisition in `IpcRequest::ToggleEnabled` (`crates/fxcursor-daemon/src/ipc/mod.rs`), enabling all 5 daemon unit tests to pass cleanly without freezing workspace test execution.
- **Windows Hook Cleanup**: added explicit `UnhookWindowsHookEx(hook)` on message loop exit in `src-tauri/src/input/windows.rs`.
- **Atomic Config & User Preset Persistence**: upgraded atomic file saving in `src-tauri/src/settings_repair.rs`, `src-tauri/src/user_presets.rs`, and `crates/fxcursor-daemon/src/state.rs` using unique PID/counter temporary file paths and automatic cleanup on failure.
- **Tray Icon Initialization Safety**: replaced unchecked `app.default_window_icon().unwrap()` in `src-tauri/src/lib.rs` with safe optional attachment, preventing panics in environments without packaged window icons.
- **Snapshot Capture Speed Optimization**: configured `png::Compression::Fast` in `src-tauri/src/capture.rs` to accelerate burst frame captures and eliminate disk write bottlenecks during transient effect inspection.
- **Depth Target Surface Clamping**: clamped width and height at entry of `ensure_depth` in `crates/fxcursor-render/src/renderer.rs` to prevent redundant depth texture re-creations on zero-dimension surface events.
- **Timer Resolution Lifecycle**: added `restore_timer_resolution` (`timeEndPeriod(1)`) and an RAII `TimerResolutionGuard` in `src-tauri/src/overlay/mod.rs`.
- **Panic Logging Path**: directed `panic.log` to the resolved application data directory (or portable `Data/` folder) with CWD fallback in `src-tauri/src/panic_log.rs`.
- **Version Manifest Tooling**: added `crates/fxcursor-render/Cargo.toml` to the version check and bump targets in `scripts/before-commit.ts`.

### Added

- **Dev Console Debug Filter**: added `debug` level button to log filters in `src/components/Tabs/DevConsoleTab.tsx`.
- **Full In-Window Shortcut Navigation**: added `Ctrl + /` (and `Ctrl + ?`) shortcut in `src/App.tsx` for immediate navigation to the About tab, and documented all 11 tab shortcuts in `src/components/Tabs/HotkeysTab.tsx`.
- **W3C ARIA Accessibility**: added `role="tablist"`, `role="tab"`, `aria-selected`, and `aria-label` attributes to the Studio tab bar and master effect toggle switch in `src/App.tsx`.
- **Unit Test Coverage Expansion**: added unit tests for `CapturedFrame::write_png` (verifying PNG magic header `0x89504E47`), completely invalid JSON syntax self-healing, and array length mismatch self-healing across the test suites (52 tests passed total).
- **Exhaustive Code Documentation**: comprehensive docstrings across all `AppConfig` structs in `crates/fxcursor-protocol`, `OverlayRenderer` architecture in `crates/fxcursor-render`, and daemon state in `crates/fxcursor-daemon`.

### Changed

- **Rust edition 2024** across the workspace (rustc 1.98.1): `cargo fix --edition` migration, nested `if let`s rewritten as let-chains, clippy clean under the new edition.
- Dependencies at the newest pre-releases: SolidJS 2.0.0-rc.7 (cleanups are now returned from `onSettled` / effects instead of `onCleanup`), Vite 8.3 beta, TypeScript 7.1 dev, oxlint 1.82, bun-types canary; transitive crates refreshed. The daemon moved to winit 0.31.0-beta.3 (trait-object `Window` / `ActiveEventLoop`, `can_create_surfaces`, `SurfaceResized`). The unused `windows-core` and `libc` dependencies were dropped.

## [0.5.0] - 2026-09-09

### Added (FXCursor / V4)

- **Configuration persistence**: `config.json` in the OS app-config directory (or portable `Data/`), loaded through the field-level self-healing deserializer (`serde_path_to_error`), corrupted files backed up to `.json.bak`, atomic temp-file writes, and a 400 ms debounced autosave thread fed by every `update_config`.
- New IPC commands: `save_config`, `list_presets`, `apply_preset`, `import_config`; `get_diagnostics` now returns `config_path`.
- Tauri 2 capability file `src-tauri/capabilities/default.json` (`core:default`, event and window permissions for `main` and `overlay`).
- **Global toggle hotkey** via `tauri-plugin-global-shortcut`, registered from `general.global_hotkey` and re-registered on change; empty string disables.
- **Login autostart** via `tauri-plugin-autostart` (`general.autostart`), launching with `--minimized`.
- `general.minimize_to_tray` and `general.start_minimized` are now honoured by the window close handler and at startup.
- Renderer: counter-rotating **dual satellite ring** (`satellites.dual_ring`), rainbow `saturation`/`lightness` applied, frame-rate-independent rainbow speed.
- Render loop **idle skip**: after three settle frames with no input, config or animation change, no GPU work is submitted.
- Frontend: presets fetched from Rust (single source of truth), reset/import delegate to the backend, theme accent switcher fixed (`--accent-primary` tokens), Developer Hub shows the config path, HotkeysTab documents the shortcut format.
- Tests: `settings_repair` round-trip, healing and missing-file tests.
- **Native Windows input**: `WH_MOUSE_LL` low-level mouse hook on a dedicated message thread feeding a shared `InputHub` (position, button state, queued click events). Clicks shorter than a frame are never lost; `GetCursorPos` fallback covers elevated-window (UIPI) focus. Non-Windows platforms feed the same hub from `device_query`.
- **Event-driven render loop**: after three settle frames the render thread parks on a condvar (100 ms bounded) and is woken by hook events or configuration commits; no busy polling while idle.
- Renderer: explicit `spawn_click` API, particle/ripple caps matching GPU buffer sizes.
- **Typed IPC bindings**: `tauri-specta` exports `src/lib/bindings.ts` (typed `commands` + all config types, floats as `number`) on every debug run; frontend migrated from `invoke` strings to `commands.*`; hand-written TypeScript config interfaces removed.
- **Default-config parity**: `bun run fixtures` snapshots `AppConfig::default()`; `crates/fxcursor-protocol/tests/fixture_parity.rs` and `test/config-parity.test.ts` both assert against it.
- **Backend log bridge**: Rust `log` records are retained (300 entries) and streamed to the Dev Console via the `rust-log` event and `get_recent_logs`; rows carry a rust/web badge.
- `get_diagnostics` now reports GPU adapter/backend/device type, input backend, uptime and build type; About tab and Developer Hub show live data; Developer Hub gains "Save Configuration Now" and a Refresh button.
- Tooling: `bun run before-commit` recognises workspace-inherited crate versions; generated bindings excluded from lint/format.
- **Shared renderer crate** `crates/fxcursor-render`: the renderer, `render.wgsl` and the prototype `physics.wgsl` moved out of `src-tauri`; the daemon's duplicated renderer was deleted and it now depends on the shared crate. Unit tests for the mode mask, fade curves and spline.
- **Effect modes** (`effect_mode`) implemented: `ModeMask` gates trail layers, head, ripples, particles and satellites across physics, click spawning, animation detection and rendering. Full, Ribbon only, Click effects only, Satellites only, Minimal (core + spine). Selector in the Studio header.
- **Render-loop telemetry** (`fps_counter`): the render thread publishes `FrameStats` (fps, CPU ms per frame, active/settling/idle, ribbon vertex and SDF instance counts); `get_diagnostics` exposes it and the Developer Hub polls at `refresh_rate_ms` with an on/off switch and rate slider.
- **Live preview parity**: the canvas preview now mirrors the Rust physics (spring/damping scaling, adaptive subdivision, fade curves, gradient/rainbow, velocity width/alpha, min width), draws ripples and particles (click the canvas), the dual satellite ring and honours the effect mode.
- Theme accent tokens replace every hardcoded cyan in components.
- Bundling enabled (`nsis`, current-user install), `bun run package:portable` builds the portable zip, and `.github/workflows/ci.yml` runs typecheck, lint, tests, build, clippy, cargo tests, fixture freshness and a Windows release build that uploads the installer and the portable zip.
- Effect-mode parity fixture (`test/fixtures/mode_masks.json`) asserted from Rust and Bun; the TypeScript `modeMask` lives in `src/lib/effectMode.ts`.
- **Ribbon rendering rewrite (capsule union)**: the trail was a chain of flat quads with per-sample normals plus separate end caps; at sharp corners the inner edge folded, quads overlapped and translucent layers double-blended, which showed as bright blotches, dark notches and "disconnected" segments. Each layer is now one tapered round capsule per sample pair, drawn instanced with an analytic distance field, and the union is resolved on the GPU with a depth-based max-coverage pre-pass (`Greater` write, then `GreaterEqual` colour pass with a 1e-4 epsilon). Joins and caps are round by construction, hairpins never fold, overlaps never double-blend, and normal-flip stabilisation / 16-step caps are gone. Near-coincident spring nodes are merged before splining and the centerline now starts at the real cursor so the front cap stays attached to the pointer. The live preview mirrors this with an erase-then-paint union on an offscreen canvas.
- **Overlay snapshots**: `capture_overlay(width, height)` IPC command, `--capture <file> [--capture-size WxH]` CLI flag (handled by the running instance through the single-instance hook) and a Developer Hub section render the current frame offscreen and write a PNG composited over dark grey. GDI screen capture cannot see the Vulkan/flip-model overlay, so this is the only reliable way to inspect it; it was used to verify the new ribbon through zig-zags, hairpins and loops.
- **Frame pacing & display fitting** (session 5): `src-tauri/src/display.rs` reads the virtual-desktop bounds, primary scale factor and display refresh rate; the overlay is created and refitted in physical pixels (it was 2× too large on HiDPI); the render loop paces to the display refresh or the new `general.max_fps` cap (Performance card); bounds are polled every second. Curvature-adaptive spline sampling and capsule culling cut a full trail from ~1400 to ~500 capsules. Strict CSP with a logged Studio handshake, hotkey status line, MODIFIED badge on presets, daemon pipe double-close fixed, physics sub-stepped at 1/120 s, D3D_CURSOR-style evenly spaced particle bursts.
- **Trail head physics** (session 6): the spring chain became `TrailChain` in `crates/fxcursor-render` — a jitter-filtered pointer follower head, `trail.lead_nodes` pursuit followers (new setting, default 4, slider in Trail Physics), the spring-damper body, and a **no-overtake rule** that keeps every node behind its predecessor (the raw pointer for the head). The loops, stubs and hooks that formed around the cursor at stops, reversals and sharp turns are gone; centripetal Catmull-Rom with phantom endpoints replaced the uniform spline. Frame deltas up to 100 ms are integrated in full (16 sub-steps). Six new GPU-free physics tests (stop, reversal, lead-node monotonicity, frame-rate independence, growth, wall clamping); the live preview mirrors the new chain.
- **Burst snapshots**: `--capture-burst N --capture-interval ms` writes `<stem>_NN.png` at a fixed cadence with PNG encoding on a worker thread (a synchronous capture stalled the very physics it recorded); each capture logs a chain summary. `scripts/snapshots/` holds the PowerShell drivers (trail shapes, stop/reverse/turn bursts, contact and zoom sheets) that verified the new physics frame by frame.
- **On-overlay FPS HUD**: `fps_counter.enabled` draws the presented frame rate on the overlay with a 3×5 bitmap font (instanced SDF quads), anchored by `align_right` / `align_bottom` and refreshed every `refresh_rate_ms`.
- **Custom presets**: `save_user_preset` / `delete_user_preset` store named presets as JSON next to `config.json`; `list_presets` merges them with the built-ins and the Presets tab can save, apply and delete them.
- **In-window shortcuts**: `Ctrl+S` saves, `Ctrl+E` toggles the effects, `Ctrl+1…9` switch tabs; the Hotkeys tab lists them. The tray tooltip shows the live state (effects on/off, active preset).
- Windows render thread runs at above-normal priority.
- `fxcursor --apply <json>`: a second process merges a partial configuration into the running app (deep merge + self-healing, persisted and broadcast) — used by the snapshot tooling to flip settings such as the HUD without the Studio.
- Test totals: 44 Rust, 17 Bun.

### Changed

- **Renamed FXCursor** (was CursorFX Studio): crates `fxcursor-protocol` / `fxcursor-render` / `fxcursor-daemon`, binary `fxcursor`, product name `FXCursor`, identifier `com.nairodorian.fxcursor` — configurations from the earlier `com.fxcursor.app` and `com.cursorfx.studio` identifiers are adopted automatically on first launch. The repository was restructured so the app is the root; the V3 React app, the original Windhawk mods and the old build scripts moved to `legacy/`; CI, scripts and docs updated for the new layout. New home: `https://github.com/NairoDorian/FXCursor`.

- All Markdown documentation rewritten to describe the implemented architecture (single-process Tauri app, CPU physics, experimental daemon) instead of the V4 specification targets; `ARCHITECTURE.md` regenerated with accurate per-file descriptions.
- `scripts/before-commit.ts` label corrected to the actual 7 gates.
- `is_animating` returns `false` when effects are disabled and tracks head squish settling.

### Known gaps (tracked in `PROGRESS.md`)

- macOS/Linux input still polls via `device_query`; only Windows has the OS hook.
- `crates/fxcursor-daemon` is a Windows-only prototype with no client; `physics.wgsl` is not dispatched.
- `fps_counter.align_*` corner placement configuration is currently fixed to the primary display; interactive corner switching in the UI is planned.
- Automated GitHub Actions CI workflow triggers on repository pushes and pull requests.

## [0.4.0] - 2026-08-30

### FXCursor V4 Next-Gen Master Architecture

- **SolidJS 2.0 Migration**: Upgraded frontend to `solid-js 2.0.0-rc.4`, replacing React/egui with a lightweight reactive UI graph, `onSettled` startup lifecycle, two-argument `createEffect(computeFn, effectFn)`, and `<Errored>` diagnostics boundary.
- **wgpu 30 Hardware Engine**: Full migration to `wgpu 30.0.1` (`SurfaceTargetUnsafe::from_display_and_window`, `queue.present(frame)`) with native Direct3D 12 / Vulkan (Windows), Metal 3 (macOS), and Vulkan 1.3 (Linux).
- **4-Layer Master Design**: Faithfully implemented the 4-layer master visual design (Outer Glow 150%, Mid Shadow 90%, Crisp Core 50%, Inner Spine 15%) with 16-step rounded semicircle end-caps and normal vector flip stabilization ($\vec{n}_i \cdot \vec{n}_{i-1} < 0$).
- **Instant Frame Clearing**: Implemented `clear_active_state()` and `render_clear()` to instantly wipe the desktop overlay clean whenever effects are toggled off.
- **Real-Time Reactive Parameter Tuning**: Adjusting any slider or toggle updates the backend configuration immediately, updating the rendering loop on the very next frame.
- **System Tray & Window Interception**: Left-click tray icon toggles GUI dashboard; close button is intercepted with `api.prevent_close()` + `window.hide()`; right-click menu provides Show Settings, Toggle Effects, and Quit.
- **Crash Elimination**: Removed dangerous `GWLP_WNDPROC` recursion hooks and replaced them with native Tauri 2 `window.set_ignore_cursor_events(true)`.
- **Dual-Ecosystem Automated Updater**: Added `scripts/update-deps.ts` probing NPM pre-release tags and Crates.io `newest_version`.
- **Protocol crate**: shared `AppConfig`, six built-in presets, self-healing deserializer.
- **Daemon prototype**: `crates/fxcursor-daemon` standalone winit renderer with JSON named-pipe server (experimental).

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
