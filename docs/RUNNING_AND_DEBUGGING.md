# Running and debugging the app

> **How to run FXCursor, and how to actually see what it draws.**
> This file supersedes the earlier blanket "never launch the app" rule, which was too broad and
> had the side effect of pushing debugging onto a CPU path that does **not** match the GPU.

---

## 1. Running the app: allowed, with Vite

`bun run tauri dev` **is allowed.** It is the correct way to run the app: it starts the Vite dev
server (`beforeDevCommand`) and then launches the binary with the front end wired up, so the
Studio UI loads and the overlay is genuinely transparent.

### The one thing that is forbidden

**Never run a bare `target\debug\fxcursor.exe` (or `target\release\fxcursor.exe`) without the Vite
server running.**

`src-tauri/tauri.conf.json` sets:

```json
"beforeDevCommand": "bun run dev",
"devUrl": "http://localhost:1420"
```

A **debug** build has **no front end bundled into it**. It is compiled to load its UI from the
Vite dev server on port 1420, and `bun run tauri dev` is what starts that server. Run the binary
on its own and:

1. the Studio webview loads nothing and renders a **webview error page**;
2. the overlay, which must be a **fully transparent** click-through surface, draws that opaque
   content instead;
3. the result is a **large opaque window across the user's desktop** that will not go away.

This happened **twice** during development and the user had to **reboot Windows** to clear it.
The overlay covers the whole virtual desktop, so a broken overlay is a broken desktop. There is no
"just the overlay" invocation, and **no front end means the overlay is not transparent.**

| ❌ Forbidden | ✅ Allowed |
| :--- | :--- |
| `target\debug\fxcursor.exe` on its own | `bun run tauri dev` |
| `Start-Process ...\fxcursor.exe` with any `-WindowStyle` | — |
| any `tauri dev` / `tauri build` without Vite | — |

`-WindowStyle Hidden` / `-WindowStyle Minimized` is **not** a mitigation. It creates the same real
windows and still hits the missing-front-end problem; it only hides the evidence from the agent,
which makes the damage harder to spot and to recover from. Do not use it.

### The snapshot scripts are allowed — with one condition

`scripts/snapshots/*.ps1` send a capture request **to an already-running instance** via the
single-instance plugin. That is fine and it is the intended way to grab frames.

**Condition: an app started with `bun run tauri dev` must already be running.** If no instance is
running, the `--capture` process becomes a *primary* instance — it ignores the capture arguments
and starts the app normally, with no Vite, i.e. the opaque-overlay case above. Always start the
app first, confirm it is up, and only then run a capture script.

---

## 2. Debugging a visual bug: use the real GPU pipeline

**Never conclude that a visual fix works from CPU rendering.** This is the most important rule in
this file, and it was learned the hard way.

A previous attempt built a CPU rasteriser that composited the capsule geometry and used it to
judge whether a fix looked right. **It produced clean pictures for a fix that was still broken,
and worse, in the real app.** The pictures were fiction.

### Why a CPU rasteriser is not a proxy for the GPU

The GPU path (`crates/fxcursor-render/src/shaders/render.wgsl`) does things a CPU approximation
does not:

- **Feathered edges from `blur`.** `edge = 1 - smoothstep(1 - effective_blur, 1, v)` with
  `effective_blur = max(blur, fwidth(v) * 1.5)`. Layer 0 ships `start_blur 0.39` → `end_blur 0.50`,
  so its edge ramps across **39–50 % of the radius** — a very soft falloff. A hard 1-pixel
  coverage edge (what the CPU rasteriser drew) has nothing like the same silhouette.
- **Depth pre-pass resolving on `alpha`, not coverage.**
  `depth = (layer + 0.001 + seg_bias + alpha * 0.99) / LAYER_BANDS`, and the colour pass only
  shades fragments that beat it. Which capsule wins a pixel therefore depends on the *feathered*
  alpha, so the visible union differs from a per-layer coverage max.
- **Pre-multiplied alpha** and premultiplied compositing order.
- **`r = max(mix(radii.x, radii.y, t), 0.35)`** and `MIN_VISIBLE_ALPHA` culling interacting with
  the feather.
- Per-vertex `blur` interpolation along each capsule.

So: **CPU geometry assertions are valid; CPU pictures are not.** Numbers that come out of
`layer_sample_style` (radii, `progress`, sample counts, node positions) are the same numbers the
GPU uses. Anything about *how it looks* must come from the GPU.

### The trustworthy pipeline

1. `bun run tauri dev` — real Vite, real wgpu surface, real shaders.
2. Capture a real frame:
   - `powershell -File scripts\snapshots\snapshot_motion.ps1 -Motion stop -Burst 12`
     (or `snapshot_trail.ps1` for shapes/hairpins/zig-zags/loops).
   - `contact_sheet.ps1` / `zoom_sheet.ps1` for reviewing a burst.
3. Look at the PNGs. **This is the only accepted evidence that a visual change worked.**

`src-tauri/src/capture.rs` makes this first-class: it calls the real
`renderer.render(device, queue, &view, ...)`, then `copy_texture_to_buffer` + `map_async` to read
the frame back. A crop is just a temporary `virtual_origin` change, so it exercises the identical
shaders. Every capture also logs the live chain state:

```
[capture] 900x520 at (x, y): N capsules, M billboards; chain nodes=80 first5=(..) (..) ...
```

which is `OverlayRenderer::chain_summary()` — head offsets, farthest node, moving count, brush
offset. Use it to correlate what you see with what the physics is doing, in the same frame.

### What the CPU is still good for

`TrailChain`, `build_samples` and `build_layer_capsules` are pure CPU with no window and no GPU
context, and that is deliberate. Use them for **numeric regression tests** — "no node sits in
front of the head", "the head radius is within 1.15× of the next capsule's radius", "the visible
length never grows" — so a regression is caught by `cargo test -p fxcursor-render` in CI. Just do
not let a CPU picture stand in for the real renderer.

---

## 3. Rules of engagement

1. **Run with `bun run tauri dev`, or not at all.** Never a bare binary.
2. **Capture from the running app** to judge anything visual. Start the app first, confirm it is
   up, then run a capture script.
3. **One change at a time**, verified in the real app between changes. Three changes landed as a
   bundle once, the regression was only seen in the real app, and the responsible change was
   never isolated. Do not repeat that.
4. **Keep `src/lib/trail.ts` in sync** with `renderer.rs`, run `bun run fixtures`, and let
   `test/trail-parity.test.ts` confirm — a physics change on one side only fails there.
5. Headless gates remain the first line of defence: `bun run validate`, `cargo test --workspace`.
6. If something cannot be checked, say so and hand the check to the user. Do not guess, and do not
   substitute a CPU approximation for the real renderer.
