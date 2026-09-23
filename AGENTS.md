# Development Procedure & Workflow Guidelines (AGENTS.md)

This repository contains **FXCursor V4 Next-Gen**, a cross-platform hardware-accelerated cursor visual effects studio and rendering engine powered by **Tauri 2**, **Bun.js**, **SolidJS 2**, **TypeScript 7**, **wgpu 30 (Direct3D 12 / Metal / Vulkan)**, **windows 0.62**, and **Cargo (Rust)**.

---

## ⚡ Primary Standard: Bun, Testing Command & Golden Rule

> [!CRITICAL]
> **1. Package Manager Standard**:
> NEVER use `npm`, `npx`, `yarn`, or `pnpm`. **ALWAYS use `bun`** for dependency management, script execution, and tooling.
>
> **2. Ultimate Testing Command**:
> The **ONLY** primary command to launch, test, and develop this application is:
>
> ```bash
> bun run tauri dev
> ```
>
> **3. Always Upgrade to Pre-Release Dependencies Standard**:
> **ALWAYS update all dependencies to the newest PRE-RELEASE state possible** across all tech stacks and languages (NPM dist-tags: `next`, `beta`, `rc`, `alpha`, `canary`, `experimental`, `insiders`, `dev`, and Crates.io `newest_version` across Rust crates). Never downgrade to stable when pre-releases exist. Always run `bun run update-deps` to enforce this.
>
> **4. Golden Rule — `rtk` Command Prefix**:
> Always prefix commands with `rtk`. If RTK has a dedicated filter, use it; otherwise it passes the command through unchanged. RTK is always safe.
>
> - This applies to **every** command, including chains: `rtk git add . && rtk git commit -m "msg" && rtk git push`.
> - `rtk bun ...` is NOT used — `rtk` is not needed in front of `bun` when running the sanctioned Bun scripts (see the command table below).
>
> | ❌ Wrong                                       | ✅ Correct                                                                |
> | :--------------------------------------------- | :------------------------------------------------------------------------ |
> | `git add . && git commit -m "msg" && git push` | `rtk git add . && rtk git commit -m "msg" && rtk git push`                |
> | `git status`                                   | `rtk git status`                                                          |
> | `cargo check --workspace`                      | `rtk cargo check --workspace`                                             |
> | `cargo test --workspace`                       | `rtk cargo test --workspace`                                              |
> | `bun run typecheck`                            | `bun run typecheck` (Bun scripts run directly, no `rtk` wrapper needed)   |
> | `bun run update-deps`                          | `bun run update-deps` (Bun scripts run directly, no `rtk` wrapper needed) |
> | `bun run tauri dev`                            | `bun run tauri dev` (Bun scripts run directly, no `rtk` wrapper needed)   |

---

## 🧭 Orientation for agents

- The repository root is the **active V4 codebase** (FXCursor). `legacy/project_cursor/` is the frozen V3 (React) app and `legacy/original_mods/` holds the original Windhawk C++ mods; they are kept for reference only — do not develop there.
- Read `PROGRESS.md` first: it states what is actually implemented versus what `docs/V4_ARCHITECTURE_SPECIFICATION.md` targets, the known issues, and the roadmap.
- Architecture today: **single Tauri process**. The overlay is a transparent Tauri window with a wgpu 30 surface driven by `src-tauri/src/overlay/`; physics is on the CPU. `crates/fxcursor-daemon` is an experimental prototype that the app does not launch.
- Configuration lives in `crates/fxcursor-protocol` (`AppConfig`, presets, self-healing). `src/lib/bindings.ts` is **generated** by `tauri-specta` on every debug run; `src/lib/presets.ts` keeps the TypeScript defaults mirror for browser preview mode, and the built-in presets are **generated** into `src/lib/generated/builtin_presets.json` (never edit it by hand). When you change the Rust struct or presets: give new fields a `#[serde(default)]`, update the defaults in `presets.ts`, run `bun run fixtures`, and let the parity tests (`test/config-parity.test.ts`, `crates/fxcursor-protocol/tests/fixture_parity.rs`) confirm both sides agree. Out-of-range values are clamped by `AppConfig::sanitize()` (`crates/fxcursor-protocol/src/sanitize.rs`); extend it with every new numeric field.
- The trail physics is `TrailChain` in `crates/fxcursor-render/src/renderer.rs` and is GPU-free on purpose: reproduce motion bugs as unit tests there (stop/retract, settle, flick, LazyBrush, resting-dot scenarios exist) before touching the code. Its TypeScript mirror is `src/lib/trail.ts` (used by the live preview): change both, run `bun run fixtures`, and `test/trail-parity.test.ts` must pass against the regenerated `test/fixtures/trail_trace.json`. The legacy reference algorithms are `legacy/original_mods/D3D_cursor_mod.wh.cpp` (Windhawk) and `legacy/TD_Web_Trail/trail-system.js`.
- To *see* the overlay, use the snapshot tools (`scripts/snapshots/*.ps1`, `--capture` / `--capture-burst` flags): ordinary screen capture cannot see the GPU overlay. Bursts are the way to judge transients.
- Persistence is automatic: every `update_config` is debounce-saved to `config.json`. Do not add ad-hoc file writes.
- After touching files, run `bun run arch` so `ARCHITECTURE.md` stays in sync, and keep `PROGRESS.md` truthful.

