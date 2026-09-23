# HANDOVER — Trail physics rewrite + LazyBrush (COMPLETE)

> [!WARNING]
> **Historical document (session 9).** Parts of this plan were revised after comparing against the
> legacy code line by line (sessions 10–11, see `PROGRESS.md` §2.13–2.14 and `CHANGELOG.md`):
> `REFERENCE_FRAME` is 1/120 (not 1/60); the distance clamp is 512 px, inelastic, teleport-only
> (not 64 px with velocity injection); sample progress is the **original node index** (the
> arc-length post-pass of §6.4 was reverted); velocity normalisation is `/20`; a collapsed chain
> draws a resting dot. The TypeScript mirror now lives in `src/lib/trail.ts` and is tested
> against `test/fixtures/trail_trace.json`. Treat the code and PROGRESS.md as the source of truth.

> **Status**: fully implemented, validated, and documented (session 9 + head-bug fix).
> All §6–§11 steps done: LazyBrush, Windhawk spring-damper, distance constraint,
> arc-length progress, head-wins tiebreak, UI, tests, docs. `bun run validate` 7/7 green
> (60 cargo + 17 bun tests), clippy clean. The raw-pointer prepend in `build_ribbon` /
> LivePreview was removed (root cause of the head blob). Nothing committed since `089e5d8`.
> Optional follow-up: retune `head_spring` / `head_damping` slider defaults in `TrailTab.tsx`
> if the softer spring lag feels too loose; smoke-test visually via `bun run tauri dev`.

---

## 1. Mission (user request, verbatim intent)

1. **Fix the trail effect "once and for all."** User reports it has been glitchy since the
   joint-rounding / 90°-turn fixes: overcomplicated, laggy, jittery, and **width scaling broken**.
2. **Study the legacy implementations** (`legacy/original_mods/`, `legacy/project_cursor/`,
   `legacy/TD_Web_Trail/`) and apply their teachings.
3. **Implement the LazyBrush effect, rewritten from scratch**, based on `legacy/TD_Web_Trail`.

Do **not** commit unless the user explicitly asks.

---

## 2. Repo state

- Root: `C:\Users\Z\Downloads\PROJECTS\Cross_Platform_Rust_WebGPU_CursorFX` (Windows, PowerShell).
- Last commit: `089e5d8` (session-7 hardening). Everything since (GPU cursor bypass **and** this
  trail investigation) is **uncommitted**.
- Working tree already dirty from the *previous* (completed) GPU-cursor task: `config.rs`,
  `renderer.rs`, `render.wgsl`, `lib.rs`, `overlay/mod.rs`, `App.tsx`, `HeadTab.tsx`, `bindings.ts`,
  `presets.ts`, fixtures, docs — all modified; `crates/fxcursor-render/src/cursor.rs` and
  `src-tauri/src/cursor.rs` are new/untracked. **Leave that work intact.**
- Baseline quality gates were GREEN before this task: `bun run validate` (7 gates), clippy clean,
  **60 cargo tests + 17 bun tests**.

### Command rules (from AGENTS.md — non-negotiable)

- **Bun only.** Never npm/yarn/pnpm. Dev launch: `bun run tauri dev`.
- Prefix shell commands with **`rtk`** (`rtk cargo check --workspace`, `rtk git status`…).
  Do **not** prefix `bun run …` scripts.
- Note: `rg` (ripgrep) is **NOT installed** in this shell — use the Grep tool, not `rg`.
- Full validation: `bun run validate` = `bun scripts/before-commit.ts --full` (7 gates:
  tsc, oxlint, bun test, vite build, cargo check, cargo test, `bun run arch`).
- After changing Rust config structs: update `presets.ts`, run `bun run fixtures`, let parity
  tests confirm. Run `bun run arch` after touching files so `ARCHITECTURE.md` stays in sync.

---

## 3. Diagnosis (why the trail is bad today)

All code under discussion: `crates/fxcursor-render/src/renderer.rs` (`TrailChain`, lines ~200–470;
`build_samples` ~586–686; tests ~1961–2256) and `src/components/Preview/LivePreview.tsx`.

