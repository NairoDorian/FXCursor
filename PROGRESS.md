# FXCursor V4 — Project Progress, Status & Roadmap

> **Milestone Status**: 🟡 Under construction — Tauri single-process path works end to end on Windows; V4 "micro-daemon" architecture is a prototype with stabilized IPC. Repository restructured and renamed **FXCursor** (app at the root, V3 under `legacy/`).
> **Version**: `0.5.0` (pre-release)
> **Last audit**: 2026-09-23, session 11 (full audit: trail brought back to the legacy algorithm — node-index taper, resting dot, `/20` speed normalisation, Windhawk squishy head; vsync-locked Fifo pacing; GPU-cursor P0 fixed; config range validation; Rust → TS trail/preset parity fixtures; 78 Cargo tests + 27 Bun tests; baseline `089e5d8`, nothing committed since)
> **Tech Stack**: Tauri 2.11, Bun 1.4, SolidJS 2.0.0-rc.7, TypeScript 7.1-dev, Vite 8.3 beta, wgpu 30, windows 0.62, Rust 2024 edition (rustc 1.98.1)

> [!CRITICAL]
> **Primary Standards & Rules** (see `AGENTS.md`):
>
> 1. **Package Manager**: NEVER use npm/yarn/pnpm. **ALWAYS use Bun**.
> 2. **Testing Command**: the only dev launch command is `bun run tauri dev` (run from the repository root).
> 3. **Always Pre-Release**: keep all dependencies at the newest pre-release (`bun run update-deps`).
> 4. **RTK Command Prefix**: prefix shell commands with `rtk` (`rtk git status`, `rtk cargo check --workspace`).

---

## 1. What the app is today (honest summary)

FXCursor V4 is a **Tauri 2 desktop app** that renders GPU cursor effects on a transparent, click-through overlay window and exposes a SolidJS 2 settings dashboard. In this repository "V4" currently means:

| Area           | Implemented today                                                                                                                                                                                                                                                                                                                                                | Spec target (`docs/V4_ARCHITECTURE_SPECIFICATION.md`)                          |
| :------------- | :--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :----------------------------------------------------------------------------- |
| Process model  | **Single process**: Tauri shell + overlay webview window + `gpu-render-thread` with a wgpu 30 surface on that window                                                                                                                                                                                                                                             | Headless `fxcursor-daemon` (< 12 MB) + transient Studio UI over named-pipe IPC |
| Physics        | **CPU** `TrailChain`: optional LazyBrush dead-zone pointer filter → Windhawk spring-damper head + body (0.3 second-neighbour coupling) on a **1/120 s reference frame** + **512 px inelastic teleport-only** clamp, integrated in equal sub-steps of at most 1/120 s; **width / fade / blur follow the original node index** (as in Windhawk, V3 and TD: the trail retracts into the cursor after a stop), a collapsed chain draws a 4-layer resting dot, velocity boost normalised by 20 px per 1/120 s; centripetal Catmull-Rom with phantom endpoints and curvature-adaptive sampling; head wins depth ties at folds. Mirrored in TypeScript by `src/lib/trail.ts` and checked against a Rust reference trace. **Defaults are trail-only** | WGSL compute pass at 240 Hz sub-steps                                          |
| Rendering      | **GPU**: 4-layer ribbon (Outer Glow 150 %, Mid Shadow 90 %, Crisp Core 50 %, Inner Spine 15 %) as a GPU-resolved union of tapered round capsules (depth max-coverage pre-pass: round joins/caps, no folding, no double blending), fwidth anti-aliasing, pre-multiplied alpha; instanced SDF head / ripples / particles / satellites (dual counter-rotating ring) | Same visuals, compute-fed vertex buffers                                       |
| Input          | **Windows: `WH_MOUSE_LL` low-level hook** on a message thread feeding an `InputHub` (position, buttons, queued clicks) with condvar wake; `GetCursorPos` fallback for UIPI. Other platforms: `device_query` polling into the same hub                                                                                                                            | Win32 Raw Input + kernel wait, CGEventTap, evdev                               |
| Idle behaviour | Render thread **parks on a condvar** (100 ms bounded) once 3 settle frames are presented; woken by hook events or config commits. No GPU work while idle                                                                                                                                                                                                         | 0.00 % CPU/GPU kernel sleep                                                    |
| Persistence    | `config.json` in the OS app-config dir (or portable `Data/`), **field-level self-healing** load, atomic save, **debounced autosave** on every change, explicit `save_config`                                                                                                                                                                                     | Same (rkyv/SQLite were considered; JSON chosen)                                |
| OS integration | Tray (show / toggle / quit), **global toggle hotkey**, **login autostart**, single-instance, minimize-to-tray, start-minimized, `--minimized` flag                                                                                                                                                                                                               | Same                                                                           |
| Presets        | 6 built-ins; **Rust is the source of truth** (`list_presets`, `apply_preset`); TypeScript keeps a mirror for browser preview mode; JSON import goes through the self-healing deserializer                                                                                                                                                                        | Same                                                                           |
| Multi-monitor  | Overlay spans the Win32 virtual desktop; shaders subtract the virtual origin (negative coordinates OK). Non-Windows falls back to 1920×1080                                                                                                                                                                                                                      | Native display-change events on all three platforms                            |
| Cross-platform | Compiles for all targets; **verified on Windows only**. macOS/Linux overlay + input untested                                                                                                                                                                                                                                                                     | Win + macOS + Linux (X11 + Wayland layer-shell)                                |
| Daemon crate   | `crates/fxcursor-daemon`: Windows-only winit prototype, JSON named-pipe / Unix-socket server, **no client, not launched by the app**, `physics.wgsl` present but never dispatched                                                                                                                                                                                | Production headless renderer                                                   |
| Type sharing   | **`tauri-specta` generates `src/lib/bindings.ts`** (typed `commands` + config types) on every debug run; a Rust ⇄ TS default-config parity test guards the TS defaults mirror                                                                                                                                                                                    | Auto-generated `bindings.ts` ✅                                                |
| Effect modes   | `effect_mode` implemented as a `ModeMask` in the shared renderer (Full, Ribbon only, Click effects only, Satellites only, Minimal core+spine); selector in the Studio header; live preview mirrors it                                                                                                                                                            | —                                                                              |
| Telemetry      | `fps_counter` drives Developer Hub polling of `FrameStats` (fps, CPU ms/frame, state, vertex/instance counts) published by the render thread every 500 ms; the overlay HUD draws it with a 3×5 bitmap font, anchored by `align_right` / `align_bottom`                                                                                                                                                                             | On-overlay HUD text (future)                                                   |
| GPU cursor     | `gpu_cursor` bypass (Windows): active OS cursor shape extracted per handle, drawn last as an on-top textured quad with smoothed movement rotation (arrow only) + sine-bump click bounce; optional global arrow hide via `SetSystemCursor`, restored on disable / exit / panic. Head-tab Studio section                                                                                                                                                                                              | —                                                                              |
| Render crate   | **`crates/fxcursor-render`** holds the one renderer + shaders; both `src-tauri` and the daemon depend on it (the daemon fork is gone)                                                                                                                                                                                                                            | Same                                                                           |

