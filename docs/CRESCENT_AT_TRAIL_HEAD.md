# Open bug: crescent / half-circle at the head of the trail

> **Status: OPEN, not fixed.** Handover for the next agent or maintainer.
> Every attempt made so far made the trail worse and was reverted. The ribbon is byte-identical
> to the last known-good build; nothing in this document has been applied.
>
> **How to work on this:** read [`RUNNING_AND_DEBUGGING.md`](RUNNING_AND_DEBUGGING.md) first.
> `bun run tauri dev` **is allowed** and is the only trustworthy place to judge a visual change.
> **Do not judge a fix from CPU rendering** — a CPU rasteriser was used for exactly that during a
> previous attempt and it showed a clean picture for a fix that was still broken, and worse, in
> the real app. The numbers below are valid (they come from `layer_sample_style`, the same code
> the GPU uses); the *pictures* produced from them were not, and that mistake is what this note
> now leads with.

---

## 1. What the user sees

Moving the pointer normally is fine. **When the pointer stops**, at the cursor position a
**half-circle / crescent / semi-circle** appears at the **head** of the trail, oriented along the
direction the pointer was last travelling. It **jitters** frame to frame.

The user described it as "half a circle at the cursor position when I stop moving the cursor and
the trail is catching up with the cursor position", and confirmed it is worse in the middle of
the retraction than at the end.

The user also reports that an earlier legacy web/JS implementation they worked from did **not**
have this artifact.

---

## 2. Where the code is

| Concern | Location |
| :--- | :--- |
| Physics (spring chain, LazyBrush) | `TrailChain` in `crates/fxcursor-render/src/renderer.rs` |
| Near-duplicate node merge | `SampleScratch::build` in the same file, `MIN_NODE_SPACING = 0.25` |
| `progress` computation | `SampleScratch::build` — `(i + t) / (N - 1)` on the **original node index** |
| Width / alpha / blur per sample | `SampleStyle::new` and `layer_sample_style` |
| Capsule generation | `build_layer_capsules_styled` |
| GPU union + depth pre-pass | `crates/fxcursor-render/src/shaders/render.wgsl` |
| TypeScript mirror (Studio preview) | `src/lib/trail.ts` — **must stay in sync** |
| Parity fixture | `test/fixtures/trail_trace.json`, checked by `test/trail-parity.test.ts` |

Relevant constants: `MIN_NODE_SPACING = 0.25`, `HEAD_REST_DISTANCE = 0.1`,
`MAX_NODE_GAP = 512.0`, `REFERENCE_FRAME = 1/120`, `PHYSICS_TICK = 1/120`, `MIN_VISIBLE_ALPHA = 0.004`.

Defaults: `length 80`, `spring 0.05`, `damping 0.7`, `head_spring 0.05`, `head_damping 0.7`,
`cursor_size 40`, `min_width 2`, `interpolation_steps 2`, `fade_mode 3` (Sigmoid),
`adaptive_quality true`. **Head stroke half-width is 30.0 px** (`cursor_size 40 × layer 0
width_factor 1.5 ÷ 2`) — keep this number in mind, it is the scale of the problem.

---

## 3. Measurements (current, unmodified code)

Script above, 80 nodes, run right at 30 px/frame (1800 px/s) for 40 frames then a dead stop:

```
frame  headGap  samples  progress jump   layer0 radii (a->b)   nodes ahead of head
   40   106.15       59           0.003  40.4->40.4  40.4->40.4      0.00 px
   70     0.00       74           0.051  29.5->29.3  29.3->29.2      2.59 px
  100     0.00       78           0.177  29.5->28.0  28.0->27.8      4.00 px
  130     0.00       83           0.316  29.5->24.4  24.4->24.0      4.08 px
  160     0.00       77           0.468  29.5->16.9  16.9->16.2      4.00 px
  190     0.00       61           0.620    29.5->8.3    8.3->7.7      3.75 px
```

Three facts fall out, and **all three contribute**:

**(a) The centreline folds.** A merged node settles up to **~4.1 px in front of the head** and
stays there for the whole retraction. A hairpin ~4 px wide drawn with a stroke ~60 px across
self-intersects in the union, which puts a **notch** in the outline.

**(b) `progress` jumps hard at the head.** The gap between the first and second *drawn* sample
grows from 0.003 while moving to **0.62** by the end of the retraction. When a third of the
chain has collapsed onto the cursor, a huge slice of the `0..1` range is consumed within a couple
of pixels.