| Symptom | Root cause |
| :-- | :-- |
| **Lag** | Cascade of ~6 serial low-pass filters: `head_target` EMA (`head_damping`) → head first-order `follow_step` (`head_spring`) → **4 `lead_nodes` pursuit followers**, each another first-order filter. Legacy Windhawk/TD use a single 2nd-order spring-damper (zero steady-state velocity lag). |
| **Jitter** | `block_overtake` "wall" logic: inelastic stop against a per-node wall, wall flips on direction change → stick-slip; plus `cursor_dir` threshold and displacement-derived `speed` noise feeding `velocity_width`. |
| **Width scaling broken** | `Sample.progress` is **segment-index-based** (`(seg + t) / total_segments`, renderer.rs:674), not arc-length. Near-coincident merge (`MIN_NODE_SPACING = 0.25`) + phantom/adaptive sampling distort index→length mapping, so fade / blur / width taper are uneven along the ribbon. |
| **90° / joint artifacts** | `render.wgsl:122` depth tiebreaker `seg * 1e-6` (up to `0.016` > `1e-4` epsilon): at a fold, **tail** segments (higher `seg`) beat **head** segments in the depth pre-pass. Head must win ties (consistent with LivePreview head-last). |

**Judgment: the capsule-union + depth/stencil render pass is geometrically sound — keep it.**
(max-coverage union = correct union; legacy needed normal stabilization only because of triangle
strips.) Only the *tiebreaker sign* is wrong.

### What to REMOVE from `TrailChain`

- `block_overtake`, `type Wall`, `OVERTAKE_MARGIN`, `motion_dir`, `MIN_DIRECTION_STEP`
- `cursor_dir` field + its threshold logic in `advance`
- `head_target` EMA (replaced by LazyBrush + spring-damper)
- `follow_step` (first-order pursuit) and the `lead_nodes` loop branch
- `TrailNode.dir` field (only existed for walls) — keep `speed` (drives `velocity_width`)
- `config.lead_nodes` everywhere

---

## 4. Legacy teachings (already extracted — do not re-derive)

### 4a. Windhawk D3D mod — `legacy/original_mods/D3D_cursor_mod.wh.cpp` lines 1751–1808

**Proven spring-damper, this is the target formulation:**

```cpp
// HEAD (ApplyTrailPhysicsSegment):
head.vx += (targetX - head.x) * headSpring * dt_scale;
head.vy += (targetY - head.y) * headSpring * dt_scale;
head.vx *= headFric;  head.vy *= headFric;
head.x  += head.vx * dt_scale;  head.y += head.vy * dt_scale;
head.speed = sqrt(vx*vx + vy*vy);

// BODY (ApplyTrailPhysicsChain), i = 1..len:
if (i > 1) { cur.v += (pp - cur) * bodySpring * 0.3 * dt_scale; }  // 2nd-neighbour coupling
cur.v     += (prev - cur) * bodySpring * dt_scale;
cur.v     *= bodyFric;
cur.p     += cur.v * dt_scale;

// params:
headSpring  = headSpring_setting / 1000.0;
headFricRaw = 1.0 - (headFriction_setting / 100.0);
headFric    = powf(headFricRaw, dt_scale);        // exponential friction → frame-rate independent
bodySpring  = bodySpring_setting / 1000.0;
bodyFric    = powf(1.0 - bodyFriction/100.0, dt_scale);
dt_scale    = clamp(g_deltaTime / (1/60), 0.1, 5.0);
```

Windhawk **defaults** (from file header comments / settings): `headSpring: 50`, `headFriction: 30`,
`spring: 50` (body), `friction: 30`, `trailLength: 100`. Same numbers V4 already uses
(`head_spring: 50`, `head_damping: 30`, `spring: 50`, `damping: 30`) — but V4 currently applies
them through first-order filters, **not** through this spring-damper. Switching to this formula
with the *existing* default numbers reproduces the legacy feel.

No overtake rule, no walls, no lead nodes in the legacy code — the spring-damper alone was enough.

V3 (`legacy/project_cursor/src-tauri/src/overlay/renderer.rs:507–555`) used the identical
formula (with `dt_scale = dt / (1/120)` and a `position_skip` frame skip — skip is **not**
recommended to port; Windhawk itself marks it "not adopted").

### 4b. TD_Web_Trail — `legacy/TD_Web_Trail/trail-system.js`

**LazyBrush (lines 18–133)** — dead-zone smoother between raw pointer and brush:

