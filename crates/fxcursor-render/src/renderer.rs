use crate::cursor::{cursor_quad_vertices, CursorShape, CursorVertex, GpuCursorState};
use fxcursor_protocol::{AppConfig, EffectMode, LayerConfig};
use std::time::Instant;
use wgpu::{BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline, Sampler, TextureFormat};

/// Which subsystems an [`EffectMode`] allows. Individual `enabled` flags in the config still
/// apply on top of this mask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModeMask {
    pub trail: bool,
    /// Per-layer gate for the 4-layer ribbon (Outer Glow, Mid Shadow, Crisp Core, Inner Spine).
    pub layers: [bool; 4],
    pub head: bool,
    pub ripples: bool,
    pub particles: bool,
    pub satellites: bool,
}

impl ModeMask {
    pub const ALL: ModeMask = ModeMask {
        trail: true,
        layers: [true; 4],
        head: true,
        ripples: true,
        particles: true,
        satellites: true,
    };

    pub fn from_mode(mode: EffectMode) -> Self {
        match mode {
            EffectMode::FourLayerGlow => Self::ALL,
            // Trail and head only: no click effects, no orbitals.
            EffectMode::Ribbon => ModeMask {
                ripples: false,
                particles: false,
                satellites: false,
                ..Self::ALL
            },
            // Click feedback only: head, ripples and particle bursts.
            EffectMode::ParticlesOnly => ModeMask {
                trail: false,
                satellites: false,
                ..Self::ALL
            },
            // Orbitals only: head and satellites.
            EffectMode::SatellitesOnly => ModeMask {
                trail: false,
                ripples: false,
                particles: false,
                ..Self::ALL
            },
            // Minimal: crisp core + inner spine with the head; no glow/shadow, no extras.
            EffectMode::Minimal => ModeMask {
                layers: [false, false, true, true],
                ripples: false,
                particles: false,
                satellites: false,
                ..Self::ALL
            },
        }
    }
}

/// One round capsule of a ribbon layer, rendered as an instanced quad (see `render.wgsl`).
/// Colours are straight (non-premultiplied); the fragment shader premultiplies after feathering.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CapsuleInstance {
    pub a: [f32; 2],
    pub b: [f32; 2],
    /// Radius at `a`, radius at `b` (tapered capsule).
    pub radii: [f32; 2],
    pub color_a: [f32; 4],
    pub color_b: [f32; 4],
    /// `[blur at a, layer index, segment index, blur at b]` (blur = feathered radius fraction).
    pub params: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: [f32; 2],
    pub angle: f32,
    pub thickness: f32,
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayUniforms {
    screen_size: [f32; 2],
    virtual_origin: [f32; 2],
    time: f32,
    padding: f32,
}

#[derive(Clone, Copy, Debug, Default)]
struct TrailNode {
    x: f32,
    y: f32,
    /// Velocity in pixels per reference frame (1/120 s — Windhawk `kReferenceFrameTime`).
    vx: f32,
    vy: f32,
    speed: f32,
}

impl TrailNode {
    fn finish_step(&mut self) {
        self.speed = (self.vx * self.vx + self.vy * self.vy).sqrt();
    }
}

/// Below this speed (px per reference frame) a node counts as at rest.
const MIN_DIRECTION_SPEED: f32 = 0.05;
/// The head counts as arrived when it is this close (px) to the brush (Windhawk: 0.1 px).
const HEAD_REST_DISTANCE: f32 = 0.1;
/// Largest gap (px) between consecutive nodes; beyond this the follower is clamped back
/// (TD-style clamped distance constraint). **Teleport guard only.**
///
/// Must sit well above the spring chain's natural steady-state gap, which at the Windhawk
/// defaults (k=0.05, f=0.7) is `gap ≈ V×(1−f)/(k×f) ≈ 8.6×` px per reference frame — easily
/// 150–400 px during a hard flick. Windhawk ships **no** constraint; a tight clamp (the old
/// 64 px) fights the springs every frame of normal motion and injects velocity that reads as
/// the trail accelerating on sudden movements.
const MAX_NODE_GAP: f32 = 512.0;

/// TD LazyBrush friction factor: how much of the excess distance the brush covers.
/// f <= 0 → 1.0 (snap); f >= 1 → 0.0 (frozen); else 1 - sqrt(1 - (1-f)^2).
fn lazy_brush_factor(friction: f32) -> f32 {
    if friction <= 0.0 {
        return 1.0;
    }
    if friction >= 1.0 {
        return 0.0;
    }
    let u = 1.0 - friction;
    1.0 - (1.0 - u * u).sqrt()
}

// ---- On-overlay FPS HUD: 3×5 dot-matrix font drawn with the SDF circle pipeline -------------

/// Dot pitch of the HUD font in overlay pixels.
const HUD_CELL: f32 = 5.0;
/// Dot radius; slightly over half the pitch so neighbouring dots fuse into strokes.
const HUD_DOT_RADIUS: f32 = 2.7;
/// Distance from the anchored screen corner.
const HUD_MARGIN: f32 = 16.0;

/// 3-column × 5-row glyphs, one row per byte, most significant of the low three bits = left.
const HUD_GLYPHS: &[(char, [u8; 5])] = &[
    ('0', [0b111, 0b101, 0b101, 0b101, 0b111]),
    ('1', [0b010, 0b110, 0b010, 0b010, 0b111]),
    ('2', [0b111, 0b001, 0b111, 0b100, 0b111]),
    ('3', [0b111, 0b001, 0b111, 0b001, 0b111]),
    ('4', [0b101, 0b101, 0b111, 0b001, 0b001]),
    ('5', [0b111, 0b100, 0b111, 0b001, 0b111]),
    ('6', [0b111, 0b100, 0b111, 0b101, 0b111]),
    ('7', [0b111, 0b001, 0b010, 0b010, 0b010]),
    ('8', [0b111, 0b101, 0b111, 0b101, 0b111]),
    ('9', [0b111, 0b101, 0b111, 0b001, 0b111]),
    ('F', [0b111, 0b100, 0b110, 0b100, 0b100]),
    ('P', [0b111, 0b101, 0b111, 0b100, 0b100]),
    ('S', [0b111, 0b100, 0b111, 0b001, 0b111]),
    ('-', [0b000, 0b000, 0b111, 0b000, 0b000]),
    (' ', [0b000; 5]),
];

fn hud_glyph(c: char) -> [u8; 5] {
    HUD_GLYPHS
        .iter()
        .find(|(g, _)| *g == c)
        .map(|(_, rows)| *rows)
        .unwrap_or([0b000; 5])
}

/// Width in pixels of `text` set in the HUD font (3 dots + 1 gap per glyph).
fn hud_text_width(text: &str) -> f32 {
    (text.chars().count() as f32 * 4.0 - 1.0).max(0.0) * HUD_CELL
}

/// Emits `text` as dots into `out`, top-left corner at `(x, y)` in world pixels: a black shadow
/// pass first, then the white glyphs, so the HUD reads on any background.
fn push_hud_text(out: &mut Vec<CircleInstance>, text: &str, x: f32, y: f32) {
    for (shadow, color, radius) in [
        (true, [0.0, 0.0, 0.0, 0.85], HUD_DOT_RADIUS + 1.2),
        (false, [0.95, 0.95, 0.95, 1.0], HUD_DOT_RADIUS),
    ] {
        let offset = if shadow { 1.0 } else { 0.0 };
        for (i, c) in text.chars().enumerate() {
            let rows = hud_glyph(c);
            for (row, bits) in rows.iter().enumerate() {
                for col in 0..3u32 {
                    if bits & (0b100 >> col) == 0 {
                        continue;
                    }
                    out.push(CircleInstance {
                        center: [
                            x + (i as f32 * 4.0 + col as f32) * HUD_CELL + offset,
                            y + row as f32 * HUD_CELL + offset,
                        ],
                        radius: [radius, radius],
                        angle: 0.0,
                        thickness: -1.0,
                        color,
                    });
                }
            }
        }
    }
}

/// The chain of nodes behind the pointer. GPU-free so the physics is unit-testable, and public
/// so `examples/dump_trail_trace.rs` can record the reference trace that the TypeScript mirror
/// (`src/lib/trail.ts`, used by the Studio live preview) is tested against.
///
/// * The raw pointer first passes through an optional **LazyBrush** dead-zone filter
///   (`lazy_enabled`/`lazy_radius`/`lazy_friction`, TD-style) producing the brush target.
/// * Node 0 (head) is a 2nd-order spring-damper toward that brush (Windhawk
///   `ApplyTrailPhysicsSegment`: `v += gap×k×dt_scale; v *= fric; p += v×dt_scale`).
/// * The remaining nodes form a spring-damper chain (forward Gauss-Seidel pass with a 0.3×
///   second-neighbour coupling that keeps sharp turns from kinking). Springs scale linearly
///   with the time step and friction exponentially, so the feel is frame-rate independent.
/// * After each step a clamped distance constraint (TD) pulls any gap larger than
///   [`MAX_NODE_GAP`] back in — a **teleport guard only** (the limit sits far above the
///   chain's natural steady-state gap so normal motion never touches it), applied
///   inelastically so it cannot inject speed into a flick.
#[derive(Debug, Default)]
pub struct TrailChain {
    nodes: Vec<TrailNode>,
    /// LazyBrush position (equals the pointer when `lazy_enabled` is off).
    brush: (f32, f32),
    brush_valid: bool,
    /// The LazyBrush moved during the last step (still catching up with the pointer).
    brush_moving: bool,
}

