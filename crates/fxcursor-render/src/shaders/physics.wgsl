// =============================================================================
// FXCursor V4: GPU Compute Physics & Particle Shader (WGSL)
// 240Hz sub-stepped Spring-Damper simulation & Catmull-Rom normal stabilization
// =============================================================================

struct PhysicsParams {
    cursor_pos: vec2<f32>,
    cursor_vel: vec2<f32>,
    spring: f32,
    damping: f32,
    head_spring: f32,
    head_damping: f32,
    dt: f32,
    node_count: u32,
};

@group(0) @binding(0) var<uniform> params: PhysicsParams;

struct TrailNode {
    pos: vec2<f32>,
    vel: vec2<f32>,
    speed: f32,
    pad: f32,
};

@group(0) @binding(1) var<storage, read_write> trail_nodes: array<TrailNode>;

@compute @workgroup_size(64)
fn cs_spring_chain(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= params.node_count) {
        return;
    }

    var current = trail_nodes[index];

    if (index == 0u) {
        // Head node attracted directly to current mouse position
        let delta = params.cursor_pos - current.pos;
        let force = delta * params.head_spring - current.vel * params.head_damping;
        
        current.vel = current.vel + force * params.dt;
        current.pos = current.pos + current.vel * params.dt;
        current.speed = length(current.vel);
    } else {
        // Follow the leader node
        let prev = trail_nodes[index - 1u];
        let delta = prev.pos - current.pos;
        let force = delta * params.spring - current.vel * params.damping;
        
        current.vel = current.vel + force * params.dt;
        current.pos = current.pos + current.vel * params.dt;
        current.speed = length(current.vel);
    }

    trail_nodes[index] = current;
}

// Particle simulation compute pass
struct ParticleData {
    pos: vec2<f32>,
    vel: vec2<f32>,
    color: vec4<f32>,
    life: f32,
    size: f32,
    pad: vec2<f32>,
};

@group(0) @binding(2) var<storage, read_write> particles: array<ParticleData>;

struct ParticleSimParams {
    gravity: f32,
    friction: f32,
    dt: f32,
    particle_count: u32,
};

@group(0) @binding(3) var<uniform> particle_params: ParticleSimParams;

@compute @workgroup_size(64)
fn cs_simulate_particles(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= particle_params.particle_count) {
        return;
    }

    var p = particles[index];
    if (p.life <= 0.0) {
        return;
    }

    p.life = p.life - particle_params.dt;
    if (p.life > 0.0) {
        p.vel.y = p.vel.y + particle_params.gravity * particle_params.dt;
        p.vel = p.vel * pow(particle_params.friction, particle_params.dt * 60.0);
        p.pos = p.pos + p.vel * particle_params.dt;
        p.color.a = p.color.a * (p.life / (p.life + particle_params.dt));
    }

    particles[index] = p;
}