```js
// per update, if enabled:
distance = dist(pointer, brush);
isOutside = round((distance - radius) * 10) / 10 > 0;   // snap to 0.1px, avoid float jitter
if (isOutside) brush.moveByAngle(angle_to_pointer, distance - radius, friction);

// moveByAngle with friction f ∈ (0,1):
u = 1 - f;
factor = 1 - Math.sqrt(1 - u * u);   // f=0 → factor 1 (instant), f→1 → factor 0 (frozen)
brush.x += cos(angle) * distance * factor;
```

- Defaults: `radius = 30`, `friction = 0.4`, `enabled = false`.
- `f=0` must be treated as *no friction* (full move) — JS uses truthiness; in Rust use
  `if friction <= 0 { factor = 1.0 }`.
- When disabled: `brush = pointer` exactly.

**Distance constraint (clamped) — lines 431–476**, applied head→tail after the spring pass:

```js
for i in 1..n:
  dist = dist(nodes[i-1], nodes[i]);
  if (dist > constraintDist) {
    ratio = constraintDist / dist;
    newX = prev.x + dx * ratio;  newY = prev.y + dy * ratio;
    curr.dx += newX - curr.x;    // velocity correction (carry the clamp into velocity)
    curr.dy += newY - curr.y;
    curr.x = newX;  curr.y = newY;
  }
```

- TD default `constraintDist = 15` at width 25 / 50 points; feature was **off by default**.
- For FXCursor: use a **const**, not a config field — `MAX_NODE_GAP = 64.0` px (plan decision:
  80 nodes × 64 px ≈ 5120 px max reach; generous, engages only on flicks/teleports).
- Apply velocity correction as `cur.vx += (new - old) / dt_scale` (positions are px, `vx` is
  px per reference frame; `/dt_scale` keeps it frame-rate consistent — see §6.3 step 5).
  TD runs at fixed 60 Hz so it adds raw; we sub-step, so divide.

Other TD teachings (optional, not required by the plan): power taper `(1-p)^1.5`,
tail-opacity floor, idle-sleep — V4 already has fade curves; **do not add** these now.

---

## 5. Config changes (`crates/fxcursor-protocol/src/config.rs`)

`TrailConfig` (lines 55–90):

- **REMOVE**: `lead_nodes` field, `#[serde(default = "default_lead_nodes")]`, and
  `fn default_lead_nodes()` (lines 68–71, 92–95), and the `lead_nodes: 4` in `Default` (line 106).
- **KEEP** `head_spring`, `head_damping` — they now mean spring constant (÷1000) and friction
  percent (same as body), matching Windhawk. Update their doc comments accordingly:

```rust
/// Spring stiffness constant for the leading head node (same scale as `spring`: /1000).
pub head_spring: f32,
/// Velocity friction percent for the leading head node (0–99; higher = more damping).
pub head_damping: f32,
```

- **ADD** (all `#[serde(default)]`, placed after `head_damping`):

```rust
/// LazyBrush: engage the TD-style dead-zone pointer smoother.
#[serde(default)]
pub lazy_enabled: bool,
/// Dead-zone radius in px the pointer must exceed before the brush starts moving.
#[serde(default = "default_lazy_radius")]
pub lazy_radius: f32,
/// Brush friction 0–0.99: fraction of the excess distance NOT applied per frame
/// (0 = snap to pointer, →1 = frozen). TD formula: factor = 1 - sqrt(1-(1-f)^2).
#[serde(default = "default_lazy_friction")]
pub lazy_friction: f32,
```

Defaults helper fns + `Default for TrailConfig`:

```rust
fn default_lazy_radius() -> f32 { 30.0 }
fn default_lazy_friction() -> f32 { 0.4 }
// in Default: lazy_enabled: false, lazy_radius: 30.0, lazy_friction: 0.4,
```

**Decision: `lazy_enabled` default = `false`** (opt-in; matches TD default; user can toggle in UI).

### Ripple effect on other files (must all be updated together)