---

## 2. Completed milestones

### 2.1 Graphics core (`src-tauri/src/overlay/`)

- [x] wgpu 30 surface on the Tauri overlay window (`SurfaceTargetUnsafe::from_display_and_window`), pre-multiplied alpha compositing, vsync-locked `Fifo` presentation (`Mailbox` only for a cap above the refresh rate, since session 11), surface loss recovery.
- [x] 4-layer master ribbon with per-layer width/alpha/blur, gradient and fade curves (linear / ease-out / exponential / sigmoid), velocity-driven width and alpha.
- [x] Capsule-union ribbon: one tapered round capsule per sample pair, union resolved with a depth max-coverage pre-pass (replaced the quad strip + normal flip + 16-step caps on 2026-09-09).
- [x] SDF effects: squishy head (velocity-elongated ellipse), click ripples per mouse button, kinematic particle bursts (gravity, friction), orbit satellites with optional ring and counter-rotating dual ring.
- [x] Rainbow mode honours `saturation` / `lightness` and cycles at a frame-rate-independent speed (2026-09-09).
- [x] Instant clear when effects are disabled; idle GPU skip when the frame cannot change (2026-09-09).

### 2.2 Desktop shell (`src-tauri/src/lib.rs`, `integrations.rs`, `settings_repair.rs`)

- [x] Overlay window: transparent, undecorated, always-on-top, skip-taskbar, `set_ignore_cursor_events(true)` click-through, unfocused at creation.
- [x] System tray with left-click toggle of the Studio window and context menu.
- [x] Tauri 2 capability file (`src-tauri/capabilities/default.json`) granting `core:default`, event and window permissions to both windows (2026-09-09).
- [x] Persistent configuration with self-healing repair, `.bak` of corrupted files, atomic writes and a 400 ms debounced autosave thread (2026-09-09).
- [x] Global hotkey registration from `general.global_hotkey`, re-registered on change, empty string disables (2026-09-09).
- [x] Login autostart synced from `general.autostart`; `--minimized` launch flag (2026-09-09).
- [x] `minimize_to_tray` and `start_minimized` honoured (2026-09-09).
- [x] IPC commands: `ping`, `get_diagnostics`, `get_config`, `update_config`, `toggle_overlay`, `reset_defaults`, `save_config`, `list_presets`, `apply_preset`, `import_config`.
- [x] Panic hook logging to `panic.log`; portable mode via `portable` marker file.

### 2.3 Studio dashboard (`src/`)

- [x] SolidJS 2.0 (`onSettled`, two-argument `createEffect`, `<Show>`, `<Errored>` boundary).
- [x] 11 tabs: 4-Layer Design, Trail Physics, Squishy Head, Click Ripples, Particles, Satellites, Presets, Hotkeys & Tray, Dev Console, Developer Hub, About.
- [x] Real-time sync: every edit is coalesced to one `requestAnimationFrame` and pushed via `update_config`; backend echoes `config-updated` (400 ms echo suppression).
- [x] Presets fetched from Rust when running under Tauri; import/reset delegate to the backend (2026-09-09).
- [x] Theme accent switcher fixed: `--accent-primary/--accent-glow/--accent-subtle` now drive the CSS (2026-09-09).
- [x] Developer Hub shows the live configuration file path (2026-09-09).
- [x] Live 2D-canvas preview of the ribbon (approximation, see Known Issues).

### 2.4 Input, diagnostics & type sharing (2026-09-09, session 2)

- [x] `InputHub` (`src-tauri/src/input/`): shared event-driven input state with queued click events and a condvar; the render thread parks while idle and is woken by input or `commit()`.
- [x] Windows `WH_MOUSE_LL` hook thread; button presses shorter than a frame are queued and never lost. `GetCursorPos` fallback keeps the trail alive when hooks are muted by an elevated window.
- [x] Renderer takes explicit click events (`spawn_click`) and caps particles/ripples to the GPU buffer sizes.
- [x] `tauri-specta`: all commands annotated, `src/lib/bindings.ts` regenerated on every debug run (floats exported as `number`), frontend migrated to the typed `commands` API; hand-written config interfaces deleted.
- [x] Default-config parity: `bun run fixtures` snapshots `AppConfig::default()`; Rust and Bun tests both assert against it.
- [x] Log bridge: Rust `log` records stream to the Dev Console (`rust-log` event + `get_recent_logs` history) with a rust/web badge.
- [x] `get_diagnostics` reports GPU adapter/backend, input backend, uptime, build type; About tab and Developer Hub render live data; "Save Configuration Now" button.

