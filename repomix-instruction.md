# Project Context & AI Instructions

## Project Overview
**FXCursor** is a GPU-accelerated cursor effects overlay and settings studio built with **Tauri 2**, **Bun**, **SolidJS 2**, **TypeScript 7**, and **wgpu 30**. It renders a 4-layer ribbon trail, squishy head, click ripples, particles and orbit satellites on a transparent, click-through desktop overlay. The active code is in ``; `legacy/project_cursor/` is the frozen V3 predecessor.

## Architecture Summary (as implemented)
- **App Shell**: one Tauri 2 process with two windows: the SolidJS Studio (`main`) and a transparent overlay (`overlay`). A `gpu-render-thread` owns a wgpu 30 surface on the overlay window (`src-tauri/src/overlay/mod.rs`).
- **Render pipeline**: CPU physics (spring chain, Catmull-Rom, normal-flip stabilisation) builds ribbon vertices and SDF instances each frame; `shader.wgsl` rasterises them with pre-multiplied alpha (`overlay/renderer.rs`). GPU work is skipped when nothing changes.
- **IPC**: SolidJS calls Tauri `invoke` commands (`get_config`, `update_config`, `toggle_overlay`, `reset_defaults`, `save_config`, `list_presets`, `apply_preset`, `import_config`, `get_diagnostics`, `ping`); Rust emits `config-updated`.
- **Config**: `AppConfig` in `crates/fxcursor-protocol/src/config.rs`, mirrored by hand in `src/lib/presets.ts`. Persisted as JSON with field-level self-healing (`self_healing.rs`, `src-tauri/src/settings_repair.rs`) and debounced autosave.
- **OS integration**: tray, global hotkey and autostart (`src-tauri/src/integrations.rs`), single instance, portable mode (`portable.rs`).
- **Daemon**: `crates/fxcursor-daemon` is an experimental Windows-only winit renderer with a JSON named-pipe server; nothing launches it and `physics.wgsl` is not dispatched.

## Key Technical Constraints
1. **Click-through overlay** uses Tauri's `set_ignore_cursor_events(true)`; do not reintroduce WndProc subclassing.
2. **Multi-monitor**: the overlay spans the Win32 virtual desktop; shaders subtract `virtual_origin`, so negative coordinates are valid.
3. **Surface recovery**: `Lost`/`Outdated` surfaces are reconfigured in place.
4. **Persistence**: never write config files outside `settings_repair.rs`; all changes flow through `commit()` in `lib.rs`.
5. **Rust ⇄ TS parity**: change `config.rs` and `presets.ts` together until `tauri-specta` bindings are generated.

## Coding Style & Conventions
- **Rust**: edition 2021, `#[cfg(target_os)]` for platform code, `unsafe` only for Win32/wgpu surface creation, `log` macros for diagnostics.
- **TypeScript**: strict, `noUnusedLocals`, `noUnusedParameters`, oxlint + Prettier (single quotes, 2 spaces).
- **SolidJS 2**: `onSettled`, two-argument `createEffect`, `<Show>`; no virtual DOM assumptions.
- **Styling**: hand-written CSS with custom properties (`--accent-primary`, `--accent-glow`, `--accent-subtle`); no Tailwind.
- **Shaders**: WGSL only.
- **Tooling**: Bun only (never npm/yarn/pnpm); prefix shell commands with `rtk`.

## Directory Layout
- `src-tauri/` — Rust backend (Tauri 2 + wgpu 30)
- `src/` — SolidJS 2 frontend (Vite 8)
- `crates/fxcursor-protocol/` — shared config, presets, self-healing
- `crates/fxcursor-daemon/` — experimental headless renderer
- `scripts/` — Bun tooling (update-deps, before-commit, generate-arch, sync-docs)
- `docs/` — V4 specification and research
- `legacy/project_cursor/` — legacy V3 (reference only)
- `legacy/original_mods/` — legacy C++ reference implementations
- `memory.md` — architectural decisions log