| File | Change |
| :-- | :-- |
| `crates/fxcursor-protocol/src/presets.rs` | 5 preset literals have `lead_nodes: 4` (lines 38, 135, 221, 328, 425). **Struct literals require every field** → delete those 5 lines and add `lazy_enabled: false, lazy_radius: 30.0, lazy_friction: 0.4` to each `TrailConfig { … }`. |
| `src/lib/presets.ts` | `getDefaultConfig().trail`: remove `lead_nodes: 4` (line 43), add the three lazy fields after `head_damping`. Preset overrides (`getPresetById`) do **not** mention `lead_nodes` — no change there. |
| `src/lib/bindings.ts` | **Generated file.** Specta marks `#[serde(default)]` fields **optional** (`gpu_cursor?: GpuCursorConfig` is the precedent). Hand-edit to match: delete `lead_nodes?: number` + its doc comment (lines 316–320); after `head_damping: number` add `lazy_enabled?: boolean, lazy_radius?: number, lazy_friction?: number` with doc comments. Will be regenerated authoritatively on next `bun run tauri dev`. |
| `test/fixtures/default_config.json` | Regenerate via `bun run fixtures` (runs `dump_default` example). |
| `test/config-parity.test.ts` | No structural change needed (deep-equality vs fixture). |
| `src/components/Tabs/TrailTab.tsx` | Local `TrailConfig` interface: remove `lead_nodes?: number`; add `lazy_enabled?: boolean; lazy_radius?: number; lazy_friction?: number`. See §7 for UI. |
| `src/components/Preview/LivePreview.tsx` | Uses `cfg.trail.lead_nodes` (line 338) — rewrite physics, see §8. |
| `crates/fxcursor-render/src/shaders/physics.wgsl` | Prototype, **never dispatched** — leave `head_spring`/`head_damping` names as-is; no change required. |
| `README.md:60`, `PROGRESS.md:25` & `:120`, `CHANGELOG.md:69`, `repomix-instruction.md:8` | Describe `lead_nodes` / no-overtake — rewrite wording (see §9). |

---

## 6. Physics rewrite — `crates/fxcursor-render/src/renderer.rs`

### 6.1 Structs / constants

```rust
#[derive(Clone, Copy, Debug, Default)]
struct TrailNode {
    x: f32, y: f32,
    vx: f32, vy: f32,   // px per reference frame (1/60 s)
    speed: f32,
    // DELETE: dir
}
// finish_step(): only refresh speed = hypot(vx, vy); drop motion_dir branch.

const MIN_DIRECTION_SPEED: f32 = 0.05;   // keep (is_moving)
// DELETE: MIN_DIRECTION_STEP, OVERTAKE_MARGIN, type Wall, block_overtake, motion_dir

/// Largest gap (px) between consecutive nodes; beyond this the follower is clamped back
/// (TD-style clamped distance constraint). Bounds overshoot / stretch on flicks.
const MAX_NODE_GAP: f32 = 64.0;

/// TD LazyBrush friction factor: how much of the excess distance the brush covers.
/// f <= 0 → 1.0 (snap); f >= 1 → 0.0 (frozen); else 1 - sqrt(1 - (1-f)^2).
fn lazy_brush_factor(friction: f32) -> f32 {
    if friction <= 0.0 { return 1.0; }
    if friction >= 1.0 { return 0.0; }
    let u = 1.0 - friction;
    1.0 - (1.0 - u * u).sqrt()
}

#[derive(Debug, Default)]
struct TrailChain {
    nodes: Vec<TrailNode>,
    brush: (f32, f32),      // LazyBrush position (== pointer when lazy off)
    brush_valid: bool,
    // DELETE: head_target, head_target_valid, last_cursor, cursor_dir
}
```

### 6.2 `advance` (once per frame)

```rust
fn advance(&mut self, x: f32, y: f32, dt: f32, config: &AppConfig) {
    if self.nodes.is_empty() { return; }
    if !self.brush_valid { self.brush = (x, y); self.brush_valid = true; }
    if !config.trail.lazy_enabled { self.brush = (x, y); }
    let substeps = ((dt / PHYSICS_TICK).ceil() as usize).clamp(1, MAX_SUBSTEPS);
    let sdt = dt / substeps as f32;
    for _ in 0..substeps { self.step(x, y, sdt, config); }
}
```

`reset()` → `nodes.clear(); brush_valid = false;`
`resize()` → same as now, minus `dir` (TrailNode no longer has it).

### 6.3 `step` (one physics slice)