### 2.5 Effect modes, telemetry & shared renderer (2026-09-09, session 3)

- [x] `crates/fxcursor-render`: renderer, `render.wgsl` and the prototype `physics.wgsl` moved out of `src-tauri`; the daemon now uses the same crate (its ~900-line fork deleted). 5 unit tests (mode mask, fade curves, spline).
- [x] `effect_mode` implemented via `ModeMask::from_mode` gating trail layers, head, ripples, particles and satellites in physics, click spawning, animation detection and rendering. Header selector in the Studio.
- [x] `fps_counter` implemented as render-loop telemetry: `FrameStats` (fps, CPU ms per frame, active/settling/idle, ribbon vertex and SDF instance counts) published to `RuntimeInfo`, exposed in `get_diagnostics`, polled by the Developer Hub at `refresh_rate_ms` with an on/off switch and rate slider.
- [x] Live preview rewritten to mirror the Rust physics: same spring/damping scaling, adaptive Catmull-Rom subdivision, fade curves, gradient/rainbow colours, velocity width/alpha, min width, ripples and particles (click the canvas), dual satellite ring, effect-mode mask.
- [x] Theme accent tokens replace every hardcoded cyan in components.
- [x] `src/lib/effectMode.ts` holds the TypeScript `modeMask` + header mode list; `test/fixtures/mode_masks.json` (from `bun run fixtures`) is asserted by both `crates/fxcursor-render/tests/mode_mask_fixture.rs` and `test/effect-mode.test.ts`, so the preview cannot drift from the renderer.
- [x] Bundling enabled (`nsis`, current-user install) and a GitHub Actions workflow (`.github/workflows/ci.yml`) running typecheck, lint, tests, build, clippy, cargo tests, fixture freshness, and a Windows release build.
- [x] `scripts/package-portable.ts` builds the portable zip from the release build; CI uploads both the NSIS installer and the zip.

### 2.6 Ribbon rewrite & overlay snapshots (2026-09-09, session 4)

- [x] **Capsule-union ribbon**: the flat quad strip (per-sample normals, normal-flip fix, 16-step caps) folded and double-blended at sharp corners. Each layer is now one tapered round capsule per sample pair, instanced with an analytic SDF; the union is resolved on the GPU with a depth max-coverage pre-pass (`Greater` write, `GreaterEqual` + 1e-4 epsilon colour pass). Round joins/caps, no folding, no double blending. Verified with overlay snapshots through zig-zags, hairpins and loops.
- [x] Near-coincident spring nodes merged before splining; centerline anchored at the real cursor position (front cap no longer lags the pointer).
- [x] Live preview mirrors the union (erase-then-paint capsules on an offscreen canvas, head painted last).
- [x] **Overlay snapshots**: `capture_overlay` IPC command, `--capture <file> [--capture-size WxH]` CLI flag via the single-instance hook, Developer Hub buttons; PNG composited over dark grey. GDI capture cannot see the Vulkan overlay, so this is the inspection tool.
- [x] Renderer geometry unit tests (straight line, merged/hairpin nodes, tapering, disabled layers) and capture crop tests.

### 2.7 Tooling (`scripts/`)

- [x] `update-deps.ts` dual-ecosystem pre-release updater, `before-commit.ts` 7-gate validator, `generate-arch.ts` Repomix architecture map, `sync-docs.ts` upstream docs mirrors, `version.ts` single version source.
- [x] `scripts/snapshots/`: `snapshot_trail.ps1` (drives the pointer through hairpins / zig-zags / loops and captures), `snapshot_motion.ps1` (stop / reverse / turn transients captured as a burst), `contact_sheet.ps1` and `zoom_sheet.ps1` (flip-book sheets from burst frames). Output goes to `target/snapshots/`.

### 2.8 Pacing, display fitting & polish (2026-09-09, session 5)

- [x] `src-tauri/src/display.rs`: virtual-desktop bounds, primary scale factor, display refresh rate (`EnumDisplaySettingsW`), 1 ms timer resolution. The overlay is created and refitted with **physical** size/position (a logical-size overlay was 2× too large on HiDPI).
- [x] Render loop paces to the display refresh rate or `general.max_fps` (new setting, Performance card in Hotkeys & Tray); display bounds polled every second and the surface refitted on change.
- [x] Curvature-adaptive spline sampling (3–24 px spacing) and capsule culling (invisible / off-screen): ~500 capsules instead of ~1400 for a full trail.
- [x] Strict CSP (`csp` + `devCsp`) with a logged Studio handshake; hotkey status line (registered / disabled / error) in the Hotkeys tab; MODIFIED badge on presets; daemon pipe double-close fixed.
- [x] Physics integrated in equal sub-steps of at most 1/120 s; particle bursts spawn at evenly spaced angles with ±0.25 rad jitter and 0.7–1.3× speed (D3D_CURSOR-style).

### 2.9 Trail head physics, burst snapshots & the FXCursor rename (2026-09-09, session 6)