impl TrailChain {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
            ..Default::default()
        }
    }

    /// Node positions and speeds, head first: `(x, y, speed px per 1/120 s)`.
    pub fn nodes(&self) -> impl ExactSizeIterator<Item = (f32, f32, f32)> + '_ {
        self.nodes.iter().map(|n| (n.x, n.y, n.speed))
    }

    /// Current LazyBrush position (the pointer itself when the brush is off).
    pub fn brush(&self) -> (f32, f32) {
        self.brush
    }

    /// Grows the chain from its tail (new nodes start on the last node, or on the pointer when
    /// the chain is empty) or truncates it.
    pub fn resize(&mut self, len: usize, x: f32, y: f32) {
        self.nodes.truncate(len);
        while self.nodes.len() < len {
            let seed = self.nodes.last().copied().unwrap_or(TrailNode {
                x,
                y,
                ..Default::default()
            });
            self.nodes.push(TrailNode {
                vx: 0.0,
                vy: 0.0,
                speed: 0.0,
                ..seed
            });
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.brush_valid = false;
        self.brush_moving = false;
    }

    /// Still in motion: a node is moving, the head has not reached the brush yet (a 1 px nudge
    /// from rest gives the head a speed below the threshold, so speed alone parked the loop with
    /// the head off the cursor), or the LazyBrush is still being dragged.
    pub fn is_moving(&self) -> bool {
        let head_away = self.nodes.first().is_some_and(|h| {
            (h.x - self.brush.0).abs() > HEAD_REST_DISTANCE
                || (h.y - self.brush.1).abs() > HEAD_REST_DISTANCE
        });
        self.brush_moving || head_away || self.nodes.iter().any(|n| n.speed > MIN_DIRECTION_SPEED)
    }

    /// Advances the chain by one frame of `dt` seconds toward the pointer at `(x, y)`.
    ///
    /// Sub-stepping: explicit Euler springs scaled by a large frame delta diverge at low frame
    /// rates, so the frame is split into `ceil(dt / PHYSICS_TICK)` equal slices of at most
    /// [`PHYSICS_TICK`] (equal slices, not a fixed-rate accumulator: with vsync pacing `dt` is
    /// steady, and there is no render-side interpolation to feed).
    pub fn advance(&mut self, x: f32, y: f32, dt: f32, config: &AppConfig) {
        if self.nodes.is_empty() {
            return;
        }
        if !self.brush_valid {
            self.brush = (x, y);
            self.brush_valid = true;
        }
        if !config.trail.lazy_enabled {
            self.brush = (x, y);
        }
        // The small epsilon keeps an exact multiple of the tick from rounding up to one extra
        // slice (in f32, (1/60)/(1/120) can come out as 2.0000002).
        let substeps = ((dt / PHYSICS_TICK - 1e-3).ceil() as usize).clamp(1, MAX_SUBSTEPS);
        let sdt = dt / substeps as f32;
        self.brush_moving = false;
        for _ in 0..substeps {
            self.step(x, y, sdt, config);
        }
        // A non-finite value (imported config, extreme spring) would otherwise poison the chain
        // for good: NaN speeds read as "at rest", so the loop parks on garbage. Start over.
        if !self.nodes.iter().all(|n| n.x.is_finite() && n.y.is_finite() && n.speed.is_finite()) {
            let len = self.nodes.len();
            self.reset();
            self.resize(len, x, y);
        }
    }

    /// One physics slice (`sdt` seconds, at most [`PHYSICS_TICK`]).
    fn step(&mut self, x: f32, y: f32, sdt: f32, config: &AppConfig) {
        let dt_scale = (sdt / REFERENCE_FRAME).clamp(0.01, 5.0);

        // 1. LazyBrush dead zone — dt-scaled factor keeps the pull frame-rate independent.
        if config.trail.lazy_enabled {
            let dx = x - self.brush.0;
            let dy = y - self.brush.1;
            let dist = (dx * dx + dy * dy).sqrt();
            // TD: round((dist - radius)*10)/10 > 0 → 0.1 px quantisation avoids float jitter.
            let excess = dist - config.trail.lazy_radius;
            if dist > 1e-4 && (excess * 10.0).round() / 10.0 > 0.0 {
                let f = lazy_brush_factor(config.trail.lazy_friction);
                let factor = 1.0 - (1.0 - f).powf(dt_scale);
                let pull = excess * factor;
                self.brush.0 += dx / dist * pull;
                self.brush.1 += dy / dist * pull;
                self.brush_moving = true;
            }
        } else {
            self.brush = (x, y);
        }

        let head_spring = config.trail.head_spring / 1000.0;
        let head_fric = (1.0 - config.trail.head_damping / 100.0)
            .clamp(0.01, 1.0)
            .powf(dt_scale);
        let body_spring = config.trail.spring / 1000.0;
        let body_fric =
            (1.0 - config.trail.damping / 100.0).clamp(0.01, 1.0).powf(dt_scale);

        // 2. Head: 2nd-order spring-damper toward the brush (Windhawk ApplyTrailPhysicsSegment).
        {
            let brush = self.brush;
            let cur = &mut self.nodes[0];
            cur.vx += (brush.0 - cur.x) * head_spring * dt_scale;
            cur.vy += (brush.1 - cur.y) * head_spring * dt_scale;
            cur.vx *= head_fric;
            cur.vy *= head_fric;
            cur.x += cur.vx * dt_scale;
            cur.y += cur.vy * dt_scale;
            cur.finish_step();
        }

        // 3. Body: Windhawk chain (0.3 second-neighbour coupling).
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
            cur.vx *= body_fric;
            cur.vy *= body_fric;
            cur.x += cur.vx * dt_scale;
            cur.y += cur.vy * dt_scale;
            cur.finish_step();
        }

        // 4. Clamped distance constraint, head→tail (TD trail-system.js, clamped mode).
        // Position is pulled back only, and only separating velocity is stripped — the clamp
        // is inelastic. TD adds the full correction into `dx` at a fixed 60 Hz; re-applying
        // that on the next sub-step (and every frame the gap stays above the limit during a
        // flick) slingshots the follower forward, which reads as the trail accelerating.
        for i in 1..self.nodes.len() {
            let prev = self.nodes[i - 1];
            let cur = &mut self.nodes[i];
            let dx = cur.x - prev.x;
            let dy = cur.y - prev.y;
            let dist = (dx * dx + dy * dy).sqrt();
            // MAX_NODE_GAP ≫ 0, so this also guarantees dist is safe to divide by.
            if dist > MAX_NODE_GAP {
                let ratio = MAX_NODE_GAP / dist;
                let nx = dx / dist;
                let ny = dy / dist;
                let vn = cur.vx * nx + cur.vy * ny;
                if vn > 0.0 {
                    cur.vx -= vn * nx;
                    cur.vy -= vn * ny;
                }
                cur.x = prev.x + dx * ratio;
                cur.y = prev.y + dy * ratio;
                cur.finish_step();
            }
        }
    }
}

/// A point on the resampled centerline. Capsules do not need normals, so hairpins and
/// cusps need no special handling: the union of round capsules is always well formed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sample {
    pub x: f32,
    pub y: f32,
    pub speed: f32,
    /// 0 at the head, 1 at the tail.
    pub progress: f32,
}

struct Ripple {
    x: f32,
    y: f32,
    time_elapsed: f32,
    color: [f32; 4],
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32,
    max_life: f32,
    size: f32,
    color: [f32; 4],
}

/// Velocity-squished head blob (Windhawk `UpdateSquishyCursor`).
#[derive(Debug, Default)]
struct SquishyState {
    pos_x: f32,
    pos_y: f32,
    prev_x: f32,
    prev_y: f32,
    current_scale: f32,
    current_angle: f32,
    target_scale: f32,
    target_angle: f32,
    /// False until the first step (or after a reset): the head then starts on the pointer
    /// instead of flying in from the origin.
    initialized: bool,
}

impl SquishyState {
    /// One frame of the Windhawk squishy head.
    ///
    /// The blob eases toward the pointer with `1 - (1 - s)^dt_scale` (exponential, so the
    /// feel is frame-rate independent), and its squish comes from the eased head's speed in
    /// **px per reference frame**. Measuring in px/s (as V4 did before) made
    /// `min(v × 8, 200)` saturate at ~25 px/s, so the head sat fully squashed on any motion.
    fn step(&mut self, x: f32, y: f32, dt_scale: f32, config: &AppConfig) {
        if !self.initialized {
            *self = SquishyState {
                pos_x: x,
                pos_y: y,
                prev_x: x,
                prev_y: y,
                initialized: true,
                ..Default::default()
            };
        }
        let smoothing = (config.head.squish_smoothing / 100.0).clamp(0.01, 1.0);
        let adaptive = 1.0 - (1.0 - smoothing).powf(dt_scale);

        self.pos_x += (x - self.pos_x) * adaptive;
        self.pos_y += (y - self.pos_y) * adaptive;
        let dx = self.pos_x - self.prev_x;
        let dy = self.pos_y - self.prev_y;
        let velocity = (dx * dx + dy * dy).sqrt() / dt_scale;
        self.prev_x = self.pos_x;
        self.prev_y = self.pos_y;

        let intensity = config.head.squish_intensity / 100.0;
        let amplified = (velocity * 8.0).min(200.0);
        self.target_scale = (amplified / 15.0) * intensity;
        self.current_scale += (self.target_scale - self.current_scale) * adaptive;

        if velocity > 0.5 {
            self.target_angle = dy.atan2(dx);
        }
        let angle_diff = (self.target_angle - self.current_angle + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        self.current_angle += angle_diff * adaptive;
    }

    /// Still easing toward the pointer or changing shape.
    fn is_animating(&self, pointer: (f32, f32)) -> bool {
        (self.current_scale - self.target_scale).abs() > 0.001
            || (self.pos_x - pointer.0).abs() > 0.05
            || (self.pos_y - pointer.1).abs() > 0.05
    }
}

struct SatelliteState {
    angle: f32,
    mirror_angle: f32,
}

/// Depth attachment used to resolve each layer's capsule union (see `render.wgsl`).
struct DepthTarget {
    width: u32,
    height: u32,
    view: wgpu::TextureView,
}

/// Depth resolves each layer's capsule union (max coverage); the stencil guarantees that each
/// pixel is painted at most once per layer even when several capsules tie within the depth
/// tolerance (layer `k` paints where stencil <= k and then writes k + 1).
pub const DEPTH_FORMAT: TextureFormat = TextureFormat::Depth24PlusStencil8;

fn next_random(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed as f32) / (u32::MAX as f32)
}

fn hsl_to_rgba(hue: f32, sat: f32, lit: f32) -> [f32; 4] {
    let h = hue.rem_euclid(360.0);
    let c = (1.0 - (2.0 * lit - 1.0).abs()) * sat;
    let x = c * (1.0 - ((h / 60.0).rem_euclid(2.0) - 1.0).abs());
    let m = lit - c * 0.5;
    let (r1, g1, b1) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };
    [r1 + m, g1 + m, b1 + m, 1.0]
}

fn lerp_rgba(c0: [f32; 4], c1: [f32; 4], t: f32) -> [f32; 4] {
    let t = t.clamp(0.0, 1.0);
    [
        c0[0] + (c1[0] - c0[0]) * t,
        c0[1] + (c1[1] - c0[1]) * t,
        c0[2] + (c1[2] - c0[2]) * t,
        c0[3] + (c1[3] - c0[3]) * t,
    ]
}

/// Head-to-tail falloff: 0 Linear, 1 Ease Out, 2 Exponential, 3 Sigmoid (Windhawk's four),
/// 4 Smoothstep (flat at both ends). Unknown modes fall back to linear.
fn apply_fade_curve(progress: f32, mode: u32) -> f32 {
    let p = progress.clamp(0.0, 1.0);
    match mode {
        1 => 1.0 - p * p,
        2 => (-p * 3.0).exp(),
        3 => 1.0 / (1.0 + (8.0 * (p - 0.5)).exp()),
        4 => 1.0 - p * p * (3.0 - 2.0 * p),
        _ => 1.0 - p,
    }
}