## 🎨 Core Architecture: 4-Layer Master Design

FXCursor V4 implements the 4-layer master trail renderer over wgpu 30 (Direct3D 12 / Vulkan on Windows, Metal on macOS, Vulkan on Linux; verified on Windows):

1. **Layer 1: Outer Glow** (`150%` width factor, `39% → 50%` blur, White `[1, 1, 1, 1]`) — Feathered exterior boundary providing high contrast across dark backdrops.
2. **Layer 2: Mid Shadow** (`90%` width factor, `10%` blur, Black `[0, 0, 0, 1]`) — Dark contrast outline preventing ribbon washout against pure white surfaces.
3. **Layer 3: Crisp Core** (`50%` width factor, `10%` blur, White `[1, 1, 1, 1]`) — Primary solid luminous body.
4. **Layer 4: Inner Spine** (`15%` width factor, `10%` blur, Black `[0, 0, 0, 1]`) — Ultra-thin centerline needle maintaining razor-sharp tracking alignment.

Each layer is rendered as a union of round capsules resolved on the GPU with a depth-based max-coverage pre-pass (`crates/fxcursor-render`): joins and caps are round by construction, hairpins never fold, and overlapping capsules never double-blend.

---

## 📋 Standard Operating Procedure (SOP)

Follow this 5-step process when developing, modifying, or testing this repository:

### Step 1: Environment Verification

Verify that Bun and Cargo/Rust toolchains are installed:

```bash
bun --version
rtk cargo --version
```

### Step 2: Automated Dual-Ecosystem @latest / Pre-Release Upgrades

To run the automated **End-to-End Dual-Ecosystem & Pre-Release Upgrade Pipeline**:

```bash
bun run update-deps
```

- Probes NPM registry for pre-release dist-tags (`next`, `beta`, `rc`, `alpha`, `canary`, `experimental`, `insiders`, `dev`) and latest.
- Probes Crates.io API for `newest_version` across workspace `Cargo.toml` files.
- Automatically executes TypeScript verification, Vite production bundle build, Cargo workspace check, and Cargo workspace unit tests.

### Step 3: Development & Primary Testing

Run the app in live development mode using the ultimate test command:

```bash
bun run tauri dev
```

- **Transparent Full-Screen Click-Through Overlay**: Renders the 4-layer trail, squishy head, shockwave ripples, and orbit satellites directly on the desktop.
- **System Tray Icon**: Left-click to toggle Studio GUI, right-click context menu with Show Settings, Toggle Effects, and Quit.
- **AMOLED 100% Black Settings Dashboard**: SolidJS 2.0 interface with real-time parameter tuning.

### Step 4: Code Quality & Typecheck Verification

Run the compiler checks before committing any changes:

```bash
bun run typecheck
bun run build
rtk cargo check --workspace
rtk cargo test --workspace
```

### Step 5: Git Version Control (Always using `rtk`)

```bash
rtk git status
rtk git add .
rtk git commit -m "feat: your change summary"
rtk git push
```