- [x] **`TrailChain`** (superseded by the session-9 rewrite below; kept for history): the head was a first-order follower of the jitter-filtered pointer with `trail.lead_nodes` pursuit followers and a **no-overtake rule**. Replaced in session 9 by LazyBrush + spring-damper + distance constraint.
- [x] Centripetal Catmull-Rom (α = 0.5) with phantom endpoints replaces the uniform spline: no hooks with uneven node spacing.
- [x] Tests: stop (nothing ever in front of the pointer), reversal (passed nodes stay behind), lead-node monotonicity, frame-rate independence (60 vs 240 fps), chain growth, wall clamping. 18 render-crate tests.
- [x] **Burst snapshots**: `--capture-burst N --capture-interval ms` writes `<stem>_NN.png` at a fixed cadence; PNG encoding runs on a worker thread so the capture no longer stalls the physics it records; each capture logs a one-line chain summary (`chain nodes=… first5=… farthest=…`).
- [x] Verified with 30-frame bursts at 50 ms: stop, reversal and 90° turn all keep a clean rounded tip at the pointer (`scripts/snapshots/`).
- [x] Renamed **FXCursor**: crates `fxcursor-{protocol,render,daemon}`, binary `fxcursor`, identifier `com.nairodorian.fxcursor` (the previous `com.cursorfx.studio` config is adopted on first launch), product name FXCursor, window title "FXCursor Studio". Repository restructured: the app is the root, V3 / original mods / old scripts live under `legacy/`, CI runs at the root.

### 2.10 Comprehensive audit, daemon stabilization & documentation (2026-09-17, session 7)

- [x] **Daemon Deadlock Resolved**: Fixed re-entrant `state.config.write()` write lock acquisition in `IpcRequest::ToggleEnabled` (`crates/fxcursor-daemon/src/ipc/mod.rs`), enabling all daemon unit tests to pass without stalling `cargo test --workspace`.
- [x] **Daemon Pipe Disconnect Safety**: Added graceful disconnect cleanup on handle clone errors in `crates/fxcursor-daemon/src/ipc/mod.rs`.
- [x] **Safe Atomic File Saving**: Upgraded atomic persistence in `src-tauri/src/settings_repair.rs` and `crates/fxcursor-daemon/src/state.rs` using unique PID/counter temporary files and clean removal upon rename failure.
- [x] **Windows Hook Cleanup**: Added explicit `UnhookWindowsHookEx(hook)` on message loop exit in `src-tauri/src/input/windows.rs`.
- [x] **Timer Resolution Cleanup**: Added `restore_timer_resolution()` (`timeEndPeriod(1)`) and an RAII `TimerResolutionGuard` in `src-tauri/src/overlay/mod.rs`.
- [x] **Reliable Panic Logging**: Directed `panic.log` to the resolved application data directory with fallback to CWD in `src-tauri/src/panic_log.rs`.
- [x] **Tooling Parity**: Added `crates/fxcursor-render/Cargo.toml` to the version targets in `scripts/before-commit.ts`.
- [x] **Dev Console QoL & Log Stream**: Added `debug` log level filter button in `src/components/Tabs/DevConsoleTab.tsx`, and converted `src/lib/console.ts` to clean static imports to eliminate Vite chunking warnings.
- [x] **Reactive Toast Fix**: Corrected inverted state reporting in `src/App.tsx` when toggling effects via `Ctrl + E`.
- [x] **GPU Device Texture Dimension Clamping**: Clamped depth texture dimensions in `crates/fxcursor-render/src/renderer.rs` against `device.limits().max_texture_dimension_2d` to prevent driver panics on ultra-wide multi-monitor configurations.
- [x] **Theme Accent Consistency**: Unified `--accent-primary` tokens across `HotkeysTab.tsx`.
- [x] **Named Capture Worker Threads**: Assigned explicit thread identifier `"capture-png-worker"` in `src-tauri/src/overlay/mod.rs` for enhanced observability.
- [x] **Exhaustive Code Comments**: Documented all `AppConfig` and subsystem structs in `fxcursor-protocol`, `OverlayRenderer` architecture in `fxcursor-render`, and daemon state in `fxcursor-daemon`.

### 2.11 GPU cursor bypass (2026-09-22, session 8)

- [x] **Protocol**: new `GpuCursorConfig` (`enabled`, `hide_system_cursor`, `rotate_with_movement`, `rotation_smoothing`, `click_scale_percent`, `click_scale_duration_ms`) on `AppConfig` with `#[serde(default)]`; TS mirror in `presets.ts`, `bindings.ts` type, fixture regenerated; parity tests green.
- [x] **Renderer** (`crates/fxcursor-render/src/cursor.rs` + pipeline in `renderer.rs` / `render.wgsl`): `CursorShape` snapshots (premultiplied top-down RGBA8), pure `cursor_quad_vertices` (hotspot-pinned quad, unit-tested), `GpuCursorState` with exponential direction smoothing + short-arc rotation easing (rest = upright) and a sine-bump click bounce; `vs_cursor`/`fs_cursor` textured-quad pipeline in shader group 1, drawn last with premultiplied blend; `is_animating` includes rotation/bounce so the loop never parks mid-animation.
- [x] **Win32 backend** (`src-tauri/src/cursor.rs`): `GetCursorInfo` → `GetIconInfo` → `GetDIBits` extraction (colour DIB + AND-mask alpha fallback, monochrome AND/XOR cursors), per-handle cache, invisible-cursor read-through while hidden; hide/restore via `CopyIcon` + `SetSystemCursor(OCR_NORMAL)` with `SPI_SETCURSORS` fallback; idempotent `force_restore` on Tauri `RunEvent::Exit*` and in the panic hook. Non-Windows: no-op stubs (extract = `None`, never hides).
- [x] **Render-loop wiring** (`overlay/mod.rs` 3a): extracts/uploads each pass (parked loop included), forces settle frames on shape change, mirrors hide state from config; feature-off drops the shape and restores the arrow.
- [x] **Studio**: "GPU Cursor Bypass" `SectionCard` in the Head tab (master toggle, hide-system, rotate, smoothing/bounce sliders) wired through `App.tsx`.
- [x] Tests: 5 pure-function tests in the render crate (`cursor_quad_vertices` pinning/axis alignment, bounce envelope, eastward rotation tracking + rest return, angle wrapping) + extraction/restore smoke tests in `src-tauri` → workspace total **60**.

