// =============================================================================
// FXCursor render shader (WGSL) — multi-monitor virtual space, pre-multiplied alpha.
//
// 1. CAPSULE RIBBON: every trail layer is a union of round capsules (one per sample pair).
//    Each capsule is an instanced quad whose fragment shader evaluates the analytic distance
//    to the segment, so joins and caps are round by construction and hairpins never fold.
//    The union is resolved with the depth buffer: fragment depth encodes
//    (layer band + coverage), a depth-only pre-pass keeps the highest coverage per pixel and
//    the colour pass draws only fragments whose depth is >= it (within a tiny epsilon). Overlapping capsules
//    of one layer therefore never blend twice, while later layers still composite "over".
// 2. SDF BILLBOARDS: head, ripples, particles and satellites as instanced circles/rings.
// 3. GPU CURSOR: the extracted OS cursor shape as a textured quad drawn last (on top).
// =============================================================================

struct Uniforms {
    screen_size: vec2<f32>,
    virtual_origin: vec2<f32>,
    time: f32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

/// Number of depth bands = maximum number of ribbon layers drawn in one pass.
const LAYER_BANDS: f32 = 4.0;

fn to_clip(world: vec2<f32>) -> vec4<f32> {
    let rel = world - uniforms.virtual_origin;
    let clip_x = (rel.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (rel.y / uniforms.screen_size.y) * 2.0;
    return vec4<f32>(clip_x, clip_y, 0.0, 1.0);
}

// ==========================================
// 1. CAPSULE RIBBON
// ==========================================

struct CapsuleVertexInput {
    @location(0) corner: vec2<f32>,       // Quad corner in [-1, 1]
    // Instance attributes (one capsule per instance)
    @location(1) a: vec2<f32>,            // Segment start (world)
    @location(2) b: vec2<f32>,            // Segment end (world)
    @location(3) radii: vec2<f32>,        // Radius at a, radius at b
    @location(4) color_a: vec4<f32>,      // Straight (non-premultiplied) RGBA at a
    @location(5) color_b: vec4<f32>,      // Straight RGBA at b
    @location(6) params: vec4<f32>,       // [blur at a, layer index, segment index, blur at b]
};

struct CapsuleVertexOutput {
    @builtin(position) @invariant clip_position: vec4<f32>,
    @location(0) world: vec2<f32>,
    @location(1) @interpolate(flat) a: vec2<f32>,
    @location(2) @interpolate(flat) b: vec2<f32>,
    @location(3) @interpolate(flat) radii: vec2<f32>,
    @location(4) @interpolate(flat) color_a: vec4<f32>,
    @location(5) @interpolate(flat) color_b: vec4<f32>,
    @location(6) @interpolate(flat) params: vec4<f32>,
};

struct CapsuleFragmentOutput {
    @location(0) color: vec4<f32>,
    @builtin(frag_depth) depth: f32,
};

@vertex
fn vs_capsule(model: CapsuleVertexInput) -> CapsuleVertexOutput {
    // (`clip_position` is declared invariant on the output struct so both capsule passes
    // rasterise identically.)
    var out: CapsuleVertexOutput;

    // Oriented bounding quad around the capsule, padded for the AA feather.
    let ab = model.b - model.a;
    let len = length(ab);
    var dir = vec2<f32>(1.0, 0.0);
    if (len > 1e-4) {
        dir = ab / len;
    }
    let nrm = vec2<f32>(-dir.y, dir.x);
    let pad = max(model.radii.x, model.radii.y) + 2.0;
    let center = (model.a + model.b) * 0.5;
    let half_len = len * 0.5 + pad;
    let world = center + dir * (model.corner.x * half_len) + nrm * (model.corner.y * pad);

    out.clip_position = to_clip(world);
    out.world = world;
    out.a = model.a;
    out.b = model.b;
    out.radii = model.radii;
    out.color_a = model.color_a;
    out.color_b = model.color_b;
    out.params = model.params;
    return out;
}

/// Shared capsule evaluation: premultiplied colour + depth (layer band + coverage).
fn capsule_fragment(in: CapsuleVertexOutput, depth_bias: f32) -> CapsuleFragmentOutput {
    // Closest point on the segment and the tapered radius / colour there.
    let ab = in.b - in.a;
    let len2 = max(dot(ab, ab), 1e-6);
    let t = clamp(dot(in.world - in.a, ab) / len2, 0.0, 1.0);
    let p = in.a + ab * t;
    let d = length(in.world - p);
    let r = max(mix(in.radii.x, in.radii.y, t), 0.35);
    let v = d / r;

    // Feather: `blur` is the fraction of the radius that fades out; never thinner than
    // ~1.5 screen pixels so the edge stays anti-aliased at any width.
    let blur = clamp(mix(in.params.x, in.params.w, t), 0.0, 1.0);
    let fw = fwidth(v);
    let effective_blur = max(blur, fw * 1.5);
    let edge = 1.0 - smoothstep(1.0 - effective_blur, 1.0, v);

    let c = mix(in.color_a, in.color_b, t);
    let alpha = clamp(c.a * edge, 0.0, 1.0);
    if (alpha < 0.002) {
        discard;
    }

    // Depth = layer band + coverage (+ head-favouring tie breaker). Higher wins.
    // Head segments have a low `seg` index and are drawn first, so at a fold the head must
    // win coverage ties; the bias decays with `seg` so deep tails stay inside the band.
    let layer = in.params.y;
    let seg = in.params.z;
    let seg_bias = 1e-3 / (1.0 + seg);
    let depth = (layer + 0.001 + seg_bias + alpha * 0.99) / LAYER_BANDS;

    var out: CapsuleFragmentOutput;
    out.color = vec4<f32>(c.rgb * alpha, alpha);
    out.depth = clamp(depth + depth_bias, 0.0, 1.0);
    return out;
}

/// Pre-pass: writes depth only (colour writes are masked in the pipeline).
@fragment
fn fs_capsule_depth(in: CapsuleVertexOutput) -> CapsuleFragmentOutput {
    return capsule_fragment(in, 0.0);
}

/// Colour pass: compared with GreaterEqual against the pre-pass depth. The epsilon absorbs
/// float-contraction differences between the two separately compiled pipelines (measured:
/// 2e-6 left single-pixel holes on NVIDIA). 1e-4 in depth units is ~4e-4 in coverage, far
/// below one visible step, so overlapping capsules still never double-blend visibly.
@fragment
fn fs_capsule_color(in: CapsuleVertexOutput) -> CapsuleFragmentOutput {
    return capsule_fragment(in, 1e-4);
}

// ==========================================
// 2. INSTANCED SDF BILLBOARD SHADER
// ==========================================

struct CircleVertexInput {
    @location(0) corner: vec2<f32>,       // Raw quad corner [-1..1, -1..1]

    // Instance attributes
    @location(1) center: vec2<f32>,
    @location(2) radius: vec2<f32>,
    @location(3) angle: f32,
    @location(4) thickness: f32,          // <0 means filled, >=0 is outline width
    @location(5) color: vec4<f32>,
};

struct CircleVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) local_pos: vec2<f32>,
    @location(1) radius: vec2<f32>,
    @location(2) thickness: f32,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_circle(model: CircleVertexInput) -> CircleVertexOutput {
    var out: CircleVertexOutput;

    let max_rad = max(model.radius.x, model.radius.y);
    let pad = max(model.thickness, 0.0) * 0.5 + 3.0;
    let half_size = max_rad + pad;
    let local_pos = model.corner * half_size;

    let cos_a = cos(model.angle);
    let sin_a = sin(model.angle);
    let rotated = vec2<f32>(
        local_pos.x * cos_a - local_pos.y * sin_a,
        local_pos.x * sin_a + local_pos.y * cos_a
    );

    out.clip_position = to_clip(model.center + rotated);
    out.local_pos = local_pos;
    out.radius = model.radius;
    out.thickness = model.thickness;
    out.color = model.color;

    return out;
}

@fragment
fn fs_circle(in: CircleVertexOutput) -> @location(0) vec4<f32> {
    let len = length(in.local_pos);
    let d_norm = length(in.local_pos / in.radius);

    var dist = -1.0;
    if (len > 0.0 && d_norm > 0.0) {
        dist = len * (d_norm - 1.0) / d_norm;
    }

    let fw = max(fwidth(dist), 0.75);

    var alpha = 0.0;
    if (in.thickness < 0.0) {
        // Filled circle/ellipse with screen-space derivative anti-aliasing
        alpha = 1.0 - smoothstep(-fw, fw, dist);
    } else {
        // Ring outline with screen-space derivative anti-aliasing
        let ring_dist = abs(dist);
        let half_t = in.thickness * 0.5;
        alpha = 1.0 - smoothstep(half_t - fw, half_t + fw, ring_dist);
    }

    let a = in.color.a * alpha;
    // Pre-multiplied alpha
    return vec4<f32>(in.color.rgb * a, a);
}

// ==========================================
// 3. GPU CURSOR SHAPE (textured quad)
// ==========================================
// Texture + sampler live in group 1; only the cursor entry points reference them, so the
// ribbon/billboard pipelines (layout with group 0 only) validate unchanged.

@group(1) @binding(0) var cursor_tex: texture_2d<f32>;
@group(1) @binding(1) var cursor_smp: sampler;

struct CursorVertexInput {
    @location(0) pos: vec2<f32>,           // World position (virtual-screen pixels)
    @location(1) uv: vec2<f32>,            // Texture coordinates (top-down)
};

struct CursorVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_cursor(v: CursorVertexInput) -> CursorVertexOutput {
    var out: CursorVertexOutput;
    out.clip_position = to_clip(v.pos);
    out.uv = v.uv;
    return out;
}

/// Sampled pixels are already pre-multiplied on the CPU → One / OneMinusSrcAlpha blend.
@fragment
fn fs_cursor(in: CursorVertexOutput) -> @location(0) vec4<f32> {
    return textureSample(cursor_tex, cursor_smp, in.uv);
}
