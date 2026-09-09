# FXCursor V4 — Project Progress, Status & Roadmap

> **Milestone Status**: 🟡 Under construction — Tauri single-process path works end to end on Windows; V4 "micro-daemon" architecture is a prototype. Repository restructured and renamed **FXCursor** on 2026-09-09 (app at the root, V3 under `legacy/`).
> **Version**: `0.5.0` (pre-release)
> **Last audit**: 2026-09-09, session 6 (full code read of `src-tauri`, `src`, `crates/*`; trail physics verified frame by frame with burst snapshots)
> **Tech Stack**: Tauri 2.11, Bun 1.4, SolidJS 2.0.0-rc.4, TypeScript 7.1-dev, Vite 8, wgpu 30, windows 0.62, Rust 2021

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
| Physics        | **CPU** `TrailChain`: pointer-smoothing head follower + `lead_nodes` pursuit followers + spring-damper body (2nd-order neighbour term), fixed 1/120 s sub-steps (frame deltas up to 100 ms integrated in full), **no-overtake rule** (no node crosses in front of its predecessor / the pointer), centripetal Catmull-Rom with phantom endpoints and curvature-adaptive sampling                                                                                                                                                                                          | WGSL compute pass at 240 Hz sub-steps                                          |
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
| Telemetry      | `fps_counter` drives Developer Hub polling of `FrameStats` (fps, CPU ms/frame, state, vertex/instance counts) published by the render thread every 500 ms                                                                                                                                                                                                        | On-overlay HUD text (future)                                                   |
| Render crate   | **`crates/fxcursor-render`** holds the one renderer + shaders; both `src-tauri` and the daemon depend on it (the daemon fork is gone)                                                                                                                                                                                                                            | Same                                                                           |

---

## 2. Completed milestones

### 2.1 Graphics core (`src-tauri/src/overlay/`)

- [x] wgpu 30 surface on the Tauri overlay window (`SurfaceTargetUnsafe::from_display_and_window`), pre-multiplied alpha compositing, Mailbox presentation, surface loss recovery.
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
- [x] Physics integrated in fixed 1/120 s sub-steps; particle bursts spawn at evenly spaced angles with ±0.25 rad jitter and 0.7–1.3× speed (D3D_CURSOR-style).

### 2.9 Trail head physics, burst snapshots & the FXCursor rename (2026-09-09, session 6)

- [x] **`TrailChain`** (GPU-free, unit-tested): the head is a first-order follower of the jitter-filtered pointer; the first `trail.lead_nodes` nodes (default 4, slider in Trail Physics) are pursuit followers that cannot whip; the rest is the spring-damper chain. A **no-overtake rule** stops any node from crossing in front of its predecessor (the raw pointer for the head): the loops and stubs that appeared around the cursor at stops and reversals are gone. Frame deltas up to 100 ms are integrated in full (16 sub-steps) so hitches do not detach the trail.
- [x] Centripetal Catmull-Rom (α = 0.5) with phantom endpoints replaces the uniform spline: no hooks with uneven node spacing.
- [x] Tests: stop (nothing ever in front of the pointer), reversal (passed nodes stay behind), lead-node monotonicity, frame-rate independence (60 vs 240 fps), chain growth, wall clamping. 18 render-crate tests.
- [x] **Burst snapshots**: `--capture-burst N --capture-interval ms` writes `<stem>_NN.png` at a fixed cadence; PNG encoding runs on a worker thread so the capture no longer stalls the physics it records; each capture logs a one-line chain summary (`chain nodes=… first5=… farthest=…`).
- [x] Verified with 30-frame bursts at 50 ms: stop, reversal and 90° turn all keep a clean rounded tip at the pointer (`scripts/snapshots/`).
- [x] Renamed **FXCursor**: crates `fxcursor-{protocol,render,daemon}`, binary `fxcursor`, identifier `com.fxcursor.app` (the previous `com.cursorfx.studio` config is adopted on first launch), product name FXCursor, window title "FXCursor Studio". Repository restructured: the app is the root, V3 / original mods / old scripts live under `legacy/`, CI runs at the root.

---

## 3. Verification matrix (2026-09-09)

| Check              | Command                                                 | Result                                                                                                                |
| :----------------- | :------------------------------------------------------ | :-------------------------------------------------------------------------------------------------------------------- |
| TypeScript         | `bun run typecheck`                                     | ✅ 0 errors                                                                                                           |
| Lint               | `bun run lint`                                          | ✅ clean                                                                                                              |
| Vite bundle        | `bun run build`                                         | ✅ 101 kB JS / 4.3 kB CSS                                                                                             |
| Bun unit tests     | `bun test`                                              | ✅ 17 passed (presets, theme, version, config parity, effect-mode parity)                                             |
| Cargo workspace    | `rtk cargo check --workspace`                           | ✅ 0 errors, 0 warnings                                                                                               |
| Cargo tests        | `rtk cargo test --workspace`                            | ✅ 44 passed (protocol 4 + parity 2, app 18 incl. capture/presets/CLI, render 18 + parity 2)                          |
| Live run (Windows) | `bun run tauri dev`                                     | ✅ virtual desktop (−308, 0) 2560×2680 across two monitors, 240 Hz pacing, RTX 4070 (Vulkan), hook installed, config adopted from the CursorFX install, bindings regenerated |
| Trail transients   | `scripts/snapshots/snapshot_motion.ps1` (30-frame bursts) | ✅ stop / reversal / 90° turn keep a clean rounded tip at the pointer; HUD verified with `--apply` + `--capture`     |
| Clippy             | `cargo clippy --workspace --all-targets -- -D warnings` | ✅ clean                                                                                                              |
| Release build      | `bun run tauri build`                                   | ✅ before the rename: 5 m 48 s, 11.4 MB exe, NSIS installer 3.5 MB; re-run under the `fxcursor` name via CI           |
| Portable zip       | `bun run package:portable`                              | ✅ `fxcursor-portable-<version>-win-x64.zip` (built from the release exe)                                              |