### 2.12 Trail physics rewrite + LazyBrush (2026-09-22, session 9)

- [x] **Root causes fixed**: lag was a cascade of ~6 serial first-order filters (head EMA → `follow_step` → 4 `lead_nodes`); jitter was `block_overtake` wall stick-slip + `cursor_dir` threshold; width taper was segment-index `progress` distorted by near-coincident merges; 90° folds favouring the tail was the WGSL `seg × 1e-6` depth tiebreaker.
- [x] **`TrailChain` rewrite** (`crates/fxcursor-render/src/renderer.rs`): optional **LazyBrush** dead-zone on the pointer (TD formula `factor = 1-√(1-(1-f)²)`, 0.1 px quantised excess, dt-scaled) → head is a **Windhawk spring-damper** toward the brush (`v += gap×k×dt_scale; v *= fric; p += v×dt_scale`) → body chain with 0.3 second-neighbour coupling → **clamped 64 px distance constraint** with velocity correction. Removed: walls/`block_overtake`, `cursor_dir`, `follow_step`, `lead_nodes` pursuit, `TrailNode.dir`. Kept: 1/120 s sub-steps (≤16), exponential friction (`powf(dt_scale)`).
- [x] **Config**: `lead_nodes` removed; `lazy_enabled` (default false), `lazy_radius` (30 px), `lazy_friction` (0.4) added with `#[serde(default)]` — old configs self-heal. Presets ×5, `presets.ts`, `bindings.ts`, fixture, parity green.
- [x] ~~**Arc-length `build_samples`**~~ — **reverted in session 11**: every legacy build parameterises by node index; arc length kept the trail at full length after a stop and made it "breathe". The real bug was the *post-merge* index, now fixed by carrying the original node index through merges.
- [x] **WGSL tiebreaker**: head-favouring saturating bias `1e-3/(1+seg)` replaces `seg × 1e-6` — at folds the head wins coverage ties; stencil paint-once and 1e-4 colour epsilon unchanged. Capsule-union depth pass otherwise untouched (judged sound).
- [x] **Studio**: "Lazy Brush" section (toggle + radius + friction sliders) in Trail tab; Head Kinematics relabelled (Spring Strength 10–300, Friction 0–99); Lead Nodes slider removed.
- [x] **LivePreview** mirrors the new chain (LazyBrush → spring-damper → constraint). Since session 11 it runs the shared `src/lib/trail.ts`.
- [x] **Head blob fix**: the ribbon no longer prepends the raw pointer ahead of the spring head — `build_ribbon` and LivePreview sample from `chain.nodes` only (legacy Windhawk/V3/TD). The old prepend stretched a full-width capsule across the spring lag and produced a visible blob at the head.
- [x] Tests: wall/overtake/lead/reversal tests replaced by bounded-overshoot settle, LazyBrush dead-zone hold-then-drag, distance-constraint clamp (2000 px teleport), arc-vs-index progress; growth + frame-rate independence kept → workspace total still **60**.

### 2.13 Trail speed & flick-acceleration fix + trail-only defaults (2026-09-22, session 10)

- [x] **`REFERENCE_FRAME` 1/60 → 1/120**: Windhawk `kReferenceFrameTime` and V3 both use 1/120; the 1/60 value halved spring impulses per second and softened `powf` friction — the trail lagged legacy and felt slow. Velocity-width normalisation rescaled (`speed/10`) for the new units — **reverted to `/20` in session 11**: Windhawk, GDI+ and V3 all use `/20` in these same 1/120 s units.
- [x] **Distance clamp**: was 64 px — *below* the natural steady-state gap at speed, so it fought the springs every frame and injected velocity via `delta/dt_scale` (reads as accelerating on sudden flicks). Now **512 px, teleport-guard only, inelastic** (strips separating `v_n`, never adds the correction into `vx`). LivePreview mirrors this.
- [x] **Trail-only defaults**: `effect_mode: Ribbon`; `head` / `ripple` / `particles` default off (V3 parity). Built-in presets keep their own explicit flags; TS preset mirror updated so neon/razor/celestial/firestorm/rainbow still enable what Rust enables. Existing user `config.json` needs one **Reset Defaults** to pick this up.
- [x] Tests: +2 (`fast_flick_does_not_slingshot_past_a_stopped_pointer`, `sustained_fast_motion_never_engages_the_distance_clamp`) → workspace **62**; frame-rate tolerance 12→20 px (sub-step residual).

### 2.14 Full audit: legacy-faithful trail, pacing, GPU/IPC hardening, parity fixtures (2026-09-23, session 11)

Three read-only audits (Tauri backend, frontend + tooling, legacy Windhawk D3D/GDI+ / V3 / TD trail pipelines vs V4) followed by fixes.