```rust
fn step(&mut self, x: f32, y: f32, sdt: f32, config: &AppConfig) {
    let dt_scale = (sdt / REFERENCE_FRAME).clamp(0.01, 5.0);

    // 1. LazyBrush dead zone — per substep, dt-scaled factor for frame-rate independence.
    if config.trail.lazy_enabled {
        let dx = x - self.brush.0;
        let dy = y - self.brush.1;
        let dist = (dx * dx + dy * dy).sqrt();
        // TD: round((dist - radius)*10)/10 > 0  →  hysteresis at 0.05 px.
        let excess = dist - config.trail.lazy_radius;
        if dist > 1e-4 && (excess * 10.0).round() / 10.0 > 0.0 {
            let f = lazy_brush_factor(config.trail.lazy_friction);
            let factor = 1.0 - (1.0 - f).powf(dt_scale);   // frame-rate independent blend
            let pull = excess * factor;
            self.brush.0 += dx / dist * pull;
            self.brush.1 += dy / dist * pull;
        }
    } else {
        self.brush = (x, y);
    }

    let head_spring = config.trail.head_spring / 1000.0;
    let head_fric   = (1.0 - config.trail.head_damping / 100.0).clamp(0.01, 1.0).powf(dt_scale);
    let body_spring = config.trail.spring / 1000.0;
    let body_fric   = (1.0 - config.trail.damping    / 100.0).clamp(0.01, 1.0).powf(dt_scale);

    // 2. Head: 2nd-order spring-damper toward the brush (Windhawk ApplyTrailPhysicsSegment).
    {
        let cur = &mut self.nodes[0];
        cur.vx += (self.brush.0 - cur.x) * head_spring * dt_scale;
        cur.vy += (self.brush.1 - cur.y) * head_spring * dt_scale;
        cur.vx *= head_fric;  cur.vy *= head_fric;
        cur.x  += cur.vx * dt_scale;  cur.y += cur.vy * dt_scale;
        cur.finish_step();
    }

    // 3. Body: Windhawk chain (0.3 second-neighbour coupling), NO walls, NO lead nodes.
    for i in 1..self.nodes.len() {
        let prev = self.nodes[i - 1];
        let second = (i > 1).then(|| self.nodes[i - 2]);
        let cur = &mut self.nodes[i];
        if let Some(pp) = second {
            cur.vx += (pp.x - cur.x) * body_spring * 0.3 * dt_scale;
            cur.vy += (pp.y - cur.y) * body_spring * 0.3 * dt_scale;
        }
        cur.vx += (prev.x - cur.x) * body_spring * dt_scale;
        cur.vy += (prev.y - cur.y) * body_spring * dt_scale;
        cur.vx *= body_fric;  cur.vy *= body_fric;
        cur.x  += cur.vx * dt_scale;  cur.y += cur.vy * dt_scale;
        cur.finish_step();
    }

    // 4. Clamped distance constraint, head→tail (TD lines 431–476).
    for i in 1..self.nodes.len() {
        let prev = self.nodes[i - 1];
        let cur  = &mut self.nodes[i];
        let dx = cur.x - prev.x;
        let dy = cur.y - prev.y;
        let dist = (dx * dx + dy * dy).sqrt();
        if dist > MAX_NODE_GAP && dist > 1e-4 {
            let ratio = MAX_NODE_GAP / dist;
            let new_x = prev.x + dx * ratio;
            let new_y = prev.y + dy * ratio;
            // px displacement → px/reference-frame velocity, so sub-steps stay consistent.
            cur.vx += (new_x - cur.x) / dt_scale;
            cur.vy += (new_y - cur.y) / dt_scale;
            cur.x = new_x;  cur.y = new_y;
            cur.finish_step();
        }
    }
}
```

Keep existing constants: `REFERENCE_FRAME = 1/60`, `PHYSICS_TICK = 1/120`, `MAX_SUBSTEPS = 16`,
`MAX_FRAME_DELTA = 0.1`. Keep `update_mouse`'s `dt` clamp and sub-step entry unchanged.

### 6.4 Arc-length progress in `build_samples` (renderer.rs:586–686)

- During generation, push samples with `progress: 0.0` placeholder (keep `speed` lerp as-is).
- After the loop + final closing point, **post-pass**:

```rust
// Arc-length parameterisation: fade/blur/width taper must follow distance, not node index.
let mut total = 0.0f32;
for w in out.windows(2) { total += ((w[1].x - w[0].x).powi(2) + (w[1].y - w[0].y).powi(2)).sqrt(); }
if total < 1e-6 {
    let n = out.len();
    for (i, s) in out.iter_mut().enumerate() { s.progress = i as f32 / (n - 1).max(1) as f32; }
} else {
    let mut acc = 0.0f32;
    out[0].progress = 0.0;
    for i in 1..out.len() {
        let (a, b) = (out[i - 1], out[i]);
        acc += ((b.x - a.x).powi(2) + (b.y - a.y).powi(2)).sqrt();
        out[i].progress = (acc / total).min(1.0);
    }
    out.last_mut().unwrap().progress = 1.0;
}
```