---

## 4. Known issues & technical debt

| #   | Issue                                                                                                                                              | Severity                        | Where                                    |
| :-- | :------------------------------------------------------------------------------------------------------------------------------------------------- | :------------------------------ | :--------------------------------------- |
| 1   | Overlay is a **webview window** hosting a wgpu surface: extra WebView2 process and RAM. A native (`winit`/raw HWND) overlay would cut idle memory. | Medium                          | `src-tauri/src/lib.rs`, `overlay/mod.rs` |
| 2   | Windows input is a low-level hook, but macOS/Linux still poll `device_query` (edge-only click detection, no idle park below 4 ms).                 | Medium                          | `src-tauri/src/tracker.rs`               |
| 3   | `crates/fxcursor-daemon` still has no client and is not launched by anything (the pipe double-close is fixed); it shares the renderer crate.        | Medium                          | `crates/fxcursor-daemon/src/ipc/mod.rs`  |
| 4   | `fps_counter.align_right` / `align_bottom` are reserved for a future on-overlay HUD and currently unused.                                          | Low                             | `crates/fxcursor-protocol/src/config.rs` |
| 5   | `LivePreview` approximates GPU edge feathering with canvas shadows; geometry and physics otherwise match.                                          | Low                             | `src/components/Preview/LivePreview.tsx` |
| 6   | Dev Console only streams records from `fxcursor*` targets; wgpu/tauri internals stay on stderr by design.                                          | Low                             | `src-tauri/src/logger.rs`                |
| 7   | Toast and tab accent colours now follow the theme tokens; a few semantic colours (success green, warning orange) are intentionally fixed.          | Low                             | `src/index.css`                          |
| 8   | No frontend/IPC tests; the GitHub Actions workflow exists but has not run yet (first push pending at the time of writing).                       | Low                             | `test/`, `.github/workflows/ci.yml`     |
| 9   | Non-Windows: virtual screen bounds fall back to 1920×1080; click-through and global mouse on macOS/Wayland unverified.                             | High (for cross-platform claim) | `overlay/mod.rs`, `tracker.rs`           |
| 10  | After a stop the ribbon retracts at the speed it was travelling (spring train), so a long fast trail takes 1–3 s to gather; inherent to soft presets. | Low                             | `crates/fxcursor-render/src/renderer.rs` |

---

## 5. Roadmap

### Phase A — Stabilise the single-process app (now)

- [x] Persistence, hotkey, autostart, tray behaviour, capabilities, dual ring, rainbow, idle skip, backend presets (2026-09-09).
- [x] V4 baseline committed as **FXCursor** (repository root = the app; V3, original mods and old scripts under `legacy/`) (2026-09-09).
- [x] Generate TypeScript bindings with `tauri-specta` and delete the hand-written `AppConfig` interfaces (2026-09-09).
- [x] Parity test between `AppConfig::default()` and `getDefaultConfig()` via a shared fixture (2026-09-09).
- [x] `effect_mode` implemented (renderer mask + header selector); `fps_counter` implemented as Developer Hub telemetry (2026-09-09).
- [x] `LivePreview` brought to parity with the Rust physics and effect modes (2026-09-09).
- [x] About tab and Developer Hub from `get_diagnostics`; Rust `log` records streamed to the Dev Console (2026-09-09).
- [x] Hardcoded cyan replaced with accent tokens (2026-09-09).

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
| Bypass the system cursor (GPU-drawn HCURSOR with rotation + click bounce) | ⏳ Backlog — needs cursor hiding across apps (`SetSystemCursor`) and a fallback when the hook is muted                             |
| Click scaling of the real cursor, click-text OSD                          | ⏳ Backlog — depends on the custom cursor above                                                                                    |
| Multi-touch trails                                                        | ⏳ Backlog — `WM_POINTER` input source feeding several `TrailChain`s                                                               |
| `WM_DISPLAYCHANGE` debounce, DwmFlush pacing, 8-frame flush               | ➖ Covered differently: bounds polled every second, frame pacing to the display refresh, 3 settle frames                            |
| `position_history_skip`                                                   | ➖ Not adopted (unused even there)                                                                                                 |

### Suggested next working session

1. Run the GitHub Actions workflow on the new repository and fix anything platform-specific it reports.
2. Custom cursor (bypass the system cursor) + click bounce, the last big D3D_CURSOR feature.
3. macOS `CGEventTap` / Linux evdev input sources and overlay verification (Phase E).
4. Decide the daemon's fate (Phase C) — recommendation: delete it once a native overlay window exists.
5. Measure idle CPU and click-to-ripple latency; record the numbers here.

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

Configuration file locations:

- Windows: `%APPDATA%\com.fxcursor.app\config.json`
- macOS: `~/Library/Application Support/com.fxcursor.app/config.json`
- Linux: `~/.config/com.fxcursor.app/config.json`
- Portable: `<exe dir>/Data/config.json` when a file named `portable` sits next to the executable.