- [x] **Trail shape = legacy**: `build_samples` progress is `(i + t)/(N − 1)` on the *original* node index (carried through the near-duplicate merge); a collapsed chain emits one zero-length capsule per layer (the resting dot Windhawk draws — it used to pop in and out); velocity boost `min(speed/20, 1)`; blur interpolated per vertex (`params.w`); `is_moving` also requires the head to reach the brush (0.1 px) and the LazyBrush to settle; non-finite chains reset on the pointer; `interpolation_steps` bounded 1–32; fade mode 4 (Smoothstep) implemented (the UI offered it, the renderer fell back to linear).
- [x] **Squishy head = Windhawk `UpdateSquishyCursor`**: eased position, exponential `1 − (1 − s)^dt_scale` smoothing, velocity in px per reference frame (px/s saturated `min(v × 8, 200)` at ~25 px/s: the head sat fully squashed on any motion).
- [x] **Pacing**: `Fifo` + `desired_maximum_frame_latency: 1`; while uncapped the next backbuffer is acquired *before* the pointer is sampled, so the loop is phase-locked to vblank (sleep pacing at the refresh period drifted and repeated a frame every ~20 frames at 144 Hz). The first frame after an idle park integrates one period, not the whole park (up to 100 ms of spring toward a pointer that just moved). Failed acquisitions back off instead of spinning. Present mode chosen from the surface caps; refresh rate re-read every second.
- [x] **No main-thread stalls**: the per-frame `window.inner_size()` (a blocking round-trip to the Tauri main thread) moved to the 1 s display poll, which also refits on DPI-driven window moves; heavy IPC commands are `async`; autostart state cached (was a registry read on every slider tick).
- [x] **Input**: `GetCursorPos` is the only position source on Windows (as in all legacy builds); the hook only wakes the loop and records clicks (its `pt` is unclipped at screen edges and alternated with the poll). Non-Windows button mapping fixed (`device_query` is 1-based; X11 order differs).
- [x] **GPU surface**: non-sRGB `Bgra8Unorm` target like Windhawk (sRGB encoding made pre-multiplied glow edges over-bright in DWM); adapter limits instead of the 8192 px default (3×4K desktops panicked), surface clamped to the device limit; an opaque-only surface aborts instead of blacking out the desktop; `WGPU_*` backend overrides honoured.
- [x] **GPU cursor**: texture created with `TEXTURE_BINDING` (enabling the feature panicked the render thread); hidden-arrow extraction reads the saved original (`SetSystemCursor` replaces the stock arrow's contents, so the old identity check never matched and the I-beam stuck); pointer visibility mirrored (fullscreen video, typing); per-handle shape cache (`Arc`, no per-frame pixel copy) invalidated every second; restore latched on exit so a late frame cannot hide the arrow again.
- [x] **Config**: `AppConfig::sanitize()` clamps every numeric field and replaces NaN/∞ on load and in `replace_config` (IPC, `--apply`, presets, import); autosave flushed on exit.
- [x] **Presets**: `particle_firestorm` / `celestial_orbit` / `razor_spine` no longer mask their own trail / ripples / shadow layer through `effect_mode` (regression test `preset_modes.rs`); the TS preset list is generated from Rust (`src/lib/generated/builtin_presets.json`) — the hand-written copy had drifted in 5 of 6 presets (false "MODIFIED").
- [x] **Parity & tests**: `src/lib/trail.ts` (TrailChain, samples, styles, squishy head) replays `test/fixtures/trail_trace.json` from the Rust `dump_trail_trace` example (4 runs incl. 144 Hz and LazyBrush; a 3 % change of one constant fails it); headless GPU smoke test renders every pipeline incl. the cursor quad; `bun run typecheck` now also checks `test/` and `scripts/`. CI checks all four fixtures for freshness.
- [x] **Studio**: config echoes recognised by content (a tray / hotkey toggle during a slider drag is no longer reverted), listener cleanup, hotkey committed on Enter/blur (no partial system-wide registrations), preview right/middle click colours fixed, accessible names on every switch / slider / colour input, `%` display fix, labels describe what parameters now do, safe console serialisation, benchmark / reset error paths.
- [x] **Tooling**: `update-deps` never downgrades (stable `max_version` compared first; `bun update` without `--latest`, which moved pre-release pins back to `latest`), discovers every workspace manifest and dependency table, anchored regexes keep the version operator, aborts on any failing step, runs cargo + bun tests, `--dry-run` prints the report.
- [x] Webview hardening moved to `src/lib/hardening.ts` (runs on every load; the one-shot Rust `eval` was lost on reload and claimed protections it did not implement).

---

## 3. Verification matrix (2026-09-23)

| Check              | Command                                                 | Result                                                                                                                |
| :----------------- | :------------------------------------------------------ | :-------------------------------------------------------------------------------------------------------------------- |
| TypeScript         | `bun run typecheck`                                     | ✅ 0 errors                                                                                                           |
| Lint               | `bun run lint`                                          | ✅ clean (oxlint on 39 files, 0 warnings, 0 errors)                                                                   |
| Vite bundle        | `bun run build`                                         | ✅ 130 kB JS / 4.3 kB CSS (0 warnings)                                                                                |
| Bun unit tests     | `bun test`                                              | ✅ 27 passed (presets, theme, version, config parity, effect-mode parity, **trail parity vs Rust trace**) |
| Cargo workspace    | `rtk cargo check --workspace`                           | ✅ 0 errors, 0 warnings                                                                                               |
| Cargo tests        | `rtk cargo test --workspace`                            | ✅ 78 passed (render 31 incl. retract-after-stop, resting dot, index taper, squishy head; GPU smoke 1; preset modes 1; protocol 11 incl. sanitize; app 27; daemon 5; parity/mode) |
| Live run (2026-09-23) | `bun run tauri dev` + `--capture`                  | ✅ RTX 4070 Vulkan, 165 Hz, `Bgra8Unorm` + PreMultiplied surface, 4-layer ribbon captured, no errors in the log |
| Live run (Windows) | `bun run tauri dev`                                     | ✅ virtual desktop (−308, 0) 2560×2680 across two monitors, 240 Hz pacing, RTX 4070 (Vulkan), hook installed, config adopted from the CursorFX install, bindings regenerated |
| Trail transients   | `scripts/snapshots/snapshot_motion.ps1` (30-frame bursts) | ✅ stop / reversal / 90° turn keep a clean rounded tip at the pointer; HUD verified with `--apply` + `--capture`     |
| Clippy             | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean (0 warnings)                                                                                                 |
| Release build      | `bun run tauri build`                                   | ✅ 6 m 22 s, `fxcursor.exe` 11.7 MB, NSIS installer `FXCursor_0.5.0_x64-setup.exe` 3.6 MB                              |
| Portable zip       | `bun run package:portable`                              | ✅ `fxcursor-portable-0.5.0-win-x64.zip` 4.6 MB                                                                        |
| GitHub Actions     | `.github/workflows/ci.yml`                              | 🟡 first runs on the new repository: Windows job reached clippy (one lint fixed), Ubuntu job needed the 24.04 package list — fixed in the follow-up commit |

---

## 4. Known issues & technical debt

| #   | Issue                                                                                                                                              | Severity                        | Where                                    |
| :-- | :------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------ | :--------------------------------------- |
| 1   | Overlay is a **webview window** hosting a wgpu surface: extra WebView2 process and RAM. A native (`winit`/raw HWND) overlay would cut idle memory. | Medium                          | `src-tauri/src/lib.rs`, `overlay/mod.rs` |
| 2   | Windows input is a low-level hook, but macOS/Linux still poll `device_query` (edge-only click detection, no idle park below 4 ms).                 | Medium                          | `src-tauri/src/tracker.rs`               |
| 3   | `crates/fxcursor-daemon` still has no client and is not launched by anything (the pipe double-close is fixed); it shares the renderer crate.        | Medium                          | `crates/fxcursor-daemon/src/ipc/mod.rs`  |
| 4   | `fps_counter.align_*` corner placement is honoured by the renderer HUD, but the Studio exposes no UI to switch corners (defaults: right/top).                          | Low                             | `src/components/Tabs/DeveloperTab.tsx` |
| 5   | `LivePreview` approximates the GPU feather with two canvas bands; physics, sampling and styling come from `src/lib/trail.ts` (tested against Rust).  | Low                             | `src/components/Preview/LivePreview.tsx` |
| 6   | Dev Console only streams records from `fxcursor*` targets; wgpu/tauri internals stay on stderr by design.                                          | Low                             | `src-tauri/src/logger.rs`                |
| 7   | Toast and tab accent colours now follow the theme tokens; a few semantic colours (success green, warning orange) are intentionally fixed.          | Low                             | `src/index.css`                          |
| 8   | No frontend/IPC tests; the GitHub Actions workflow exists but has not run yet (first push pending at the time of writing).                       | Low                             | `test/`, `.github/workflows/ci.yml`     |
| 9   | Non-Windows: virtual screen bounds fall back to 1920×1080; click-through and global mouse on macOS/Wayland unverified.                             | High (for cross-platform claim) | `overlay/mod.rs`, `tracker.rs`           |
| 10  | After a stop the bright part of the ribbon retracts into the cursor within ~1 s (node-index taper, as legacy); the faint tail finishes gathering later. | Low                             | `crates/fxcursor-render/src/renderer.rs` |
| 13  | Vsync-locked pacing was verified by the init log and unit tests, not yet by a high-speed camera; judge smoothness on the desktop with `bun run tauri dev`. | Low                             | `src-tauri/src/overlay/mod.rs`           |
| 11  | GPU cursor: animated cursors (.ani) freeze on their first frame; only the arrow rotates with movement.                                             | Low (accepted)                  | `src-tauri/src/cursor.rs`, `renderer.rs` |
| 12  | `kill -9` / power loss while `hide_system_cursor` is active leaves the invisible arrow installed (`SetSystemCursor` persists); next launch cannot know to restore. | Low (accepted)           | `src-tauri/src/cursor.rs`               |

---

## 5. Roadmap

### Phase A — Stabilise the single-process app (now)

- [x] Persistence, hotkey, autostart, tray behaviour, capabilities, dual ring, rainbow, idle skip, backend presets (2026-09-09).
- [x] V4 baseline committed as **FXCursor** (repository root = the app; V3, original mods and old scripts under `legacy/`) (2026-09-09).
- [x] Generate TypeScript bindings with `tauri-specta` and delete the hand-written `AppConfig` interfaces (2026-09-09).
- [x] Parity test between `AppConfig::default()` and `getDefaultConfig()` via a shared fixture (2026-09-09).
- [x] `effect_mode` implemented (renderer mask + header selector); `fps_counter` implemented as Developer Hub telemetry (2026-09-09).
- [x] `LivePreview` brought to parity with the Rust physics and effect modes (2026-09-09); enforced by the Rust → TS trail trace fixture since 2026-09-23.
- [x] About tab and Developer Hub from `get_diagnostics`; Rust `log` records streamed to the Dev Console (2026-09-09).
- [x] Hardcoded cyan replaced with accent tokens (2026-09-09).
- [x] GPU cursor bypass (`gpu_cursor`): shape extraction, on-top quad with rotation + click bounce, optional system-arrow hide with exit/panic restore (2026-09-22).

### Phase B — Native input & true idle (V4 Pillar 4)

- [x] Windows: `WH_MOUSE_LL` hook on a message thread with `GetCursorPos` UIPI fallback (2026-09-09).
- [x] Render thread parks on a condvar; woken by input, config commits, or while animating (2026-09-09).
- [ ] Measure and record: idle CPU, click-to-ripple latency; consider Raw Input (`WM_INPUT`) if the hook is ever throttled.
- [ ] macOS `CGEventTap` and Linux evdev sources for the same `InputHub`.

### Phase C — Shared render crate & daemon decision (V4 Pillar 1)

- [x] `crates/fxcursor-render` extracted and used by both `src-tauri` and the daemon (2026-09-09).
- [ ] Decide: (a) keep single-process Tauri and delete the daemon, or (b) make the daemon the renderer and turn `src-tauri` into a thin pipe client. Recommendation: (a) until a native overlay window exists, then revisit (b).
- [ ] If keeping the daemon: fix pipe double-close, cap vertex/instance counts, add an IPC client + tests.

### Phase D — GPU compute physics (V4 Pillar 2, optional)

- [ ] Dispatch `physics.wgsl` for the spring chain and particles; bind the storage buffer as the ribbon vertex source.
- [ ] Only worth it after Phase B; CPU physics is currently < 1 % of a core.

### Phase E — Cross-platform

- [ ] macOS: verify transparent overlay level, `ignoresMouseEvents`, accessibility prompt for global mouse; Metal surface.
- [ ] Linux: X11 ARGB visual + input shape; Wayland layer-shell investigation.
- [x] Display-change handling: virtual-desktop bounds polled every second and the overlay refitted (physical size/position) (2026-09-09). A `WM_DISPLAYCHANGE` listener would only make it instant.

### Phase F — Release engineering

- [x] `bundle.active = true` with an NSIS current-user installer (2026-09-09).
- [x] Portable zip: `bun run package:portable` (exe + `portable` marker + `Data/` + README) (2026-09-09).
- [x] GitHub Actions workflow: typecheck, lint, tests, build, clippy, cargo tests, fixture freshness, Windows release build (2026-09-09; runs at the repository root).
- [ ] Changelog discipline and version bump via `scripts/version.ts`.

### Phase G — Ideas adopted from the D3D_CURSOR reference (Windhawk-era C++ engine)

The sibling `D3D_CURSOR` project (D3D11 engine + Tauri studio) was reviewed in full. What it does that FXCursor did not, and the decision for each:

| Idea                                                                      | Decision                                                                                                                          |
| :------------------------------------------------------------------------ | :-------------------------------------------------------------------------------------------------------------------------------- |
| Fixed sub-stepped physics (their self-critique: explicit Euler diverges)  | ✅ Done (1/120 s, up to 16 slices)                                                                                                |
| Evenly spaced particle burst angles, 0.7–1.3× speed                       | ✅ Done                                                                                                                           |
| On-overlay FPS HUD drawn with a 3×5 bitmap font as instanced SDF quads     | ✅ Done (`fps_counter.enabled`, `align_right` / `align_bottom`)                                                                    |
| In-window shortcuts (`Ctrl+S` save, `Ctrl+E` toggle effects, `Ctrl+1..9`) | ✅ Done, listed in the Hotkeys tab cheat-sheet                                                                                     |
| Tray tooltip showing the live state                                       | ✅ Done ("FXCursor · effects on · preset")                                                                                         |
| Render thread priority (MMCSS / above-normal)                             | ✅ Done (`THREAD_PRIORITY_ABOVE_NORMAL` on Windows)                                                                                |
| Custom user presets saved next to `config.json`                           | ✅ Done (`save_user_preset` / `delete_user_preset`, Presets tab)                                                                   |
| Bypass the system cursor (GPU-drawn HCURSOR with rotation + click bounce) | ✅ Done (2026-09-22): `gpu_cursor` extraction + on-top quad + `SetSystemCursor` hide with exit/panic restore                                |
| Click scaling of the real cursor, click-text OSD                          | ✅ Bounce done on the GPU cursor (`click_scale_percent` / `_duration_ms`); click-text OSD ⏳ Backlog                        |
| Multi-touch trails                                                        | ⏳ Backlog — `WM_POINTER` input source feeding several `TrailChain`s                                                               |
| `WM_DISPLAYCHANGE` debounce, DwmFlush pacing, 8-frame flush               | ➖ Covered differently: bounds polled every second, frame pacing to the display refresh, 3 settle frames                            |
| `position_history_skip`                                                   | ➖ Not adopted (unused even there)                                                                                                 |

### Suggested next working session

1. Run the GitHub Actions workflow on the new repository and fix anything platform-specific it reports.
2. macOS `CGEventTap` / Linux evdev input sources and overlay verification (Phase E).
3. Decide the daemon's fate (Phase C) — recommendation: delete it once a native overlay window exists.
4. Measure idle CPU and click-to-ripple latency; record the numbers here.
5. Click-text OSD and per-app cursor overrides on top of the shipped `gpu_cursor` bypass.

---

## 6. Running and testing

```powershell
cd FXCursor            # repository root
bun install
bun run tauri dev          # primary dev/test command
bun run typecheck && bun run lint && bun test
rtk cargo check --workspace && rtk cargo test --workspace
bun run before-commit      # 7-gate validation
bun run arch               # regenerate ARCHITECTURE.md
bun run tauri build        # release build + NSIS installer
bun run package:portable   # portable zip (exe + portable marker + Data/)
```

> **Trail-only defaults**: factory config enables only the ribbon (`effect_mode: Ribbon`, head/ripples/particles off). After upgrading an existing install, press **Reset Defaults** once in the Developer Hub — self-healing keeps stored `enabled` flags from the old `config.json`.

Configuration file locations:

- Windows: `%APPDATA%\com.nairodorian.fxcursor\config.json`
- macOS: `~/Library/Application Support/com.nairodorian.fxcursor/config.json`
- Linux: `~/.config/com.nairodorian.fxcursor/config.json`
- Portable: `<exe dir>/Data/config.json` when a file named `portable` sits next to the executable.