**(c) The head capsule is therefore far fatter than the ribbon behind it.** Layer-0 radii go
`29.5 → 24.4` and then down to `8.3`, i.e. the first capsule is **3.5× wider than the last
before it**. A round cap that much wider than the tube it joins is what reads as a crescent.
This is the single biggest contributor.

The jitter comes from (a) and (b) re-resolving every frame as nodes cross the 0.25 px merge
threshold: a node that is merged on one frame and kept on the next changes the sample count and
the radius sequence, so the outline twitches.

**Note the ordering issue:** the visible artifact appears while the *tail* is still catching up
(the retraction takes ~3 s at these settings), which is why the user sees it "when the trail is
catching up".

---

## 4. What was ruled out (do not re-investigate)

### Draw order — impossible
The shader keeps **one winner per pixel per layer**. The depth pre-pass stores
`depth = (layer + 0.001 + seg_bias + alpha*0.99) / LAYER_BANDS` and the colour pass only shades
fragments whose depth beats it. The result is a union, so reversing the order in which capsules
are submitted changes nothing. Confirmed in the source, not inferred.

### Per-segment Z depth — pointless
Same reason. `alpha * 0.99` dominates the `seg_bias` tiebreaker and alpha peaks at the head
(fade → 1 at `progress 0`), so **the head already wins** every tie and is already drawn in front
of the other nodes. The user's own suggestion was reasonable; the code already does it.

### The `arc`-only taper (progress by drawn length)
Using drawn arc length for *everything* removed the crescent completely (head radii became
`29.5 → 30.4 → 31.1`, monotonic) but made the ribbon **breathe**: the visible length pulsed
`+6.2 px` on **15 of 240** frames after the stop, because opacity then stops tracking the chain.
Measured, rejected.

---

## 5. What was tried and reverted

All of these are reverted. `build_samples` is byte-identical to the last known-good build and the
only renderer diff is the parameter-unit change (`spring / 1000` → `spring`, etc.). **The user
reported the trail became jittery everywhere and that this broke the whole ribbon.**

> ⚠️ **The central mistake, and the reason the fix has to be redone.** These were judged on **CPU
> rasterisations** that looked clean. Run in the real app under `bun run tauri dev` with wgpu,
> the artifact was **still there and worse**. See
> [`RUNNING_AND_DEBUGGING.md`](RUNNING_AND_DEBUGGING.md) §2 for exactly why a CPU picture is
> not a proxy: the GPU feathers edges with `blur` (layer 0 ships 0.39–0.50 of the radius), resolves
> the union by **feathered alpha** in a depth pre-pass, and composites pre-multiplied. None of
> that was modelled, so "it looked right" meant nothing.
>
> The **numbers** in §3 are still sound — they are read from `layer_sample_style`, the same code
> that fills the GPU instance buffer. It was only the *visual* judgement that was invalid.

### Attempt 1 — collapse the head cluster at rest
*Fold the nodes within 8 px of the head onto it, once the head has arrived, zeroing their
velocity.*
- **Result: worse.** Snapping and zeroing velocity every frame is a stick-slip cycle: the
  cluster re-accelerates from the springs, gets snapped again, and nodes pop across the
  threshold — exactly the "more jitter" the user reported.
- Reverted.

### Attempt 2 — duplicate the endpoint phantom
*The head segment's first Bézier control point comes from `lerp(p0, p2)`. With a **mirrored**
phantom (`2*p0 - p1`) that control point sits on the far side of the head, pulling the curve
forward; the curve bulges out before turning back.*
- Looked correct in isolation and removed a ~4 px forward bulge.
- Shipped together with attempts 3 and 4. Reverted with them.

### Attempt 3 — fold-removal pass
*Drop any merged point that reverses direction (negative dot product between consecutive
spans), removing hairpins in one pass.*
- Correct in principle for (a), but it **deletes points near the reversal threshold**, and
  whether a given node is above or below that threshold flips frame to frame. Net effect was
  the pervasive jitter.
- Reverted. **Strong suspect for the "jitter everywhere" the user reported** — it was the most
  invasive of the three.

### Attempt 4 — split the taper (`progress` for opacity, new `arc` for width/blur)
*Add a second progress measure, position by drawn length, and drive width and blur from it;
leave opacity on the node index.*
- This is the most promising idea and it **did work in the rasteriser**: head radii went from
  `29.5 → 24.4 → 16.9` to `29.5 → 30.4 → 31.1`, and the retract shimmer dropped from 15/240
  frames (+6.2 px) to **2/240 (+2.1 px)**.
- It was landed together with attempts 2 and 3. The user then reported the GPU trail was
  "so bugged, even more bugs". **Which of the three was responsible was never isolated** — that
  is the key thing the next attempt must not repeat.