Merge-near-duplicate step, phantom endpoints, adaptive steps, `catmull_rom_centripetal` — **unchanged**.

### 6.5 `chain_summary` (renderer.rs:1891–1916)

References `self.chain.cursor_dir` and `self.chain.head_target` — both deleted. Rewrite to report
`brush` offset instead, e.g. `brush=({:+.1},{:+.1})` relative to `last_mouse_pos`. Keep
`nodes=`, `first5=`, `farthest=`, `moving=` fields (snapshot scripts log this line).

---

## 7. WGSL tiebreaker — `crates/fxcursor-render/src/shaders/render.wgsl:119–122`

Current (tail wins ties — wrong):

```wgsl
let depth = (layer + alpha * 0.99 + 0.001 + seg * 1e-6) / LAYER_BANDS;
```

Replace with a **saturating head-favoring bias** (head = low `seg` index; instances are pushed
head-first, so draw order already favors head within epsilon — the bias must extend that to the
pre-pass for larger `seg` spreads without driving depth negative):

```wgsl
// Head (low seg) wins coverage ties: bias toward low seg, saturating so depth never
// leaves the layer band (max seg ~4800 → bias ≤ ~4.8e-4 with 1e-7 scale… see note).
let seg_bias = 1e-3 / (1.0 + seg);            // seg=0 → 1e-3, decays toward 0
let depth = (layer + 0.001 + seg_bias + alpha * 0.99) / LAYER_BANDS;
```

Notes:
- Min depth at `alpha = 0.002` (MIN_VISIBLE_ALPHA): `0.001 + ~0 + 0.00198 > 0` — never clamps to 0.
- Bias (`≤1e-3`) sits inside the `0.001`-scale band floor, well below the `0.99` alpha range, and
  adjacent-seg differences for small `seg` exceed the `1e-4` color-pass epsilon so the head wins
  the pre-pass; for large `seg` bias→0 and the existing epsilon + head-first draw order resolves it.
- Keep `fs_capsule_color`'s `+1e-4` bias and the stencil paint-once logic **unchanged**.
- Update the comment on line 119 (“tiny per-segment tie breaker”) to say **head** wins ties.

---

## 8. Tests — rewrite the physics block (renderer.rs:2122–2255)

**DELETE** these tests (they test removed machinery):
- `block_overtake_stops_a_node_at_the_wall_but_leaves_a_hairpin_alone` (2133)
- `lead_nodes_never_overshoot_their_predecessor` (2191)
- `chain_never_shoots_through_a_stopped_pointer` (2167) — spring-damper **can** overshoot; replace (below)
- `reversal_keeps_passed_nodes_behind_the_pointer` (2212) — relied on no-overtake walls

**KEEP**: `chain_grows_from_its_tail` (2155), `chain_is_frame_rate_independent` (2238, tolerance
12 px between 60/240 fps — should still pass; if the new physics drifts more, investigate before
loosening), and all non-physics geometry tests (spline, capsules, fade, mode mask).

**ADD** (names suggested):

1. `chain_settles_on_a_stopped_pointer_with_bounded_overshoot`
   - Drive right for 90 frames (`x += 15/frame`), then hold `x` for 300 frames.
   - Assert during settle: no node more than `~80 px` in front of the pointer (empirically tune;
     spring-damper overshoot should be small with defaults k=0.05, f=0.7).
   - After 300 frames: every node within 1 px of the pointer, `!c.is_moving()`, `y` stays ~0.
2. `lazy_brush_dead_zone_holds_then_drags`
   - `cfg.trail.lazy_enabled = true; lazy_radius = 30; lazy_friction = 0.4`.
   - Move pointer 10 px/frame for 30 frames: brush displacement per frame ≈ 0 while
     `dist(pointer, brush) ≤ radius` (dead zone), then brush starts following, always keeping
     roughly `radius` behind once in steady state (`dist` ≈ radius, not 0).
   - With `lazy_enabled = false`: brush == pointer every frame.