/// Uniform Catmull-Rom (kept for the tests and as a reference; the ribbon uses the centripetal
/// form below, which never overshoots when control points are unevenly spaced).
#[cfg(test)]
fn catmull_rom(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let t2 = t * t;
    let t3 = t2 * t;
    let x = 0.5
        * ((2.0 * p1.0)
            + (-p0.0 + p2.0) * t
            + (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2
            + (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
    let y = 0.5
        * ((2.0 * p1.1)
            + (-p0.1 + p2.1) * t
            + (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2
            + (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
    (x, y)
}

/// Centripetal Catmull-Rom (α = 0.5, Barry–Goldman form) evaluated between `p1` and `p2`.
///
/// Spring nodes bunch up on reversals and can be very unevenly spaced after LazyBrush lag or a
/// hard stop; the uniform spline then loops and hooks (the classic "curl at the head"). The
/// centripetal parametrisation has no cusps or self-intersections within a segment for any
/// spacing.
fn catmull_rom_centripetal(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    #[inline]
    fn knot(a: (f32, f32), b: (f32, f32)) -> f32 {
        let d2 = (b.0 - a.0) * (b.0 - a.0) + (b.1 - a.1) * (b.1 - a.1);
        // |Δ|^0.5 with a floor so coincident points cannot collapse a knot interval.
        d2.max(1e-6).sqrt().sqrt()
    }
    #[inline]
    fn lerp(a: (f32, f32), b: (f32, f32), wa: f32, wb: f32) -> (f32, f32) {
        (a.0 * wa + b.0 * wb, a.1 * wa + b.1 * wb)
    }

    let t0 = 0.0;
    let t1 = t0 + knot(p0, p1);
    let t2 = t1 + knot(p1, p2);
    let t3 = t2 + knot(p2, p3);
    let tt = t1 + (t2 - t1) * t.clamp(0.0, 1.0);

    let a1 = lerp(p0, p1, (t1 - tt) / (t1 - t0), (tt - t0) / (t1 - t0));
    let a2 = lerp(p1, p2, (t2 - tt) / (t2 - t1), (tt - t1) / (t2 - t1));
    let a3 = lerp(p2, p3, (t3 - tt) / (t3 - t2), (tt - t2) / (t3 - t2));
    let b1 = lerp(a1, a2, (t2 - tt) / (t2 - t0), (tt - t0) / (t2 - t0));
    let b2 = lerp(a2, a3, (t3 - tt) / (t3 - t1), (tt - t1) / (t3 - t1));
    lerp(b1, b2, (t2 - tt) / (t2 - t1), (tt - t1) / (t2 - t1))
}

/// Hard caps matching the GPU buffer sizes allocated in `OverlayRenderer::new`.
/// 150 nodes × up to 24 adaptive steps × 4 layers ≈ 14 k capsules at the extreme.
const MAX_CAPSULES: usize = 16384;
const MAX_CIRCLE_INSTANCES: usize = 2048;
/// Leave headroom in the instance buffer for head, ripples, satellites.
const MAX_PARTICLES: usize = MAX_CIRCLE_INSTANCES - 256;
/// Nodes closer than this (in overlay pixels) are merged before splining so the centerline
/// never contains zero-length segments.
const MIN_NODE_SPACING: f32 = 0.25;
/// Sample spacing bounds for the curvature-adaptive subdivision (overlay pixels).
const MIN_SAMPLE_SPACING: f32 = 3.0;
const MAX_SAMPLE_SPACING: f32 = 24.0;
/// Capsules whose both ends are fainter than this are skipped entirely.
const MIN_VISIBLE_ALPHA: f32 = 0.004;
/// Physics reference frame. Windhawk (`kReferenceFrameTime`) and V3 both use **1/120 s**:
/// `dt_scale = dt / REFERENCE_FRAME`, spring forces scale by `dt_scale`, friction as
/// `fric.powf(dt_scale)`. Using 1/60 here halved every spring impulse per second and softened
/// friction — the trail then lagged far behind legacy and felt slow / uneven.
const REFERENCE_FRAME: f32 = 1.0 / 120.0;
/// Largest integration slice for the spring chain; longer frames are sub-stepped.
const PHYSICS_TICK: f32 = 1.0 / 120.0;
const MAX_SUBSTEPS: usize = 16;
/// Longest frame delta (s) the physics integrates; `MAX_SUBSTEPS` slices keep it stable.
const MAX_FRAME_DELTA: f32 = 0.1;

/// Absolute change of direction (radians, 0..π) at `b` for the polyline a → b → c.
fn turning_angle(a: (f32, f32), b: (f32, f32), c: (f32, f32)) -> f32 {
    let (ux, uy) = (b.0 - a.0, b.1 - a.1);
    let (vx, vy) = (c.0 - b.0, c.1 - b.1);
    let lu = (ux * ux + uy * uy).sqrt();
    let lv = (vx * vx + vy * vy).sqrt();
    if lu < 1e-4 || lv < 1e-4 {
        return 0.0;
    }
    let cos = ((ux * vx + uy * vy) / (lu * lv)).clamp(-1.0, 1.0);
    cos.acos()
}

const QUAD_VERTICES: [[f32; 2]; 6] = [
    [-1.0, -1.0],
    [1.0, -1.0],
    [-1.0, 1.0],
    [-1.0, 1.0],
    [1.0, -1.0],
    [1.0, 1.0],
];

/// Resamples the spring-chain nodes into a smooth Catmull-Rom centerline.
///
/// Near-coincident nodes are merged first (they appear when the cursor stops or reverses), and
/// each spline segment is subdivided by curvature when `adaptive_quality` is on.
///
/// `progress` follows the **original node index** (`(i + t) / (N − 1)`), exactly like every
/// legacy implementation (Windhawk D3D/GDI+, V3, TD). Width, fade and blur are therefore tied to
/// the chain itself: after a stop the nodes reel in one by one and the visible trail retracts
/// into the cursor. An arc-length parameterisation (tried in session 9) keeps the trail at full
/// length until the tail arrives and makes the whole ribbon "breathe" whenever its total length
/// changes. Using the index of the *merged* points instead (the pre-session-9 bug) would stretch
/// the taper over whatever nodes survive the merge, so merged nodes keep their original index.
pub fn build_samples(nodes: &[(f32, f32, f32)], config: &AppConfig, out: &mut Vec<Sample>) {
    out.clear();
    let total = nodes.len();
    if total == 0 {
        return;
    }
    let index_scale = 1.0 / (total - 1).max(1) as f32;

    // 1. Merge near-duplicate nodes: (x, y, speed, original index).
    let mut pts: Vec<(f32, f32, f32, f32)> = Vec::with_capacity(total);
    for (i, &(x, y, speed)) in nodes.iter().enumerate() {
        if let Some(&(px, py, _, _)) = pts.last() {
            let dx = x - px;
            let dy = y - py;
            if dx * dx + dy * dy < MIN_NODE_SPACING * MIN_NODE_SPACING {
                continue;
            }
        }
        pts.push((x, y, speed, i as f32));
    }

    let n = pts.len();
    if n == 1 {
        // Collapsed chain (cursor at rest): a single point, drawn as a round dot.
        let (x, y, speed, _) = pts[0];
        out.push(Sample {
            x,
            y,
            speed,
            progress: 0.0,
        });
        return;
    }

    // Phantom control points mirrored past both ends so the drawn curve runs from the very
    // first point (the spring head) to the very last node instead of skipping them.
    let first = pts[0];
    let second = pts[1];
    let last = pts[n - 1];
    let before_last = pts[n - 2];
    let phantom_start = (
        2.0 * first.0 - second.0,
        2.0 * first.1 - second.1,
        first.2,
        first.3,
    );
    let phantom_end = (
        2.0 * last.0 - before_last.0,
        2.0 * last.1 - before_last.1,
        last.2,
        last.3,
    );
    let ctrl = |i: isize| -> (f32, f32, f32, f32) {
        if i < 0 {
            phantom_start
        } else if i as usize >= n {
            phantom_end
        } else {
            pts[i as usize]
        }
    };

    // Bounded like the adaptive path: the UI offers 1–10, an imported config could ask for more.
    let base_steps = config.trail.interpolation_steps.clamp(1, 32) as usize;

    for seg in 0..(n - 1) {
        let c0 = ctrl(seg as isize - 1);
        let c1 = ctrl(seg as isize);
        let c2 = ctrl(seg as isize + 1);
        let c3 = ctrl(seg as isize + 2);
        let p0 = (c0.0, c0.1);
        let p1 = (c1.0, c1.1);
        let p2 = (c2.0, c2.1);
        let p3 = (c3.0, c3.1);
        let (sp1, sp2) = (c1.2, c2.2);
        let (i1, i2) = (c1.3, c2.3);

        let seg_dist = ((p2.0 - p1.0).powi(2) + (p2.1 - p1.1).powi(2)).sqrt();
        let steps = if config.trail.adaptive_quality {
            // Curvature-adaptive subdivision: a union of round capsules along a straight run is
            // exact at any spacing, so only bends need dense samples. The turning angle over the
            // segment gives a local radius; keep the chord sagitta under ~0.5 px.
            let turn = turning_angle(p0, p1, p2) + turning_angle(p1, p2, p3);
            let radius = seg_dist / turn.max(1e-3);
            let spacing = (4.0 * radius).sqrt().clamp(MIN_SAMPLE_SPACING, MAX_SAMPLE_SPACING);
            ((seg_dist / spacing).ceil() as usize).clamp(1, 32)
        } else {
            base_steps
        };

        for step in 0..steps {
            let t = step as f32 / steps as f32;
            let (x, y) = catmull_rom_centripetal(p0, p1, p2, p3, t);
            out.push(Sample {
                x,
                y,
                speed: sp1 + (sp2 - sp1) * t,
                progress: (i1 + (i2 - i1) * t) * index_scale,
            });
        }
    }
    // Close the curve at the last node so the tail cap sits on the final point.
    out.push(Sample {
        x: last.0,
        y: last.1,
        speed: last.2,
        progress: last.3 * index_scale,
    });
}

/// Straight RGBA (alpha ≤ 1) and radius for one sample of one layer.
fn layer_sample_style(
    s: &Sample,
    layer: &LayerConfig,
    config: &AppConfig,
    start_c: [f32; 4],
    end_c: [f32; 4],
    gradient: bool,
) -> ([f32; 4], f32) {
    let fade = apply_fade_curve(s.progress, config.trail.fade_mode);
    // `speed` is px per REFERENCE_FRAME (1/120 s). Windhawk D3D/GDI+ and V3 all normalise by
    // 20 in these same units (saturating at 2400 px/s); /10 doubled the boost at normal speed
    // and made the ribbon look fat and pulsing.
    let norm_speed = (s.speed / 20.0).min(1.0);
    let vel_width = 1.0 + norm_speed * config.trail.velocity_width_mult;
    let vel_alpha = 1.0 + norm_speed * config.trail.velocity_alpha_mult;
    let width = (config.trail.cursor_size * layer.width_factor * fade * vel_width)
        .max(config.trail.min_width);

    let mut c = if gradient {
        lerp_rgba(start_c, end_c, s.progress)
    } else {
        start_c
    };
    c[3] = (c[3] * layer.alpha_factor * fade * vel_alpha).clamp(0.0, 1.0);
    (c, width * 0.5)
}

/// Visible rectangle in world (virtual-desktop) pixels: `(x, y, width, height)`.
pub type Viewport = (f32, f32, f32, f32);

/// A viewport that never culls anything (tests, offscreen use).
pub const UNBOUNDED_VIEWPORT: Viewport = (-1.0e9, -1.0e9, 2.0e9, 2.0e9);

/// Appends one capsule per consecutive sample pair for `layer`, skipping capsules that are
/// invisible (both ends fainter than [`MIN_VISIBLE_ALPHA`]) or entirely outside `viewport`.
///
/// A single sample (chain collapsed on a resting cursor) becomes one zero-length capsule, i.e.
/// a round dot. Windhawk keeps that 4-layer dot at rest too; emitting nothing made the ribbon
/// pop in and out depending on whether the nodes had merged when the render loop parked.
pub fn build_layer_capsules(
    samples: &[Sample],
    layer_index: usize,
    layer: &LayerConfig,
    config: &AppConfig,
    rainbow_hue: f32,
    viewport: Viewport,
    dst: &mut Vec<CapsuleInstance>,
) {
    if !layer.enabled || samples.is_empty() {
        return;
    }
    let dot;
    let samples = if samples.len() == 1 {
        dot = [samples[0], samples[0]];
        &dot[..]
    } else {
        samples
    };
    let (vx, vy, vw, vh) = viewport;
    let (vx1, vy1) = (vx + vw, vy + vh);

    let sat = config.rainbow.saturation.clamp(0.0, 1.0);
    let lit = config.rainbow.lightness.clamp(0.0, 1.0);
    let start_c = if config.rainbow.enabled {
        let mut c = hsl_to_rgba(rainbow_hue, sat, lit);
        c[3] = layer.start_color[3];
        c
    } else {
        layer.start_color
    };
    let end_c = if config.rainbow.enabled {
        let mut c = hsl_to_rgba(rainbow_hue + 180.0, sat, lit * 0.6);
        c[3] = if config.trail.enable_gradient {
            layer.end_color[3]
        } else {
            layer.start_color[3]
        };
        c
    } else if config.trail.enable_gradient {
        layer.end_color
    } else {
        start_c
    };
    let gradient = config.trail.enable_gradient || config.rainbow.enabled;

    let (mut prev_c, mut prev_r) =
        layer_sample_style(&samples[0], layer, config, start_c, end_c, gradient);
    for (i, pair) in samples.windows(2).enumerate() {
        if dst.len() >= MAX_CAPSULES {
            break;
        }
        let (a, b) = (&pair[0], &pair[1]);
        let (cb, rb) = layer_sample_style(b, layer, config, start_c, end_c, gradient);
        let (ca, ra) = (prev_c, prev_r);
        prev_c = cb;
        prev_r = rb;

        // Invisible (faded-out tail) or fully off-screen capsules cost fill for nothing.
        if ca[3] < MIN_VISIBLE_ALPHA && cb[3] < MIN_VISIBLE_ALPHA {
            continue;
        }
        let reach = ra.max(rb) + 2.0;
        let (min_x, max_x) = (a.x.min(b.x) - reach, a.x.max(b.x) + reach);
        let (min_y, max_y) = (a.y.min(b.y) - reach, a.y.max(b.y) + reach);
        if max_x < vx || min_x > vx1 || max_y < vy || min_y > vy1 {
            continue;
        }

        // Blur ramps per vertex like Windhawk's HLSL (interpolated along the capsule in WGSL).
        let blur_at = |p: f32| (layer.start_blur + (layer.end_blur - layer.start_blur) * p).clamp(0.0, 1.0);
        dst.push(CapsuleInstance {
            a: [a.x, a.y],
            b: [b.x, b.y],
            radii: [ra, rb],
            color_a: ca,
            color_b: cb,
            params: [blur_at(a.progress), layer_index as f32, i as f32, blur_at(b.progress)],
        });
    }
}

/// The unified cross-platform hardware-accelerated cursor effects renderer.
///
/// Encapsulates:
/// - CPU physics simulation ([`TrailChain`], velocity squish head, particles, ripples, satellites).
/// - Adaptive Catmull-Rom resampling and tapered round capsule instancing.
/// - wgpu 30 graphics pipelines: depth max-coverage pre-pass, capsule color pass, and SDF billboard pass.
/// - On-overlay FPS heads-up display (HUD) rendered via a 3×5 dot-matrix bitmap font.
pub struct OverlayRenderer {
    capsule_prepass_pipeline: RenderPipeline,
    capsule_color_pipeline: RenderPipeline,
    circle_pipeline: RenderPipeline,
    cursor_pipeline: RenderPipeline,
    cursor_bind_layout: BindGroupLayout,
    cursor_sampler: Sampler,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    quad_vertex_buffer: Buffer,
    cursor_vertex_buffer: Buffer,
    capsule_instance_buffer: Buffer,
    circle_instance_buffer: Buffer,
    quad_initialized: bool,
    /// Depth attachments by size, most recently used first (see `ensure_depth`).
    depth: Vec<DepthTarget>,
    /// Uploaded OS cursor shape (GPU bypass); `None` while the feature is off or extraction
    /// has not produced a shape yet.
    cursor_shape: Option<BoundCursorShape>,
    /// False while the OS hides its pointer (fullscreen video, typing): the copy hides too.
    cursor_visible: bool,
    gpu_cursor: GpuCursorState,

    chain: TrailChain,
    node_scratch: Vec<(f32, f32, f32)>,
    samples: Vec<Sample>,
    ripples: Vec<Ripple>,
    particles: Vec<Particle>,
    squishy: SquishyState,
    satellites: SatelliteState,

    /// Raw pointer position of the last `update_mouse` call (satellites orbit it).
    last_mouse_pos: (f32, f32),
    /// Xorshift state for the particle jitter.
    rng_seed: u32,
    rainbow_hue: f32,
    start_time: Instant,

    capsules: Vec<CapsuleInstance>,
    circle_instances: Vec<CircleInstance>,
    pub virtual_origin: (f32, f32),
    /// Geometry uploaded by the last `render` call: (ribbon capsules, SDF instances).
    pub last_counts: (u32, u32),
    /// Frame rate shown by the on-overlay HUD (`fps_counter`); fed by the render loop.
    hud_fps: f32,
    /// Rectangle the HUD is anchored to (world pixels); `None` = the current viewport.
    pub hud_rect: Option<(f32, f32, f32, f32)>,
}

/// GPU-resident copy of an extracted OS cursor shape, keyed by its source handle.
struct BoundCursorShape {
    key: isize,
    width: u32,
    height: u32,
    hotspot: (u32, u32),
    is_arrow: bool,
    bind_group: BindGroup,
}

impl OverlayRenderer {
    pub fn new(device: &Device, format: TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shaders/render.wgsl").into()),
        });

        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("uniform_buffer"),
            size: std::mem::size_of::<OverlayUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("uniform_bind_group_layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("uniform_bind_group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });

        // Pre-multiplied "over" compositing.
        let blend_state = wgpu::BlendState {
            color: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
            alpha: wgpu::BlendComponent {
                src_factor: wgpu::BlendFactor::One,
                dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                operation: wgpu::BlendOperation::Add,
            },
        };

        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        };

        let quad_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        let capsule_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CapsuleInstance>() as u64,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 0,
                    shader_location: 1,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 8,
                    shader_location: 2,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x2,
                    offset: 16,
                    shader_location: 3,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 24,
                    shader_location: 4,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 40,
                    shader_location: 5,
                },
                wgpu::VertexAttribute {
                    format: wgpu::VertexFormat::Float32x4,
                    offset: 56,
                    shader_location: 6,
                },
            ],
        };

        let capsule_pipeline = |label: &str, prepass: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(label),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some("vs_capsule"),
                    buffers: &[Some(quad_layout.clone()), Some(capsule_layout.clone())],
                    compilation_options: Default::default(),
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if prepass { "fs_capsule_depth" } else { "fs_capsule_color" }),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(blend_state),
                        write_mask: if prepass {
                            wgpu::ColorWrites::empty()
                        } else {
                            wgpu::ColorWrites::ALL
                        },
                    })],
                    compilation_options: Default::default(),
                }),
                primitive,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    // Pre-pass: keep the highest coverage per pixel. Colour pass: draw only that.
                    depth_write_enabled: Some(prepass),
                    depth_compare: Some(if prepass {
                        wgpu::CompareFunction::Greater
                    } else {
                        wgpu::CompareFunction::GreaterEqual
                    }),
                    stencil: if prepass {
                        wgpu::StencilState::default()
                    } else {
                        // Paint-once guard: pass while `reference (= layer + 1) > stored`, then
                        // store the reference so later fragments of this layer fail.
                        let face = wgpu::StencilFaceState {
                            compare: wgpu::CompareFunction::Greater,
                            fail_op: wgpu::StencilOperation::Keep,
                            depth_fail_op: wgpu::StencilOperation::Keep,
                            pass_op: wgpu::StencilOperation::Replace,
                        };
                        wgpu::StencilState {
                            front: face,
                            back: face,
                            read_mask: 0xFF,
                            write_mask: 0xFF,
                        }
                    },
                    bias: wgpu::DepthBiasState::default(),
                }),
                multisample: wgpu::MultisampleState::default(),
                multiview_mask: None,
                cache: None,
            })
        };
        let capsule_prepass_pipeline = capsule_pipeline("capsule_prepass_pipeline", true);
        let capsule_color_pipeline = capsule_pipeline("capsule_color_pipeline", false);

        let circle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("circle_pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_circle"),
                buffers: &[
                    Some(quad_layout.clone()),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<CircleInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 0,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x2,
                                offset: 8,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 16,
                                shader_location: 3,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32,
                                offset: 20,
                                shader_location: 4,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 24,
                                shader_location: 5,
                            },
                        ],
                    }),
                ],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_circle"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive,
            // Billboards ignore the depth buffer but must declare the pass's depth format.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        // ---- GPU cursor bypass: textured quad drawn last, on top of everything -------------
        let cursor_bind_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cursor_bind_group_layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let cursor_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("cursor_pipeline_layout"),
                bind_group_layouts: &[Some(&bind_group_layout), Some(&cursor_bind_layout)],
                immediate_size: 0,
            });
        let cursor_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("cursor_pipeline"),
            layout: Some(&cursor_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_cursor"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<CursorVertex>() as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &[
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 0,
                            shader_location: 0,
                        },
                        wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x2,
                            offset: 8,
                            shader_location: 1,
                        },
                    ],
                })],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_cursor"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(blend_state),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive,
            // On-top sprite: ignore depth/stencil, same declaration as the SDF billboards.
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(false),
                depth_compare: Some(wgpu::CompareFunction::Always),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });
        let cursor_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("cursor_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            ..Default::default()
        });

        let quad_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("quad_vertex_buffer"),
            size: (std::mem::size_of::<[f32; 2]>() * 6) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let cursor_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("cursor_vertex_buffer"),
            size: (std::mem::size_of::<CursorVertex>() * 6) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let capsule_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("capsule_instance_buffer"),
            size: (std::mem::size_of::<CapsuleInstance>() * MAX_CAPSULES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let circle_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("circle_instance_buffer"),
            size: (std::mem::size_of::<CircleInstance>() * MAX_CIRCLE_INSTANCES) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            capsule_prepass_pipeline,
            capsule_color_pipeline,
            circle_pipeline,
            cursor_pipeline,
            cursor_bind_layout,
            cursor_sampler,
            uniform_buffer,
            bind_group,
            quad_vertex_buffer,
            cursor_vertex_buffer,
            capsule_instance_buffer,
            circle_instance_buffer,
            quad_initialized: false,
            depth: Vec::with_capacity(2),
            cursor_shape: None,
            cursor_visible: true,
            gpu_cursor: GpuCursorState::default(),
            chain: TrailChain::with_capacity(160),
            hud_fps: 0.0,
            hud_rect: None,
            node_scratch: Vec::with_capacity(160),
            samples: Vec::with_capacity(4096),
            ripples: Vec::with_capacity(32),
            particles: Vec::with_capacity(256),
            squishy: SquishyState::default(),
            satellites: SatelliteState {
                angle: 0.0,
                mirror_angle: 0.0,
            },
            last_mouse_pos: (0.0, 0.0),
            rng_seed: 12345,
            rainbow_hue: 0.0,
            start_time: Instant::now(),
            capsules: Vec::with_capacity(MAX_CAPSULES),
            circle_instances: Vec::with_capacity(MAX_CIRCLE_INSTANCES),
            virtual_origin: (0.0, 0.0),
            last_counts: (0, 0),
        }
    }

    /// Spawns the click effects (ripple + particle burst) for a button press at `(x, y)`.
    /// `button`: 0 = left, 1 = right, anything else = middle/extra.
    pub fn spawn_click(&mut self, button: usize, x: f32, y: f32, config: &AppConfig) {
        if !config.enabled {
            return;
        }
        if config.gpu_cursor.enabled {
            self.gpu_cursor.spawn_bounce();
        }
        let mask = ModeMask::from_mode(config.effect_mode);
        if config.ripple.enabled && mask.ripples {
            let color = match button {
                0 => config.ripple.color_left,
                1 => config.ripple.color_right,
                _ => config.ripple.color_middle,
            };
            if self.ripples.len() < 64 {
                self.ripples.push(Ripple {
                    x,
                    y,
                    time_elapsed: 0.0,
                    color,
                });
            }
        }
        if config.particles.enabled && mask.particles {
            let count = config.particles.count_per_click as usize;
            let room = MAX_PARTICLES.saturating_sub(self.particles.len());
            let spawn = count.min(room);
            for i in 0..spawn {
                // Evenly spaced directions with a little jitter read as a burst rather than
                // a random spray, and never leave a lopsided gap.
                let base = i as f32 / spawn.max(1) as f32 * std::f32::consts::TAU;
                let angle = base + (next_random(&mut self.rng_seed) - 0.5) * 0.5;
                let speed_mult = 0.7 + next_random(&mut self.rng_seed) * 0.6;
                let speed = config.particles.base_speed * speed_mult;
                let max_life = (config.particles.duration_ms as f32 / 1000.0)
                    * (0.6 + next_random(&mut self.rng_seed) * 0.8);
                self.particles.push(Particle {
                    x,
                    y,
                    vx: angle.cos() * speed,
                    vy: angle.sin() * speed,
                    life: max_life,
                    max_life,
                    size: config.particles.size * (0.7 + next_random(&mut self.rng_seed) * 0.6),
                    color: config.particles.color,
                });
            }
        }
    }

    /// Advances all simulations by `delta_seconds` towards cursor position `(x, y)`.
    pub fn update_mouse(&mut self, x: f32, y: f32, delta_seconds: f32, config: &AppConfig) {
        // Hitches up to 100 ms are integrated in full (sub-stepped) so the trail catches up
        // instead of falling behind the pointer; a longer gap only integrates 100 ms of it
        // (the render loop already caps the first frame after an idle park to one period).
        let dt = delta_seconds.clamp(0.001, MAX_FRAME_DELTA);

        for r in &mut self.ripples {
            r.time_elapsed += dt;
        }
        self.ripples.retain(|r| r.time_elapsed < (config.ripple.duration_ms as f32 / 1000.0));

        for p in &mut self.particles {
            p.life -= dt;
            if p.life > 0.0 {
                p.vy += config.particles.gravity * dt;
                let friction_mult = config.particles.friction.powf(dt * 60.0);
                p.vx *= friction_mult;
                p.vy *= friction_mult;
                p.x += p.vx * dt;
                p.y += p.vy * dt;
            }
        }
        self.particles.retain(|p| p.life > 0.0);

        let mask = ModeMask::from_mode(config.effect_mode);
        if config.satellites.enabled && mask.satellites {
            self.satellites.angle = (self.satellites.angle + config.satellites.speed * dt).rem_euclid(std::f32::consts::TAU);
            self.satellites.mirror_angle = (self.satellites.mirror_angle - config.satellites.speed * dt).rem_euclid(std::f32::consts::TAU);
        }

        if config.rainbow.enabled {
            // `speed` is expressed in degrees per 60 Hz frame so existing presets keep their feel;
            // scaling by dt makes the cycle rate independent of the actual frame rate.
            self.rainbow_hue = (self.rainbow_hue + config.rainbow.speed * dt * 60.0).rem_euclid(360.0);
        }

        let target_len = (config.trail.length as usize).clamp(4, 150);
        self.chain.resize(target_len, x, y);
        self.chain.advance(x, y, dt, config);
        let dt_scale = (dt / REFERENCE_FRAME).clamp(0.1, 5.0);

        if config.head.enabled {
            self.squishy.step(x, y, dt_scale, config);
        } else {
            self.squishy.initialized = false;
        }

        // GPU cursor bypass: chase the travel direction with the arrow and run down the
        // click bump. Uses the previous pointer position, so it must run before
        // `last_mouse_pos` is overwritten below.
        if config.enabled && config.gpu_cursor.enabled {
            let is_arrow = self.cursor_shape.as_ref().map(|s| s.is_arrow).unwrap_or(true);
            self.gpu_cursor.update(
                x - self.last_mouse_pos.0,
                y - self.last_mouse_pos.1,
                dt,
                config.gpu_cursor.rotate_with_movement && is_arrow,
                config.gpu_cursor.rotation_smoothing,
            );
        } else {
            self.gpu_cursor.reset();
        }

        self.last_mouse_pos = (x, y);
    }

    pub fn is_animating(&self, config: &AppConfig) -> bool {
        if !config.enabled {
            return false;
        }
        let mask = ModeMask::from_mode(config.effect_mode);
        (config.satellites.enabled && mask.satellites)
            || (config.rainbow.enabled && config.trail.enabled && mask.trail)
            || !self.ripples.is_empty()
            || !self.particles.is_empty()
            || (mask.trail && self.chain.is_moving())
            || (config.head.enabled && mask.head && self.squishy.is_animating(self.last_mouse_pos))
            // GPU cursor: rotation easing and the click bump must finish before parking.
            || (config.gpu_cursor.enabled
                && self
                    .gpu_cursor
                    .is_animating(config.gpu_cursor.click_scale_duration_ms))
    }

    /// Moves a depth attachment of exactly `width × height` to the front of the cache, creating
    /// it if needed. Two sizes are kept (the overlay surface and the snapshot crop), so a
    /// capture burst no longer reallocates the full-desktop depth buffer twice per frame —
    /// that hitch distorted exactly the transients a burst is meant to record.
    fn ensure_depth(&mut self, device: &Device, width: u32, height: u32) {
        let width = width.max(1);
        let height = height.max(1);
        if let Some(i) = self
            .depth
            .iter()
            .position(|d| d.width == width && d.height == height)
        {
            self.depth.swap(0, i);
            return;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("ribbon_depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        self.depth.insert(
            0,
            DepthTarget {
                width,
                height,
                view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            },
        );
        self.depth.truncate(2);
    }

    /// Rebuilds the centerline and the capsule list for every enabled layer.
    /// Returns the capsule index range of each layer.
    fn build_ribbon(
        &mut self,
        config: &AppConfig,
        mask: &ModeMask,
        viewport: Viewport,
    ) -> [(u32, u32); 4] {
        let mut ranges = [(0u32, 0u32); 4];
        self.capsules.clear();
        if !(config.enabled && config.trail.enabled && mask.trail) {
            self.samples.clear();
            return ranges;
        }

        self.node_scratch.clear();
        // Legacy (Windhawk / V3 / TD) builds the ribbon from the spring chain only.
        // Prepending the raw pointer here stretched a full-width head capsule across
        // the spring lag and produced a blob at the head.
        self.node_scratch
            .extend(self.chain.nodes.iter().map(|n| (n.x, n.y, n.speed)));
        build_samples(&self.node_scratch, config, &mut self.samples);

        for (layer_idx, range) in ranges.iter_mut().enumerate() {
            let start = self.capsules.len() as u32;
            if mask.layers[layer_idx] {
                build_layer_capsules(
                    &self.samples,
                    layer_idx,
                    &config.trail.layers[layer_idx],
                    config,
                    self.rainbow_hue,
                    viewport,
                    &mut self.capsules,
                );
            }
            *range = (start, self.capsules.len() as u32);
        }
        ranges
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        screen_width: u32,
        screen_height: u32,
        config: &AppConfig,
    ) {
        if !self.quad_initialized {
            queue.write_buffer(&self.quad_vertex_buffer, 0, bytemuck::cast_slice(&QUAD_VERTICES));
            self.quad_initialized = true;
        }
        self.ensure_depth(device, screen_width, screen_height);

        let uniforms = OverlayUniforms {
            screen_size: [screen_width as f32, screen_height as f32],
            virtual_origin: [self.virtual_origin.0, self.virtual_origin.1],
            time: self.start_time.elapsed().as_secs_f32(),
            padding: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        let mask = ModeMask::from_mode(config.effect_mode);

        // ---- Ribbon capsules --------------------------------------------------------------
        let viewport: Viewport = (
            self.virtual_origin.0,
            self.virtual_origin.1,
            screen_width as f32,
            screen_height as f32,
        );
        let layer_ranges = self.build_ribbon(config, &mask, viewport);
        let num_capsules = self.capsules.len().min(MAX_CAPSULES);
        if num_capsules > 0 {
            queue.write_buffer(
                &self.capsule_instance_buffer,
                0,
                bytemuck::cast_slice(&self.capsules[..num_capsules]),
            );
        }

        // ---- SDF billboards -----------------------------------------------------------------
        let mut circle_instances = std::mem::take(&mut self.circle_instances);
        circle_instances.clear();

        if config.fps_counter.enabled {
            let text = format!("{} FPS", self.hud_fps.round().max(0.0) as u32);
            let width = hud_text_width(&text);
            let height = 5.0 * HUD_CELL;
            let (rx, ry, rw, rh) = self.hud_rect.unwrap_or((
                self.virtual_origin.0,
                self.virtual_origin.1,
                screen_width as f32,
                screen_height as f32,
            ));
            let x = if config.fps_counter.align_right {
                rx + rw - HUD_MARGIN - width
            } else {
                rx + HUD_MARGIN
            };
            let y = if config.fps_counter.align_bottom {
                ry + rh - HUD_MARGIN - height
            } else {
                ry + HUD_MARGIN
            };
            push_hud_text(&mut circle_instances, &text, x, y);
        }

        for r in self.ripples.iter().filter(|_| mask.ripples) {
            let duration_sec = config.ripple.duration_ms as f32 / 1000.0;
            let progress = (r.time_elapsed / duration_sec).clamp(0.0, 1.0);
            let fade = apply_fade_curve(progress, config.trail.fade_mode);
            let progress_ease = 1.0 - (1.0 - progress).powi(3);
            let diameter = config.ripple.max_diameter * progress_ease;
            let width = config.ripple.start_width * fade;
            if diameter > 0.1 && width >= 0.5 {
                let mut c = r.color;
                c[0] *= fade;
                c[1] *= fade;
                c[2] *= fade;
                c[3] = fade;
                circle_instances.push(CircleInstance {
                    center: [r.x, r.y],
                    radius: [diameter * 0.5, diameter * 0.5],
                    angle: 0.0,
                    thickness: width,
                    color: c,
                });
            }
        }

        if config.head.enabled && config.enabled && mask.head {
            let scale_x = 1.0 + self.squishy.current_scale;
            let scale_y = (1.0 - self.squishy.current_scale * 0.5).max(0.3);
            let base_radius = config.head.size * 0.5;
            let mut c = config.head.color;
            c[0] *= c[3];
            c[1] *= c[3];
            c[2] *= c[3];
            circle_instances.push(CircleInstance {
                center: [self.squishy.pos_x, self.squishy.pos_y],
                radius: [base_radius * scale_x, base_radius * scale_y],
                angle: self.squishy.current_angle,
                thickness: if config.head.filled { -1.0 } else { config.head.thickness },
                color: c,
            });
        }

        for p in self.particles.iter().filter(|_| mask.particles) {
            let alpha = (p.life / p.max_life).clamp(0.0, 1.0);
            let mut c = p.color;
            c[0] *= alpha;
            c[1] *= alpha;
            c[2] *= alpha;
            c[3] = alpha;
            circle_instances.push(CircleInstance {
                center: [p.x, p.y],
                radius: [p.size * 0.5, p.size * 0.5],
                angle: 0.0,
                thickness: -1.0,
                color: c,
            });
        }

        if config.satellites.enabled && mask.satellites {
            let orbit_radius = config.satellites.orbit_diameter * 0.5;
            let sat_radius = config.satellites.size * 0.5;
            let origin = self.last_mouse_pos;
            let mut c = config.satellites.color;
            c[0] *= c[3];
            c[1] *= c[3];
            c[2] *= c[3];

            if config.satellites.show_orbit_ring {
                let mut rc = config.satellites.color;
                rc[0] *= rc[3] * 0.3;
                rc[1] *= rc[3] * 0.3;
                rc[2] *= rc[3] * 0.3;
                rc[3] *= 0.3;
                circle_instances.push(CircleInstance {
                    center: [origin.0, origin.1],
                    radius: [orbit_radius, orbit_radius],
                    angle: 0.0,
                    thickness: config.satellites.orbit_ring_thickness,
                    color: rc,
                });
            }

            let count = config.satellites.count.max(1);
            let step = std::f32::consts::TAU / count as f32;
            for i in 0..count {
                let angle = self.satellites.angle + i as f32 * step;
                circle_instances.push(CircleInstance {
                    center: [
                        origin.0 + angle.cos() * orbit_radius,
                        origin.1 + angle.sin() * orbit_radius,
                    ],
                    radius: [sat_radius, sat_radius],
                    angle: 0.0,
                    thickness: -1.0,
                    color: c,
                });
            }

            // Counter-rotating second ring, phase-shifted by half a step so the two sets
            // interleave instead of colliding.
            if config.satellites.dual_ring {
                let mut mc = c;
                mc[0] *= 0.7;
                mc[1] *= 0.7;
                mc[2] *= 0.7;
                mc[3] *= 0.7;
                let mirror_radius = sat_radius * 0.75;
                for i in 0..count {
                    let angle = self.satellites.mirror_angle + (i as f32 + 0.5) * step;
                    circle_instances.push(CircleInstance {
                        center: [
                            origin.0 + angle.cos() * orbit_radius,
                            origin.1 + angle.sin() * orbit_radius,
                        ],
                        radius: [mirror_radius, mirror_radius],
                        angle: 0.0,
                        thickness: -1.0,
                        color: mc,
                    });
                }
            }
        }

        let num_circle_instances = circle_instances.len().min(MAX_CIRCLE_INSTANCES);
        if num_circle_instances > 0 {
            queue.write_buffer(
                &self.circle_instance_buffer,
                0,
                bytemuck::cast_slice(&circle_instances[..num_circle_instances]),
            );
        }

        // ---- GPU cursor shape ---------------------------------------------------------------
        // Built from `last_mouse_pos` + the current bounce/rotation state; drawn last so the
        // cursor always sits on top of the trail and effects.
        let cursor_draw = if config.enabled && config.gpu_cursor.enabled && self.cursor_visible {
            self.cursor_shape.as_ref().map(|shape| {
                let scale = self.gpu_cursor.bounce_scale(
                    config.gpu_cursor.click_scale_percent,
                    config.gpu_cursor.click_scale_duration_ms,
                );
                let rotation = if shape.is_arrow {
                    self.gpu_cursor.rotation
                } else {
                    0.0
                };
                cursor_quad_vertices(
                    self.last_mouse_pos,
                    shape.width,
                    shape.height,
                    shape.hotspot,
                    rotation,
                    scale,
                )
            })
        } else {
            None
        };
        if let Some(vertices) = cursor_draw {
            queue.write_buffer(&self.cursor_vertex_buffer, 0, bytemuck::cast_slice(&vertices));
        }

        // ---- Render pass ------------------------------------------------------------------
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("render_encoder"),
        });

        {
            let depth_view = &self.depth[0].view; // `ensure_depth` above put this size first
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay_rpass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0),
                        store: wgpu::StoreOp::Discard,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.set_vertex_buffer(0, self.quad_vertex_buffer.slice(..));

            if num_capsules > 0 {
                rpass.set_vertex_buffer(1, self.capsule_instance_buffer.slice(..));
                for (layer_idx, &(start, end)) in layer_ranges.iter().enumerate() {
                    let end = end.min(num_capsules as u32);
                    if end <= start {
                        continue;
                    }
                    // 1. Depth-only pre-pass: highest coverage per pixel wins.
                    rpass.set_pipeline(&self.capsule_prepass_pipeline);
                    rpass.draw(0..6, start..end);
                    // 2. Colour pass: fragments within the depth tolerance of the winner may
                    //    blend, but the stencil lets only the first of them paint each pixel.
                    rpass.set_pipeline(&self.capsule_color_pipeline);
                    rpass.set_stencil_reference(layer_idx as u32 + 1);
                    rpass.draw(0..6, start..end);
                }
            }

            if num_circle_instances > 0 {
                rpass.set_pipeline(&self.circle_pipeline);
                rpass.set_vertex_buffer(1, self.circle_instance_buffer.slice(..));
                rpass.draw(0..6, 0..num_circle_instances as u32);
            }

            // 3. GPU cursor on top of everything (group 0 uniform bind stays valid).
            if cursor_draw.is_some()
                && let Some(shape) = &self.cursor_shape
            {
                rpass.set_pipeline(&self.cursor_pipeline);
                rpass.set_vertex_buffer(0, self.cursor_vertex_buffer.slice(..));
                rpass.set_bind_group(1, &shape.bind_group, &[]);
                rpass.draw(0..6, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));

        self.last_counts = (num_capsules as u32, num_circle_instances as u32);
        self.circle_instances = circle_instances;
    }

    /// Updates the frame rate shown by the on-overlay HUD.
    pub fn set_hud_fps(&mut self, fps: f32) {
        self.hud_fps = if fps.is_finite() { fps } else { 0.0 };
    }

    /// Uploads an extracted OS cursor shape when its source handle (or size) changed.
    /// Returns `true` if the GPU-side texture was replaced — the caller should then force
    /// a few settle frames so a parked loop repaints with the new shape.
    pub fn set_cursor_shape(&mut self, device: &Device, queue: &Queue, shape: &CursorShape) -> bool {
        if shape.pixels.len() != (shape.width as usize) * (shape.height as usize) * 4 {
            return false;
        }
        if let Some(bound) = &self.cursor_shape
            && bound.key == shape.source_key
            && bound.width == shape.width
            && bound.height == shape.height
            && bound.hotspot == shape.hotspot
        {
            return false;
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("cursor_shape_texture"),
            size: wgpu::Extent3d {
                width: shape.width.max(1),
                height: shape.height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            // Sampled in `fs_cursor` (TEXTURE_BINDING) and filled by `write_texture` (COPY_DST).
            // Plain UNORM: the pixels are gamma-encoded and pre-multiplied, like the surface.
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &shape.pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(shape.width.max(1) * 4),
                rows_per_image: None,
            },
            wgpu::Extent3d {
                width: shape.width.max(1),
                height: shape.height.max(1),
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("cursor_bind_group"),
            layout: &self.cursor_bind_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.cursor_sampler),
                },
            ],
        });
        self.cursor_shape = Some(BoundCursorShape {
            key: shape.source_key,
            width: shape.width,
            height: shape.height,
            hotspot: shape.hotspot,
            is_arrow: shape.is_arrow,
            bind_group,
        });
        true
    }

    /// Drops the uploaded cursor shape (feature disabled); the quad stops drawing.
    /// Shows or hides the GPU cursor copy (mirrors the OS pointer visibility). Returns
    /// `true` when the state changed, so the caller can schedule a repaint.
    pub fn set_cursor_visible(&mut self, visible: bool) -> bool {
        let changed = self.cursor_visible != visible;
        self.cursor_visible = visible;
        changed
    }

    pub fn clear_cursor_shape(&mut self) {
        self.cursor_shape = None;
    }

    /// One-line description of the chain state relative to the pointer (diagnostics).
    pub fn chain_summary(&self) -> String {
        let (cx, cy) = self.last_mouse_pos;
        let offsets: Vec<String> = self
            .chain
            .nodes
            .iter()
            .take(5)
            .map(|n| format!("({:+.1},{:+.1})", n.x - cx, n.y - cy))
            .collect();
        let farthest = self
            .chain
            .nodes
            .iter()
            .map(|n| ((n.x - cx).powi(2) + (n.y - cy).powi(2)).sqrt())
            .fold(0.0f32, f32::max);
        let moving = self.chain.nodes.iter().filter(|n| n.speed > MIN_DIRECTION_SPEED).count();
        format!(
            "nodes={} first5={} farthest={farthest:.1} moving={moving} brush=({:+.1},{:+.1})",
            self.chain.nodes.len(),
            offsets.join(" "),
            self.chain.brush.0 - cx,
            self.chain.brush.1 - cy
        )
    }

    pub fn clear_active_state(&mut self) {
        self.chain.reset();
        self.squishy.initialized = false;
        self.samples.clear();
        self.ripples.clear();
        self.particles.clear();
        self.capsules.clear();
        self.circle_instances.clear();
        self.gpu_cursor.reset();
    }

    pub fn render_clear(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        view: &wgpu::TextureView,
    ) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("clear_encoder"),
        });

        {
            let _rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay_clear_rpass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> AppConfig {
        AppConfig::default()
    }

    #[test]
    fn full_mode_enables_everything() {
        assert_eq!(ModeMask::from_mode(EffectMode::FourLayerGlow), ModeMask::ALL);
    }

    #[test]
    fn minimal_mode_keeps_only_core_and_spine() {
        let m = ModeMask::from_mode(EffectMode::Minimal);
        assert!(m.trail && m.head);
        assert_eq!(m.layers, [false, false, true, true]);
        assert!(!m.ripples && !m.particles && !m.satellites);
    }

    #[test]
    fn exclusive_modes_disable_the_trail() {
        for mode in [EffectMode::ParticlesOnly, EffectMode::SatellitesOnly] {
            let m = ModeMask::from_mode(mode);
            assert!(!m.trail, "{mode:?} must hide the ribbon");
            assert!(m.head, "{mode:?} keeps the head as the cursor anchor");
        }
        assert!(ModeMask::from_mode(EffectMode::ParticlesOnly).particles);
        assert!(!ModeMask::from_mode(EffectMode::ParticlesOnly).satellites);
        assert!(ModeMask::from_mode(EffectMode::SatellitesOnly).satellites);
        assert!(!ModeMask::from_mode(EffectMode::SatellitesOnly).particles);
    }

    #[test]
    fn fade_curves_start_bright_and_end_dark() {
        for mode in 0..5 {
            assert!(apply_fade_curve(0.0, mode) > 0.9, "mode {mode} start");
            assert!(apply_fade_curve(1.0, mode) < 0.1, "mode {mode} end");
        }
    }

    #[test]
    fn catmull_rom_interpolates_endpoints() {
        let p = catmull_rom((0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0), 0.0);
        assert!((p.0 - 1.0).abs() < 1e-6 && (p.1 - 1.0).abs() < 1e-6);
        let q = catmull_rom((0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0), 1.0);
        assert!((q.0 - 2.0).abs() < 1e-6 && q.1.abs() < 1e-6);
    }

    #[test]
    fn samples_follow_a_straight_line_with_monotonic_progress() {
        let nodes: Vec<(f32, f32, f32)> = (0..10).map(|i| (i as f32 * 10.0, 50.0, 5.0)).collect();
        let mut out = Vec::new();
        build_samples(&nodes, &cfg(), &mut out);
        assert!(out.len() >= nodes.len(), "spline emits at least one sample per segment");
        // The curve starts at the very first input point and ends at the last node.
        assert_eq!(out.first().map(|s| (s.x, s.y)), Some((0.0, 50.0)));
        assert_eq!(out.last().map(|s| (s.x, s.y)), Some((90.0, 50.0)));
        assert!(out.windows(2).all(|w| w[1].progress >= w[0].progress));
        assert!((out.last().unwrap().progress - 1.0).abs() < 1e-6);
        assert!(out.iter().all(|s| (s.y - 50.0).abs() < 1e-3), "straight input stays straight");
    }

    #[test]
    fn coincident_nodes_are_merged_and_produce_finite_geometry() {
        // Cursor parked: every node on the same spot, then a hairpin.
        let mut nodes = vec![(100.0, 100.0, 0.0); 20];
        nodes.extend([(100.0, 100.0, 0.0), (140.0, 100.0, 30.0), (100.0, 101.0, 30.0), (60.0, 100.0, 30.0)]);
        let mut out = Vec::new();
        build_samples(&nodes, &cfg(), &mut out);
        assert!(!out.is_empty());
        assert!(out.iter().all(|s| s.x.is_finite() && s.y.is_finite() && s.progress.is_finite()));

        let mut capsules = Vec::new();
        let config = cfg();
        build_layer_capsules(&out, 0, &config.trail.layers[0], &config, 0.0, UNBOUNDED_VIEWPORT, &mut capsules);
        // The stacked nodes keep their index, so the short run sits near the faded-out tail;
        // capsules there may be culled as invisible, but never more than one per sample pair.
        assert!(!capsules.is_empty() && capsules.len() < out.len());
        for c in &capsules {
            assert!(c.radii.iter().all(|r| r.is_finite() && *r >= config.trail.min_width * 0.5));
            assert!(c.color_a[3] <= 1.0 && c.color_b[3] <= 1.0, "alpha is clamped");
            assert_eq!(c.params[1], 0.0, "layer index");
        }
    }

    #[test]
    fn fewer_than_four_unique_nodes_still_draw() {
        let nodes = [(0.0, 0.0, 1.0), (10.0, 0.0, 1.0), (20.0, 0.0, 1.0)];
        let mut out = Vec::new();
        build_samples(&nodes, &cfg(), &mut out);
        assert!(out.len() >= 3);
        assert_eq!(out.first().map(|s| (s.x, s.y)), Some((0.0, 0.0)));
        assert_eq!(out.last().map(|s| (s.x, s.y)), Some((20.0, 0.0)));
        assert_eq!(out.last().unwrap().progress, 1.0);
        let single = [(5.0, 5.0, 0.0)];
        out.clear();
        build_samples(&single, &cfg(), &mut out);
        assert_eq!(out.len(), 1);
    }

    /// Regression: with a far lead point ahead of a tightly bunched chain, the uniform spline
    /// used to hook back on itself at the head. The centripetal spline must move monotonically
    /// from the first control point towards the rest of the chain.
    #[test]
    fn head_segment_never_hooks_backwards() {
        let mut nodes = vec![(200.0, 0.0, 40.0)]; // far first control point
        for i in 0..30 {
            nodes.push((100.0 - i as f32 * 2.0, (i as f32 * 0.3).sin() * 0.5, 5.0));
        }
        let mut out = Vec::new();
        build_samples(&nodes, &cfg(), &mut out);
        let head: Vec<&Sample> = out.iter().take_while(|s| s.x > 100.0).collect();
        assert!(head.len() >= 2, "the head segment is sampled");
        assert!(
            head.windows(2).all(|w| w[1].x <= w[0].x + 1e-3),
            "x must decrease monotonically from the first control point into the chain"
        );
        assert!(head.iter().all(|s| s.y.abs() < 2.0), "no lateral hook near the head");
        assert!(out.iter().all(|s| s.x.is_finite() && s.y.is_finite()));
    }

    #[test]
    fn centripetal_spline_interpolates_endpoints_and_stays_between_them() {
        let p = catmull_rom_centripetal((0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0), 0.0);
        assert!((p.0 - 1.0).abs() < 1e-5 && (p.1 - 1.0).abs() < 1e-5);
        let q = catmull_rom_centripetal((0.0, 0.0), (1.0, 1.0), (2.0, 0.0), (3.0, 1.0), 1.0);
        assert!((q.0 - 2.0).abs() < 1e-5 && q.1.abs() < 1e-5);
        // Very uneven spacing (far control point) must not push the curve outside the hull.
        for i in 1..20 {
            let t = i as f32 / 20.0;
            let m = catmull_rom_centripetal((500.0, 0.0), (100.0, 0.0), (98.0, 0.0), (96.0, 0.0), t);
            assert!(m.0 <= 100.0 + 1e-4 && m.0 >= 98.0 - 1e-4, "x={} at t={t}", m.0);
        }
    }

    #[test]
    fn disabled_layer_emits_no_capsules() {
        let nodes: Vec<(f32, f32, f32)> = (0..8).map(|i| (i as f32 * 8.0, 0.0, 1.0)).collect();
        let mut out = Vec::new();
        let mut config = cfg();
        build_samples(&nodes, &config, &mut out);
        config.trail.layers[1].enabled = false;
        let mut capsules = Vec::new();
        build_layer_capsules(&out, 1, &config.trail.layers[1], &config, 0.0, UNBOUNDED_VIEWPORT, &mut capsules);
        assert!(capsules.is_empty());
    }

    #[test]
    fn capsule_radius_tapers_towards_the_tail() {
        let nodes: Vec<(f32, f32, f32)> = (0..40).map(|i| (i as f32 * 6.0, 0.0, 0.0)).collect();
        let mut out = Vec::new();
        let config = cfg();
        build_samples(&nodes, &config, &mut out);
        let mut capsules = Vec::new();
        build_layer_capsules(&out, 2, &config.trail.layers[2], &config, 0.0, UNBOUNDED_VIEWPORT, &mut capsules);
        let head_r = capsules.first().unwrap().radii[0];
        let tail_r = capsules.last().unwrap().radii[1];
        assert!(head_r > tail_r, "head {head_r} should be wider than tail {tail_r}");
        assert!((tail_r - config.trail.min_width * 0.5).abs() < 1e-4, "tail reaches min_width");
    }

    // ---- Trail chain physics (GPU-free) ----------------------------------------------------

    const FRAME: f32 = 1.0 / 60.0;

    fn chain(len: usize, x: f32, y: f32) -> TrailChain {
        let mut c = TrailChain::with_capacity(len);
        c.resize(len, x, y);
        c
    }

    #[test]
    fn chain_grows_from_its_tail() {
        let mut c = chain(4, 100.0, 50.0);
        assert!(c.nodes.iter().all(|n| (n.x, n.y) == (100.0, 50.0)));
        c.nodes[3].x = 20.0;
        c.resize(6, 100.0, 50.0);
        assert_eq!(c.nodes.len(), 6);
        assert!(c.nodes[4..].iter().all(|n| n.x == 20.0 && n.speed == 0.0));
        c.resize(2, 0.0, 0.0);
        assert_eq!(c.nodes.len(), 2);
    }

    #[test]
    fn chain_settles_on_a_stopped_pointer_with_bounded_overshoot() {
        let cfg = cfg();
        let mut c = chain(40, 0.0, 0.0);
        let mut x = 0.0;
        for _ in 0..90 {
            x += 15.0;
            c.advance(x, 0.0, FRAME, &cfg);
        }
        let mut max_overshoot = 0.0f32;
        for _ in 0..300 {
            c.advance(x, 0.0, FRAME, &cfg);
            for n in &c.nodes {
                max_overshoot = max_overshoot.max(n.x - x);
            }
        }
        // Spring-damper may overshoot slightly; it must stay bounded and then settle.
        assert!(
            max_overshoot < 80.0,
            "head shot {max_overshoot:.1} px past the stopped pointer"
        );
        for (i, n) in c.nodes.iter().enumerate() {
            assert!(
                (n.x - x).abs() < 1.0,
                "node {i} is {:.3} px from the pointer after settling",
                n.x - x
            );
            assert!(n.y.abs() < 1.0, "node {i} drifted sideways: y={}", n.y);
        }
        assert!(!c.is_moving(), "chain comes to rest");
    }

    #[test]
    fn lazy_brush_dead_zone_holds_then_drags() {
        let mut config = cfg();
        config.trail.lazy_enabled = true;
        config.trail.lazy_radius = 30.0;
        config.trail.lazy_friction = 0.4;
        let mut c = chain(10, 0.0, 0.0);

        // Pointer moves 10 px/frame: while inside the dead zone the brush must not move.
        let mut x = 0.0;
        let mut saw_dead_zone = false;
        let mut saw_follow = false;
        for _ in 0..60 {
            x += 10.0;
            let prev_brush = c.brush;
            c.advance(x, 0.0, FRAME, &config);
            let dist = (x - c.brush.0).hypot(0.0 - c.brush.1);
            let brush_moved = (c.brush.0 - prev_brush.0).abs() > 1e-6;
            if dist <= config.trail.lazy_radius + 0.5 && !brush_moved && x <= config.trail.lazy_radius {
                saw_dead_zone = true;
            }
            if dist > config.trail.lazy_radius + 0.5 && brush_moved {
                saw_follow = true;
            }
            // Once moving, the brush keeps roughly `radius` behind the pointer.
            if x > config.trail.lazy_radius * 3.0 {
                assert!(
                    dist >= config.trail.lazy_radius - 10.0 && dist <= config.trail.lazy_radius + 60.0,
                    "brush distance {dist:.1} should hover near radius 30"
                );
            }
        }
        assert!(saw_dead_zone, "brush never held still inside the dead zone");
        assert!(saw_follow, "brush never started following after the dead zone");

        // Disabled: the brush tracks the pointer exactly.
        let off = cfg();
        let mut c2 = chain(5, 0.0, 0.0);
        c2.advance(100.0, 40.0, FRAME, &off);
        assert_eq!(c2.brush, (100.0, 40.0), "disabled lazy brush mirrors pointer");
    }

    #[test]
    fn distance_constraint_bounds_node_gaps() {
        let cfg = cfg();
        let mut c = chain(40, 0.0, 0.0);
        // Teleport far in one frame; springs alone would stretch the chain across the gap.
        c.advance(2000.0, 500.0, FRAME, &cfg);
        for _ in 0..30 {
            c.advance(2000.0, 500.0, FRAME, &cfg);
        }
        for (i, w) in c.nodes.windows(2).enumerate() {
            let d = (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
            assert!(
                d <= MAX_NODE_GAP + 1e-3,
                "gap between nodes {i} and {} is {d:.2} px",
                i + 1
            );
        }
    }

    #[test]
    fn fast_flick_does_not_slingshot_past_a_stopped_pointer() {
        // Regression: the old 64 px clamp sat *below* the chain's natural steady-state gap
        // during fast motion, so it fired every frame and injected `delta/dt_scale` into
        // velocity — the trail visibly accelerated on sudden flicks.
        let cfg = cfg();
        let mut c = chain(40, 0.0, 0.0);
        for i in 0..30 {
            c.advance(i as f32 * 8.0, 0.0, FRAME, &cfg);
        }
        // Hard flick to a far target, then stop dead on it.
        let target = 1500.0;
        for _ in 0..5 {
            c.advance(target, 0.0, FRAME, &cfg);
        }
        let mut max_ahead = f32::MIN;
        for _ in 0..240 {
            c.advance(target, 0.0, FRAME, &cfg);
            for n in &c.nodes {
                max_ahead = max_ahead.max(n.x - target);
            }
        }
        assert!(
            max_ahead < 200.0,
            "head shot {max_ahead:.1} px past the stopped pointer (slingshot)"
        );
        for (i, n) in c.nodes.iter().enumerate() {
            assert!(
                (n.x - target).abs() < 5.0,
                "node {i} is {:.3} px from the pointer after settling",
                n.x - target
            );
        }
    }

    #[test]
    fn sustained_fast_motion_never_engages_the_distance_clamp() {
        // Natural steady-state gap at these defaults is a small multiple of the per-frame
        // velocity — well under MAX_NODE_GAP. If the clamp were tight again (64 px) it would
        // pin every consecutive pair to exactly MAX_NODE_GAP while moving.
        let cfg = cfg();
        let mut c = chain(40, 0.0, 0.0);
        let mut min_pinned = f32::MAX;
        for i in 0..90 {
            // ~4800 px/s along x.
            c.advance(i as f32 * 40.0, 0.0, FRAME, &cfg);
            if i < 20 {
                continue; // let the chain stretch out
            }
            for w in c.nodes.windows(2) {
                let d = (w[1].x - w[0].x).hypot(w[1].y - w[0].y);
                min_pinned = min_pinned.min(MAX_NODE_GAP - d);
            }
        }
        assert!(
            min_pinned > 1.0,
            "a consecutive pair sat within 1 px of MAX_NODE_GAP during normal fast motion \
             (clamp fighting the springs)"
        );
    }

    #[test]
    fn sample_progress_follows_the_original_node_index() {
        // Legacy (Windhawk / V3 / TD) parameterise by node index: progress = (i + t) / (N − 1).
        // Uneven spacing must not move it (arc length would put x=51 at ~0.5).
        let nodes = [(0.0, 0.0, 5.0), (1.0, 0.0, 5.0), (51.0, 0.0, 5.0), (101.0, 0.0, 5.0)];
        let mut config = cfg();
        config.trail.interpolation_steps = 1;
        config.trail.adaptive_quality = false;
        let mut out = Vec::new();
        build_samples(&nodes, &config, &mut out);
        assert!(out.windows(2).all(|w| w[1].progress >= w[0].progress - 1e-6));
        assert!(out[0].progress.abs() < 1e-6);
        assert!((out.last().unwrap().progress - 1.0).abs() < 1e-6);
        let at51 = out.iter().find(|s| (s.x - 51.0).abs() < 1e-3).expect("sample at node 2");
        assert!((at51.progress - 2.0 / 3.0).abs() < 1e-4, "progress {}", at51.progress);
    }

    #[test]
    fn merged_nodes_keep_their_original_index() {
        // Head nodes 0..=4 collapsed on the cursor, then an even run: the survivors must keep
        // their place in the taper instead of being stretched over the whole [0, 1] range.
        let mut nodes = vec![(0.0, 0.0, 0.0); 5];
        nodes.extend((1..=5).map(|i| (i as f32 * 10.0, 0.0, 5.0)));
        let mut config = cfg();
        config.trail.interpolation_steps = 1;
        config.trail.adaptive_quality = false;
        let mut out = Vec::new();
        build_samples(&nodes, &config, &mut out);
        let first_run = out.iter().find(|s| (s.x - 10.0).abs() < 1e-3).unwrap();
        assert!((first_run.progress - 5.0 / 9.0).abs() < 1e-4, "node 5 → {}", first_run.progress);
    }

    #[test]
    fn a_collapsed_chain_still_draws_a_round_dot_per_layer() {
        // Windhawk keeps a 4-layer dot on a resting cursor; the ribbon must not pop out.
        let nodes = vec![(300.0, 200.0, 0.0); 40];
        let config = cfg();
        let mut out = Vec::new();
        build_samples(&nodes, &config, &mut out);
        assert_eq!(out.len(), 1);
        for (i, layer) in config.trail.layers.iter().enumerate() {
            let mut capsules = Vec::new();
            build_layer_capsules(&out, i, layer, &config, 0.0, UNBOUNDED_VIEWPORT, &mut capsules);
            assert_eq!(capsules.len(), 1, "layer {i} emits one dot");
            let c = capsules[0];
            assert_eq!(c.a, c.b, "zero-length capsule");
            let fade = apply_fade_curve(0.0, config.trail.fade_mode);
            let expected = config.trail.cursor_size * layer.width_factor * fade * 0.5;
            assert!((c.radii[0] - expected).abs() < 0.5, "layer {i} radius {} vs {expected}", c.radii[0]);
        }
    }

    /// Length of the part of the ribbon that is at least half opaque (the part the eye reads).
    fn visible_length(c: &TrailChain, config: &AppConfig) -> f32 {
        let nodes: Vec<(f32, f32, f32)> = c.nodes.iter().map(|n| (n.x, n.y, n.speed)).collect();
        let mut out = Vec::new();
        build_samples(&nodes, config, &mut out);
        out.windows(2)
            .filter(|w| apply_fade_curve(w[1].progress, config.trail.fade_mode) >= 0.5)
            .map(|w| (w[1].x - w[0].x).hypot(w[1].y - w[0].y))
            .sum()
    }

    #[test]
    fn stopping_retracts_the_visible_trail_into_the_cursor() {
        // Legacy behaviour: after a stop the visible (bright) half of the ribbon shrinks
        // steadily into the pointer. With arc-length progress it hung at full length.
        let config = cfg();
        let mut c = chain(80, 0.0, 0.0);
        let mut x = 0.0;
        for _ in 0..120 {
            x += 10.0;
            c.advance(x, 0.0, FRAME, &config);
        }
        let moving = visible_length(&c, &config);
        assert!(moving > 100.0, "a moving trail is visible ({moving:.0} px)");
        let mut prev = moving;
        let mut shrank_by_half_at = None;
        for frame in 0..240 {
            c.advance(x, 0.0, FRAME, &config);
            let len = visible_length(&c, &config);
            assert!(len <= prev + 2.0, "visible trail grew after the stop: {prev:.1} → {len:.1}");
            if shrank_by_half_at.is_none() && len < moving * 0.5 {
                shrank_by_half_at = Some(frame);
            }
            prev = len;
        }
        let half = shrank_by_half_at.expect("the visible trail never halved");
        assert!(half < 60, "took {half} frames to halve (should retract within ~1 s)");
    }

    #[test]
    fn a_tiny_nudge_keeps_the_chain_awake_until_the_head_arrives() {
        let config = cfg();
        let mut c = chain(20, 0.0, 0.0);
        c.advance(1.0, 0.0, FRAME, &config);
        assert!(c.is_moving(), "head is 1 px off the pointer: not at rest yet");
        for _ in 0..600 {
            c.advance(1.0, 0.0, FRAME, &config);
        }
        assert!(!c.is_moving());
        assert!((c.nodes[0].x - 1.0).abs() <= HEAD_REST_DISTANCE);
    }

    #[test]
    fn non_finite_state_resets_the_chain_on_the_pointer() {
        let mut config = cfg();
        config.trail.spring = f32::INFINITY;
        let mut c = chain(10, 0.0, 0.0);
        for _ in 0..3 {
            c.advance(50.0, 20.0, FRAME, &config);
        }
        assert!(c.nodes.iter().all(|n| n.x.is_finite() && n.y.is_finite()));
    }

    #[test]
    fn squishy_head_is_proportional_to_speed_not_saturated() {
        // Windhawk measures squish velocity in px per frame; px/s saturated at ~25 px/s.
        let mut config = cfg();
        config.head.enabled = true;
        let run = |px_per_frame: f32| {
            let mut h = SquishyState::default();
            let mut x = 0.0;
            for _ in 0..120 {
                x += px_per_frame;
                h.step(x, 0.0, FRAME / REFERENCE_FRAME, &config);
            }
            h.current_scale
        };
        let (slow, fast) = (run(1.0), run(6.0));
        assert!(slow > 0.0 && fast > slow * 3.0, "slow {slow:.3} fast {fast:.3}");
        // The head eases toward the pointer instead of snapping (smoothing 50 %).
        let mut h = SquishyState::default();
        h.step(0.0, 0.0, 1.0, &config);
        h.step(100.0, 0.0, 1.0, &config);
        assert!((h.pos_x - 50.0).abs() < 1e-3, "pos {}", h.pos_x);
    }

    #[test]
    fn chain_is_frame_rate_independent() {
        let cfg = cfg();
        let run = |fps: f32| {
            let mut c = chain(30, 0.0, 0.0);
            let frames = (fps * 1.0) as usize; // one simulated second
            for k in 0..frames {
                let t = k as f32 / fps;
                c.advance(600.0 * t, (t * 6.0).sin() * 80.0, 1.0 / fps, &cfg);
            }
            c.nodes.iter().map(|n| (n.x, n.y)).collect::<Vec<_>>()
        };
        let a = run(60.0);
        let b = run(240.0);
        for (i, (p, q)) in a.iter().zip(&b).enumerate() {
            let d = ((p.0 - q.0).powi(2) + (p.1 - q.1).powi(2)).sqrt();
            // Sub-stepped Euler is not bit-identical across frame rates; with the 1/120
            // reference frame the residual is a few px on mid-chain nodes after 1 s.
            assert!(d < 20.0, "node {i} differs by {d:.1} px between 60 and 240 fps");
        }
    }
}

