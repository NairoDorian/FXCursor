# Open bug: crescent / half-circle at the head of the trail

> **Status: OPEN, not fixed.** Handover for the next agent or maintainer.
> Every attempt made so far made the trail worse and was reverted. The ribbon is byte-identical
> to the last known-good build; nothing in this document has been applied.
>
> **Reproduce the numbers:** `cargo run -p fxcursor-render --example dump_crescent`
> (headless, CPU only — see [`DO_NOT_LAUNCH_THE_APP.md`](DO_NOT_LAUNCH_THE_APP.md), the app
> must never be launched to test this).

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

### How to see the result without launching the app
`TrailChain`, `build_samples` and `build_layer_capsules` are **pure CPU with no window and no GPU
context** — that is deliberate. Assertions belong in `cargo test -p fxcursor-render`, e.g.:
- no merged node sits in front of the head (`node.x > head.x` for the first ~8 kept points) after
  a stop;
- the head radius is within some factor (say 1.15×) of the next capsule's radius;
- the visible length does not grow by more than a small percentage across the whole retract.

A previous session also wrote a throwaway CPU rasteriser that composited the real capsules per
layer into a PPM/PNG. It was **removed** during the revert because it added a `png` dev-dependency;
re-adding it is cheap and was genuinely the only way to *see* the shape. It is not in the tree now.

---

## 7. Ground rules for working on this

1. **Never launch the app.** A debug build has no front end bundled (it loads from
   `devUrl: http://localhost:1420`), so running `fxcursor.exe` directly renders a webview error
   page and an **opaque** overlay; that has twice left a stuck non-transparent window over the
   whole desktop that needed a Windows reboot. `bun run tauri dev` is the user's job. Full rule:
   [`DO_NOT_LAUNCH_THE_APP.md`](DO_NOT_LAUNCH_THE_APP.md). Note that `scripts/snapshots/*.ps1`
   also spawn `fxcursor.exe` and count as launching it.
2. **Verify headlessly:** `bun run validate` (7 gates) and `cargo test --workspace`.
3. **Keep `src/lib/trail.ts` in sync** with `renderer.rs`, then `bun run fixtures`, and let
   `test/trail-parity.test.ts` confirm. A physics change on one side only will fail there.
4. **One change at a time**, with the user verifying in the real app between changes.
5. If a change cannot be verified headlessly, say so and hand the check to the user rather than
   opening a window.