3. `distance_constraint_bounds_node_gaps`
   - Teleport the pointer 2000 px in one `advance`, run a few frames:
     every consecutive pair satisfies `dist ≤ MAX_NODE_GAP + 1e-3`.
4. `sample_progress_is_arc_length_not_index`
   - Nodes: `(0,0), (10,0), (10,0.001…skip merge)` — better: uneven spacing like
     `(0,0,s), (5,0,s), (100,0,s), (101,0,s)` with adaptive off (`interpolation_steps = 1`).
   - Assert the sample near x=50 (half the 101 px length) has `progress ≈ 0.5 ± 0.05`, whereas
     index-based progress would put it near `1/3`. Also keep monotonic + endpoints 0/1.

Also update the existing straight-line test (2012) only if it asserts index-based progress — it
asserts monotonicity + endpoints, which arc-length still satisfies.

---

## 9. LivePreview mirror — `src/components/Preview/LivePreview.tsx`

Must stay behaviorally identical to Rust (repo rule).

- **Delete**: `Wall` interface, `blockOvertake`, `followStep`, `cursorDir`, `headTarget` EMA,
  `leadNodes`, `lastMouse` (only used for `cursorDir`), `Node.dx/dy` **if** nothing else uses them
  (they encode motion direction for walls — check `mirror()` helper and `chain` construction; the
  spline path does not need `dx/dy`; remove from `Node` and the synthetic cursor/mirror nodes).
- **Add**: `const brush = { x: 0, y: 0 }; let brushValid = false;`
- **Physics block** (replacing lines ~335–384): mirror §6.3 **without sub-stepping** (the preview
  already single-steps per rAF with `dtScale` — keep that approximation, it matches the existing
  preview design). Sequence per frame:
  1. LazyBrush update (dt-scaled factor as in §6.3 step 1).
  2. Head spring-damper toward brush (`head_spring/1000`, `head_damping` friction).
  3. Body chain with `0.3` second-neighbour coupling.
  4. Distance constraint with `MAX_NODE_GAP = 64` and `vx += (new-old)/dtScale`.
  5. `finishStep` = speed only (drop `dx/dy` updates if fields removed).
- **Sample progress**: the preview builds samples inline (lines ~451–486) with
  `progress: (s + t) / segCount` — apply the same arc-length post-pass over the finished
  `samples` array before `drawLayer` runs (compute chord lengths, assign, endpoints 0/1).
- Also remove the **normal** computation (`nx/ny`, `prevNx/prevNy`, `Sample.nx/ny`) **only if**
  unused — they are legacy from the quad-strip era; capsules don't need normals. Verify no other
  reference first; if unused, delete to reduce drift surface. (Optional cleanup; not required.)

---

## 10. UI — `src/components/Tabs/TrailTab.tsx`

- Local interface: drop `lead_nodes?: number`; add `lazy_enabled?: boolean; lazy_radius?: number;
  lazy_friction?: number;`.
- **Remove** the "Lead Nodes" slider (lines 66–74).
- **Head Point Kinematics card**: keep "Head Follow Strength" (`head_spring`) and "Head Smoothing"
  — **relabel the latter**: it is now velocity friction, not pointer EMA. Suggested:
  - `head_spring`: label "Head Spring Strength", sub "Spring pulling the ribbon head toward the
    pointer / brush (10 – 300, higher = tighter)", min 10, max 300, step 5 (was 5–98 first-order
    scale; ÷1000 scale means presets use 45–80, keep them valid).
  - `head_damping`: label "Head Friction", sub "Velocity damping on the head node (0 = none,
    99 = heavy)", min 0, max 99, step 1.
- **Add a new SectionCard "Lazy Brush"** (after Head Kinematics):
  - `Toggle` bound to `lazy_enabled ?? false` (field is optional in TS because of specta).
  - Slider `lazy_radius`: min 5, max 150, step 1, unit px, value `props.trail.lazy_radius ?? 30`.
  - Slider `lazy_friction`: min 0, max 0.99, step 0.01, value `props.trail.lazy_friction ?? 0.4`.
  - Show radius/friction sliders always (or gate on enabled — either fine; TD gates via dependsOn).
- Section descs that say "D3D11 multi-segment…" can be updated to mention Windhawk spring-damper
  + clamped distance constraint.

---

## 11. Docs to update after code is green

