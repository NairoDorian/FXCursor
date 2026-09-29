# ⛔ Do not launch the app

> **Status: permanent rule. This is not a suggestion and there are no exceptions.**

## The rule

**Never launch FXCursor.** The user runs it. An agent never runs it — not for a smoke test, not
to "verify a fix", not for a screenshot, not in the background, not hidden, not minimised.

## ⛔ The most destructive mistake: running a bare `fxcursor.exe`

> **Never run `target\debug\fxcursor.exe` (or `target\release\fxcursor.exe`) directly.**
> This is the single worst thing an agent can do here, and it is what damaged the user's OS
> twice. It is called out separately because it looks harmless and is not.

`src-tauri/tauri.conf.json` sets:

```json
"beforeDevCommand": "bun run dev",
"devUrl": "http://localhost:1420"
```

A **debug** build has **no front end bundled into it**. It is compiled to load its UI from the
Vite dev server at `http://localhost:1420`, and `bun run tauri dev` is what starts that server
(`beforeDevCommand`).

So running the binary on its own — with nothing serving port 1420 — produces exactly this:

1. The Studio webview cannot load anything and renders a **webview error page**.
2. The overlay window, which is supposed to be a **fully transparent** click-through surface,
   ends up drawing that opaque error content instead of transparency.
3. The result is a **large opaque window across the user's desktop** that will not go away.
   This happened twice and the user had to **reboot Windows** to recover. The overlay covers the
   whole virtual desktop, so a broken overlay is a broken desktop.

**Therefore: a bare `fxcursor.exe` is not a safe or a partial way to run the app. It is a
broken way to run the app.** There is no "just the overlay" or "headless-ish" invocation.

## The commands that violate it

| ❌ Never run | Why |
| :--- | :--- |
| `target\debug\fxcursor.exe` | **Worst case.** No Vite server → opaque error overlay → stuck window, needed a Windows reboot. Twice. |
| `target\release\fxcursor.exe` | Same class of problem if it is a dev-profile build; do not run it either. |
| `bun run tauri dev` | Launches the app and its windows properly, but it is still the **user's** job. |
| `bun tauri dev` | Same. |
| `bun run tauri build`, any `tauri build` | Same. |
| `Start-Process ...\fxcursor.exe` (any `-WindowStyle`) | Same. `Hidden`/`Minimized` is not a safety measure — see below. |
| `powershell -File scripts\snapshots\snapshot_trail.ps1` | Spawns a **new** `fxcursor.exe --capture …`. See below. |
| `powershell -File scripts\snapshots\snapshot_motion.ps1` | Same. |
| `powershell -File scripts\snapshots\contact_sheet.ps1` | Same. |
| `powershell -File scripts\snapshots\zoom_sheet.ps1` | Same. |
| `fxcursor.exe --capture <png>` | Same. |

## Why, precisely

The overlay is a **full-desktop transparent window**. Every launch puts real windows on the
user's screen while they are working at their machine. This happened **three times in a single
session** and twice left a non-transparent window stuck on the desktop that the user had to
**reboot Windows** to clear.

Two mechanisms caused it, and both are easy to walk into by accident:

### 1. `--capture` is a second process, not a message

The snapshot scripts work by starting a **new** `fxcursor.exe --capture <png>` and relying on
the single-instance plugin to forward that request over IPC to an already-running instance.

**If no instance is running, that "capture" process becomes a primary instance.** The
single-instance callback never fires, so it ignores the capture arguments, starts the app
normally, and — per the `devUrl` problem above — may come up with **no front end at all**, which
is how the opaque overlay happens. That is what left the stuck window on the desktop.

So running a snapshot script is **also** launching the app. There is no "capture only" mode.

### 2. `-WindowStyle Hidden` / `-WindowStyle Minimized` is not a safety measure

Starting the app "gently" still creates the real windows, and with the `devUrl` problem it
still produces the opaque overlay. The window style only hides the evidence from the agent, which
makes it harder to notice the damage and harder for the user to recover. It is forbidden for
exactly the same reason as the plain launch. Do not add it to any command.

## What to do instead

Everything needed to verify this project is headless, and none of it opens a window.

| Goal | Command |
| :--- | :--- |
| Ribbon / trail shape, physics, motion bugs | `rtk cargo test -p fxcursor-render` |
| Rust ⇄ TypeScript trail parity | `bun run fixtures` then `bun test` |
| Config / preset parity | `bun run fixtures` + `bun test` |
| GPU adapter / surface only | `rtk cargo test -p fxcursor-render --test gpu_smoke` |
| All seven gates | `bun run validate` |

`TrailChain`, `build_samples` and `build_layer_capsules` are pure CPU functions with no window
and no GPU context, which is deliberate: **reproduce a motion bug as a unit test** rather than
looking at it on screen.

## When a change cannot be verified headlessly

Say so plainly and **hand the check to the user**:

1. Describe exactly what was changed and what to look for.
2. Ask the user to run the app and report what they see.
3. Record the result as *unverified* in `PROGRESS.md`.

Do not offer to launch the app. Do not launch it "just to check". An unverifiable change is
reported as unverified, not quietly assumed to work.

> Working example: the open crescent-on-the-head bug
> ([`CRESCENT_AT_TRAIL_HEAD.md`](CRESCENT_AT_TRAIL_HEAD.md)) is diagnosed entirely from headless
> CPU measurements, because `TrailChain` / `build_samples` / `build_layer_capsules` need no window
> and no GPU context. Reproduce them with
> `cargo run -p fxcursor-render --example dump_crescent`.
