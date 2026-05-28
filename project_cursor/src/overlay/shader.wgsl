struct Uniforms {
    color: vec4<f32>,
    screen_size: vec2<f32>,
    mouse_pos: vec2<f32>,
    time: f32,
    ripple_radius: f32,
    effect_type: u32,
    padding: f32,
};

@group(0) @binding(0) var<uniform> uniforms: Uniforms;

struct VertexInput {
    @location(0) position: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) size: f32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

fn get_quad_offset(index: u32) -> vec2<f32> {
    switch (index % 6u) {
        case 0u: { return vec2<f32>(-1.0, -1.0); }
        case 1u: { return vec2<f32>(1.0, -1.0); }
        case 2u: { return vec2<f32>(-1.0, 1.0); }
        case 3u: { return vec2<f32>(-1.0, 1.0); }
        case 4u: { return vec2<f32>(1.0, -1.0); }
        default: { return vec2<f32>(1.0, 1.0); }
    }
}

@vertex
fn vs_main(
    model: VertexInput,
    @builtin(vertex_index) in_vertex_index: u32,
) -> VertexOutput {
    var out: VertexOutput;
    
    let uv_offset = get_quad_offset(in_vertex_index);
    let offset = uv_offset * model.size;
    let world_pos = model.position + offset;
    
    // Map to normalized device coordinates (NDC)
    let clip_x = (world_pos.x / uniforms.screen_size.x) * 2.0 - 1.0;
    let clip_y = 1.0 - (world_pos.y / uniforms.screen_size.y) * 2.0; // Y is inverted in screen coordinates vs NDC
    
    out.clip_position = vec4<f32>(clip_x, clip_y, 0.0, 1.0);
    out.color = model.color;
    out.uv = uv_offset;
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist_sq = in.uv.x * in.uv.x + in.uv.y * in.uv.y;
    
    if (dist_sq > 1.0) {
        discard;
    }
    
    if (uniforms.effect_type == 0u) {
        // Particle trail: soft circle mask
        let alpha = 1.0 - smoothstep(0.3, 1.0, dist_sq);
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    } else if (uniforms.effect_type == 1u) {
        // Ripple: Ring shape
        let dist = sqrt(dist_sq);
        // Fade out at the center and edges
        let ring = smoothstep(0.8, 0.9, dist) * (1.0 - smoothstep(0.9, 1.0, dist));
        return vec4<f32>(in.color.rgb, in.color.a * ring);
    } else {
        // Glow aura: Soft radial glow
        let alpha = exp(-dist_sq * 4.0);
        return vec4<f32>(in.color.rgb, in.color.a * alpha);
    }
}
