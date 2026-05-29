struct Uniforms {
    screen_size: vec2<f32>,
    time: f32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

// ==========================================
// 1. RIBBON SHADER
// ==========================================

struct RibbonVertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) tex: vec2<f32>, // [blur, v] where v is in [-1.0, 1.0]
};

struct RibbonVertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_ribbon(model: RibbonVertexInput) -> RibbonVertexOutput {
    var out: RibbonVertexOutput;
    
    let clip_x = (model.position.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (model.position.y / uniforms.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = model.color;
    out.uv = model.tex;
    
    return out;
}

@fragment
fn fs_ribbon(in: RibbonVertexOutput) -> @location(0) vec4<f32> {
    let blur = clamp(in.uv.x, 0.001, 1.0);
    let v = abs(in.uv.y);
    let alpha = 1.0 - smoothstep(1.0 - blur, 1.0, v);
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}

// ==========================================
// 2. INSTANCED SDF CIRCLE SHADER
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
    let padding = max(model.thickness, 0.0) * 0.5 + 2.0;
    let half_size = max_rad + padding;
    let local_pos = model.corner * half_size;
    
    let cos_a = cos(model.angle);
    let sin_a = sin(model.angle);
    let rotated_local = vec2<f32>(
        local_pos.x * cos_a - local_pos.y * sin_a,
        local_pos.x * sin_a + local_pos.y * cos_a
    );
    
    let world_pos = model.center + rotated_local;
    let clip_x = (world_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (world_pos.y / uniforms.screen_size.y) * 2.0;
    
    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
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
    
    var alpha = 0.0;
    if (in.thickness < -1.0) {
        // Blurred filled circle
        let blur = clamp(abs(in.thickness) - 1.0, 0.001, 1.0);
        alpha = 1.0 - smoothstep(1.0 - blur, 1.0, d_norm);
    } else if (in.thickness < 0.0) {
        // Filled circle/ellipse
        alpha = 1.0 - smoothstep(-1.0, 1.0, dist);
    } else {
        // Ring outline
        let ring_dist = abs(dist);
        let half_t = in.thickness * 0.5;
        alpha = 1.0 - smoothstep(half_t - 1.0, half_t + 1.0, ring_dist);
    }
    
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
