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
>
> `bun run tauri dev` is a sanctioned command **provided Vite is started by it** — that is the
> only correct way to run the app. What is forbidden is a bare `fxcursor.exe` (no front end
> bundled in a debug build → opaque overlay → stuck window). See "Running the app" below.

---

## 🧭 Orientation for agents

> [!CAUTION]
> ## Running the app and judging visual changes
>
> **Running the app with `bun run tauri dev` is allowed and expected.** It starts Vite and
> launches the app with the front end wired up, so the overlay is genuinely transparent. Use it.
>
> **Never run a bare `target\debug\fxcursor.exe`.** `tauri.conf.json` sets `"devUrl":
> "http://localhost:1420"` with `"beforeDevCommand": "bun run dev"`, so a **debug build has no
> front end bundled in**. Run on its own it renders a webview error page, the overlay is drawn
> **opaque** instead of transparent, and the result is a stuck non-transparent window across the
> whole desktop — that happened **twice** and needed a **Windows reboot**. `-WindowStyle
> Hidden`/`Minimized` is not a mitigation; it creates the same windows and only hides the
> evidence.
>
> **Never judge a visual change from CPU rendering.** A CPU rasteriser was used to approve a
> trail fix once; it looked clean and the bug was still there, and worse, in the real wgpu app.
> The GPU feathers edges with `blur` (layer 0: 39–50% of the radius), resolves the union by
> feathered alpha in a depth pre-pass, and composites pre-multiplied. **Capture from the running
> app instead:** start `bun run tauri dev`, confirm it is up, then
> `powershell -ExecutionPolicy Bypass -File scripts\snapshots\snapshot_motion.ps1 -Motion stop`
> (keep `-StartX`/`-StartY` inside the `-Size` crop). The capture scripts are allowed *only* when
> an app is already running — otherwise their `--capture` process becomes a primary instance and
> hits the same no-front-end problem.
>
> Full policy, including what the CPU *is* good for: [`docs/RUNNING_AND_DEBUGGING.md`](docs/RUNNING_AND_DEBUGGING.md).
> One change at a time, verified in the real app between changes — three changes landed as a
> bundle once and the responsible one was never identified.

- The repository root is the **entire** codebase (FXCursor). It is standalone: nothing here is derived from, or refers to, any other project.
- Read `PROGRESS.md` first: it states what is actually implemented versus what `docs/V4_ARCHITECTURE_SPECIFICATION.md` targets, the known issues, and the roadmap.
- Architecture today: **single Tauri process**. The overlay is a transparent Tauri window with a wgpu 30 surface driven by `src-tauri/src/overlay/`; physics is on the CPU. `crates/fxcursor-daemon` is an experimental prototype that the app does not launch.
- Configuration lives in `crates/fxcursor-protocol` (`AppConfig`, presets, self-healing). `src/lib/bindings.ts` is **generated** by `tauri-specta` on every debug run; `src/lib/presets.ts` keeps the TypeScript defaults mirror for browser preview mode, and the built-in presets are **generated** into `src/lib/generated/builtin_presets.json` (never edit it by hand). When you change the Rust struct or presets: give new fields a `#[serde(default)]`, update the defaults in `presets.ts`, run `bun run fixtures`, and let the parity tests (`test/config-parity.test.ts`, `crates/fxcursor-protocol/tests/fixture_parity.rs`) confirm both sides agree. Out-of-range values are clamped by `AppConfig::sanitize()` (`crates/fxcursor-protocol/src/sanitize.rs`); extend it with every new numeric field.
- The trail physics is `TrailChain` in `crates/fxcursor-render/src/renderer.rs` and is GPU-free on purpose: reproduce motion bugs as unit tests there (stop/retract, settle, flick, LazyBrush, resting-dot scenarios exist) before touching the code. Its TypeScript mirror is `src/lib/trail.ts` (used by the live preview): change both, run `bun run fixtures`, and `test/trail-parity.test.ts` must pass against the regenerated `test/fixtures/trail_trace.json`.
- **Numerical behaviour is pinned by headless tests** (`bun run validate`, `cargo test --workspace`), but **visual changes are confirmed on a capture from the real wgpu app** — never on a CPU approximation of the renderer. See "Running the app and judging visual changes" above.
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