- Reverted.

> ⚠️ **Lesson:** three changes were landed as a bundle and the regression was only noticed in the
> real app. **Change one thing at a time**, and verify with `bun run validate` **and** have the
> user run the app between changes.

---

## 6. Where the next attempt should start

**The one idea that was never tried: limit the *width* taper rate per pixel of arc length.**

Everything above either changed the centreline, the merge, or the opacity profile — all three of
which are load-bearing and all three caused the regression. The width profile is the part that
actually produces the visible crescent, and nothing has touched it in isolation.

Sketch: in `layer_sample_style` (or as a post-pass over the samples), cap how much the radius
may change between consecutive samples relative to the arc length between them, e.g.

```
max |r[i+1] - r[i]| <= RATE * max(distance(i, i+1), MIN_DISTANCE)
```

That is smooth, frame-rate independent, and cannot delete geometry — it only limits how fast the
silhouette may change. It addresses (c) directly and leaves (a) and (b) alone, so it cannot
reintroduce the hairpin or the flicker. Start with a rate that is generous enough to be a no-op
during normal motion, so behaviour only changes where the taper is too steep.

**If that is not enough, then address (a) and (b) separately, one per change**, with the user
verifying each. Possible directions, in order of increasing risk:

1. **Width rate limit** (above) — touches only the silhouette.
2. **Interpolate `progress` across a merged run.** When the merge collapses nodes `a..b` into one
   point, that point currently keeps the *original* index of the surviving node, so the taper
   jumps. Spreading the run's index range across the neighbouring drawn points would make (b)
   continuous without changing the centreline. Careful: the existing behaviour of `progress` is
   load-bearing for the retraction (see `PROGRESS.md`; an earlier arc-length attempt was reverted
   for making the ribbon "breathe").
3. **Only then** reconsider fold removal, and if so make it hysteretic (a node must clear the
   threshold by some margin before being dropped) so it cannot flip frame to frame.

### How to see the result

**Capture from the real app.** Start it with `bun run tauri dev` (Vite up, transparent overlay),
confirm it is running, then:

```powershell
powershell -ExecutionPolicy Bypass -File scripts\snapshots\snapshot_motion.ps1 `
  -Motion stop -StartX 250 -StartY 300 -Speed 0.9 -Size "900x520" -Burst 12 -IntervalMs 90
```

Keep `-StartX`/`-StartY` **inside** the `-Size` crop, or the trail is off-frame and you get a
blank PNG. Each capture also logs the live chain from `chain_summary()`, so you can read the
head offsets and node count for the exact frame you are looking at.

A burst matters: the artifact is transient, so a single frame proves nothing either way.

### Regression tests (numbers, not pictures)

`TrailChain` / `build_samples` / `build_layer_capsules` are pure CPU, so assertions belong in
`cargo test -p fxcursor-render` and run in CI:

- no merged node sits in front of the head after a stop;
- the head radius is within some factor (say 1.15×) of the next capsule's radius;
- the visible length does not grow by more than a small percentage across the whole retract.

`cargo run -p fxcursor-render --example dump_crescent` prints the current numbers (the table in
§3) and is fine for that. **It is a geometry dump, not a renderer** — it cannot tell you what
the artifact looks like, and must never be used to claim a visual fix works.

---

## 7. Ground rules for working on this

1. **Run the app with `bun run tauri dev`.** That is allowed and is the correct way. **Never run a
   bare `fxcursor.exe`**: a debug build has no front end bundled (it loads from
   `devUrl http://localhost:1420`), so the webview renders an error page and the overlay is drawn
   **opaque** — that has twice left a stuck non-transparent window over the whole desktop needing
   a Windows reboot. `scripts/snapshots/*.ps1` are allowed too, but only once an app is already
   running, because their `--capture` process becomes a *primary* instance (and hits the same
   no-front-end problem) if none is. Full rule: [`RUNNING_AND_DEBUGGING.md`](RUNNING_AND_DEBUGGING.md).
2. **Judge every visual change on a capture from the real wgpu app.** A CPU approximation of the
   renderer is not evidence — that mistake invalidated a whole round of fixes here.
3. **One change at a time**, verified in the real app between changes. Three landed as a bundle
   once and the responsible change was never identified.
4. **Keep `src/lib/trail.ts` in sync** with `renderer.rs`, then `bun run fixtures`, and let
   `test/trail-parity.test.ts` confirm. A physics change on one side only will fail there.
5. Headless gates are the first line of defence, not a substitute for looking at the app:
   `bun run validate`, `cargo test --workspace`.
6. If something cannot be checked, say so plainly rather than substituting an approximation.