| File | Line(s) | New wording (substance) |
| :-- | :-- | :-- |
| `README.md` | 60 | Trail = LazyBrush (optional dead-zone) → Windhawk spring-damper head + body (0.3 second-neighbour coupling) + clamped 64 px distance constraint; arc-length progress; fixed 1/120 s sub-steps. No `lead_nodes`, no no-overtake rule. |
| `PROGRESS.md` | 25 (table Physics row) | Same substitution. |
| `PROGRESS.md` | 120–122 (session 6 bullet + tests) | Mark superseded; add a **session 9** subsection under §2 describing this rewrite (LazyBrush, spring-damper, constraint, arc-length, head-wins-tiebreak, new tests). |
| `PROGRESS.md` | Verification matrix (§3) | Update test counts (was 52→60 cargo in session 8; re-count after new tests) and date. |
| `CHANGELOG.md` | 69 (session 6 entry) | Leave history intact; **add a new entry** for this session describing the replacement (don't rewrite history). |
| `repomix-instruction.md` | 8 | "pursuit followers and no-overtake rule" → "LazyBrush + spring-damper chain + distance constraint". |
| `ARCHITECTURE.md` | generated | Run `bun run arch`. |

---

## 12. Execution order (do it in this order to keep gates green as long as possible)

1. `config.rs` — remove `lead_nodes`, add 3 lazy fields + defaults + doc comments.
2. `presets.rs` — 5 literals updated (remove `lead_nodes`, add 3 fields).
3. `presets.ts` — mirror defaults.
4. `bindings.ts` — hand-edit TrailConfig (optional lazy fields, drop `lead_nodes`).
5. `renderer.rs` — TrailChain rewrite (§6), `chain_summary`, arc-length `build_samples`,
   test rewrite (§8).
6. `render.wgsl` — tiebreaker (§7).
7. `LivePreview.tsx` — mirror (§9).
8. `TrailTab.tsx` — UI (§10).
9. `bun run fixtures` → regenerates `default_config.json` (+ mode_masks).
10. Gates, in order, fixing as you go:
    ```powershell
    rtk cargo check --workspace
    rtk cargo test --workspace
    rtk cargo clippy --workspace --all-targets -- -D warnings
    bun run typecheck
    bun run lint
    bun test
    bun run build
    bun run validate        # all 7, includes bun run arch
    ```
11. Docs (§11). Then optionally `bun run tauri dev` smoke test — this also regenerates
    `bindings.ts` authoritatively via specta (confirm lazy fields land as optional).
12. **Do NOT commit** unless the user asks.

### Sanity expectations after the rewrite

- Head lag drops massively (single spring instead of 6 filters) — if the head feels *too* snappy or
  oscillates at rest, tune defaults (`head_spring` ↑ stiffens / reduces lag; `head_damping` ↑
  damps oscillation). Windhawk shipped `50/30` happily with this exact formula.
- Jitter from stick-slip walls disappears entirely.
- Width/fade taper becomes uniform along the ribbon (arc-length).
- Folded trails keep the **head** on top (tiebreak fix).
- `lazy_enabled=false` by default → existing user configs behave like a plain Windhawk chain
  (self-healing fills the new fields via `#[serde(default)]`).

---

## 13. Quick reference — files you will touch

```
crates/fxcursor-protocol/src/config.rs          lead_nodes → lazy_* fields
crates/fxcursor-protocol/src/presets.rs         5 preset literals
crates/fxcursor-render/src/renderer.rs          TrailChain rewrite, build_samples, tests, chain_summary
crates/fxcursor-render/src/shaders/render.wgsl  depth tiebreaker (head wins)
src/lib/presets.ts                              TS defaults mirror
src/lib/bindings.ts                             hand-edit; regenerated by tauri dev
src/components/Tabs/TrailTab.tsx                UI: remove lead nodes, add Lazy Brush card
src/components/Preview/LivePreview.tsx          physics + progress mirror
test/fixtures/default_config.json               via `bun run fixtures`
README.md, PROGRESS.md, CHANGELOG.md, repomix-instruction.md   wording
ARCHITECTURE.md                                 via `bun run arch`
```

Primary references while coding:
- `legacy/original_mods/D3D_cursor_mod.wh.cpp:1751–1808` (spring-damper)
- `legacy/TD_Web_Trail/trail-system.js:18–133` (LazyBrush), `:431–476` (distance constraint)
- `legacy/project_cursor/src-tauri/src/overlay/renderer.rs:507–555` (V3 Rust port of the above)
