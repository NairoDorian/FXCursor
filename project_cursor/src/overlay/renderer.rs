use std::time::Instant;
use wgpu::{
    BindGroup, Buffer, Device, Queue, RenderPipeline, TextureFormat,
};
use crate::config::{AppConfig, LayerConfig};

// =============================================================================
// VERTEX FORMATS & INSTANCES
// =============================================================================

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct TrailVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub tex: [f32; 2], // [blur, v]
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CircleInstance {
    pub center: [f32; 2],
    pub radius: [f32; 2],
    pub angle: f32,
    pub thickness: f32, // <0 filled, >=0 outline thickness
    pub color: [f32; 4],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayUniforms {
    screen_size: [f32; 2],
    time: f32,
    padding: f32,
}

// =============================================================================
// INTERNAL DATA STRUCTS
// =============================================================================

#[derive(Clone, Copy, Debug)]
struct TrailNode {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    speed: f32,
}

#[derive(Clone, Copy, Debug)]
struct Sample {
    x: f32,
    y: f32,
    nx: f32,
    ny: f32,
    speed: f32,
    progress: f32,
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
    size: f32,
    color: [f32; 4],
}

struct SquishyState {
    pos_x: f32,
    pos_y: f32,
    prev_x: f32,
    prev_y: f32,
    current_scale: f32,
    current_angle: f32,
    target_scale: f32,
    target_angle: f32,
}

struct SatelliteState {
    angle: f32,
    mirror_angle: f32,
}

// =============================================================================
// HELPER PRNG & MATH FUNCTIONS
// =============================================================================

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

fn apply_fade_curve(progress: f32, mode: u32) -> f32 {
    match mode {
        1 => 1.0 - progress * progress,                     // Ease-Out
        2 => (-progress * 3.0).exp(),                       // Exponential
        3 => 1.0 / (1.0 + (8.0 * (progress - 0.5)).exp()),  // Sigmoid
        _ => 1.0 - progress,                                // Linear
    }
}

fn catmull_rom(
    p0: (f32, f32),
    p1: (f32, f32),
    p2: (f32, f32),
    p3: (f32, f32),
    t: f32,
) -> (f32, f32) {
    let t2 = t * t;
    let t3 = t2 * t;
    
    let x = 0.5 * ((2.0 * p1.0) +
                   (-p0.0 + p2.0) * t +
                   (2.0 * p0.0 - 5.0 * p1.0 + 4.0 * p2.0 - p3.0) * t2 +
                   (-p0.0 + 3.0 * p1.0 - 3.0 * p2.0 + p3.0) * t3);
                   
    let y = 0.5 * ((2.0 * p1.1) +
                   (-p0.1 + p2.1) * t +
                   (2.0 * p0.1 - 5.0 * p1.1 + 4.0 * p2.1 - p3.1) * t2 +
                   (-p0.1 + 3.0 * p1.1 - 3.0 * p2.1 + p3.1) * t3);
                   
    (x, y)
}

// Static billboard corners for instanced SDF drawing
const QUAD_VERTICES: [[f32; 2]; 6] = [
    [-1.0, -1.0],
    [1.0, -1.0],
    [-1.0, 1.0],
    [-1.0, 1.0],
    [1.0, -1.0],
    [1.0, 1.0],
];

// =============================================================================
// OVERLAY RENDERER IMPLEMENTATION
// =============================================================================

pub struct OverlayRenderer {
    ribbon_pipeline: RenderPipeline,
    circle_pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    
    // WebGPU Vertex & Instance buffers
    ribbon_vertex_buffer: Buffer,
    circle_quad_vertex_buffer: Buffer,
    circle_instance_buffer: Buffer,
    quad_initialized: bool,

    // Kinematic / Entity State arrays
    trail_nodes: Vec<TrailNode>,
    samples: Vec<Sample>,
    ripples: Vec<Ripple>,
    particles: Vec<Particle>,
    squishy: SquishyState,
    satellites: Vec<SatelliteState>,
    
    // Physics pacing & transmission caches
    last_mouse_pos: (f32, f32),
    last_used_mouse_pos: (f32, f32),
    last_buttons: Vec<bool>,
    frame_counter: u32,
    max_recent_speed: f32,
    rainbow_hue: f32,
    seed: u32,
    
    start_time: Instant,
    last_update: Instant,
}

impl OverlayRenderer {
    pub fn new(
        device: &Device,
        format: TextureFormat,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("overlay shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        // 1. Uniforms
        let uniform_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay uniforms"),
            size: std::mem::size_of::<OverlayUniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("overlay bind group layout"),
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
            label: Some("overlay bind group"),
            layout: &bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
        });

        // 2. Pipelines Layout
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        // 3. Ribbon Pipeline
        let ribbon_vertex_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<TrailVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // position
                1 => Float32x4, // color
                2 => Float32x2, // tex: [blur, v]
            ],
        };

        let ribbon_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("ribbon pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_ribbon",
                buffers: &[ribbon_vertex_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_ribbon",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One, // Premultiplied alpha
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // 4. Circle / SDF Pipeline (with instanced spacing)
        let circle_quad_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![
                0 => Float32x2, // corner
            ],
        };

        let circle_instance_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<CircleInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &wgpu::vertex_attr_array![
                1 => Float32x2, // center
                2 => Float32x2, // radius
                3 => Float32,   // angle
                4 => Float32,   // thickness
                5 => Float32x4, // color
            ],
        };

        let circle_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("circle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_circle",
                buffers: &[circle_quad_layout, circle_instance_layout],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_circle",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One, // Premultiplied
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // 5. GPU Buffers
        let ribbon_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ribbon vertex buffer"),
            size: (60000 * std::mem::size_of::<TrailVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let circle_quad_vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("circle quad vertex buffer"),
            size: (std::mem::size_of::<[[f32; 2]; 6]>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let circle_instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("circle instance buffer"),
            size: (2048 * std::mem::size_of::<CircleInstance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            ribbon_pipeline,
            circle_pipeline,
            uniform_buffer,
            bind_group,
            
            ribbon_vertex_buffer,
            circle_quad_vertex_buffer,
            circle_instance_buffer,
            quad_initialized: false,

            trail_nodes: Vec::new(),
            samples: Vec::new(),
            ripples: Vec::new(),
            particles: Vec::new(),
            squishy: SquishyState {
                pos_x: 0.0,
                pos_y: 0.0,
                prev_x: 0.0,
                prev_y: 0.0,
                current_scale: 0.0,
                current_angle: 0.0,
                target_scale: 0.0,
                target_angle: 0.0,
            },
            satellites: Vec::new(),

            last_mouse_pos: (0.0, 0.0),
            last_used_mouse_pos: (0.0, 0.0),
            last_buttons: vec![false; 5],
            frame_counter: 0,
            max_recent_speed: 0.0,
            rainbow_hue: 0.0,
            seed: 0xACE1, // Custom LCG/PRNG non-zero seed

            start_time: Instant::now(),
            last_update: Instant::now(),
        }
    }

    /// Simulates physics for the cursor trail, click ripples, and particles.
    pub fn update_physics(
        &mut self,
        mouse_pos: (f32, f32),
        buttons: &[bool],
        config: &AppConfig,
    ) {
        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32().min(0.03); // Cap dt
        self.last_update = now;

        self.last_mouse_pos = mouse_pos;

        // --- 1. Track Click Events & Spawning ---
        let mut clicked_idx = None;
        for i in 0..5 {
            let pressed = buttons.get(i).copied().unwrap_or(false);
            let prev_pressed = self.last_buttons.get(i).copied().unwrap_or(false);
            if pressed && !prev_pressed {
                clicked_idx = Some(i);
                break;
            }
        }
        self.last_buttons = buttons.to_vec();

        if let Some(btn_idx) = clicked_idx {
            if config.click_response {
                let ripple_color = match btn_idx {
                    0 | 1 => config.ripple_left_color,
                    2 => config.ripple_right_color,
                    3 => config.ripple_middle_color,
                    _ => config.ripple_left_color,
                };

                // Spawn ripple
                self.ripples.push(Ripple {
                    x: mouse_pos.0,
                    y: mouse_pos.1,
                    time_elapsed: 0.0,
                    color: ripple_color,
                });

                // Spawn particles
                if config.particle_enabled {
                    let count = config.particle_count;
                    let speed = config.particle_speed;
                    for i in 0..count {
                        let angle = (i as f32) * 2.0 * std::f32::consts::PI / (count as f32);
                        let jitter_angle = angle + (next_random(&mut self.seed) - 0.5) * 0.5;
                        let jitter_speed = speed * (0.7 + next_random(&mut self.seed) * 0.6);

                        self.particles.push(Particle {
                            x: mouse_pos.0,
                            y: mouse_pos.1,
                            vx: jitter_angle.cos() * jitter_speed,
                            vy: jitter_angle.sin() * jitter_speed,
                            life: 1.0,
                            size: config.particle_size,
                            color: ripple_color,
                        });
                    }
                }
            }
        }

        // --- 2. Update Ripples ---
        for r in &mut self.ripples {
            r.time_elapsed += dt;
        }
        self.ripples.retain(|r| r.time_elapsed < config.ripple_duration);

        // --- 3. Update Particles ---
        let drag = (1.0 - config.particle_friction / 100.0).powf(dt * 120.0);
        for p in &mut self.particles {
            p.vy += config.particle_gravity * dt;
            p.vx *= drag;
            p.vy *= drag;
            p.x += p.vx * dt;
            p.y += p.vy * dt;
            p.life -= dt / config.particle_lifetime;
        }
        self.particles.retain(|p| p.life > 0.0);

        // --- 4. Update Trail Physics (Spring-Damper Chain) ---
        if config.enabled && config.effect_type == 0 {
            // Re-initialize or resize trail nodes if config length changed
            let length = config.trail_length as usize;
            if self.trail_nodes.len() != length {
                self.trail_nodes = vec![
                    TrailNode {
                        x: mouse_pos.0,
                        y: mouse_pos.1,
                        vx: 0.0,
                        vy: 0.0,
                        speed: 0.0,
                    };
                    length
                ];
            }

            if !self.trail_nodes.is_empty() {
                let dt_scale = dt / (1.0 / 120.0);

                let head_spring = config.head_spring / 1000.0;
                let head_fric_raw = 1.0 - (config.head_friction / 100.0);
                let head_fric = head_fric_raw.powf(dt_scale);

                let body_spring = config.body_spring / 1000.0;
                let body_fric_raw = 1.0 - (config.body_friction / 100.0);
                let body_fric = body_fric_raw.powf(dt_scale);

                // Position skip logic
                let skip_cycle = config.position_skip + 1;
                if self.frame_counter % skip_cycle == 0 {
                    self.last_used_mouse_pos = mouse_pos;
                }
                self.frame_counter += 1;

                // Head node follows target
                let head = &mut self.trail_nodes[0];
                head.vx += (self.last_used_mouse_pos.0 - head.x) * head_spring * dt_scale;
                head.vy += (self.last_used_mouse_pos.1 - head.y) * head_spring * dt_scale;
                head.vx *= head_fric;
                head.vy *= head_fric;
                head.x += head.vx * dt_scale;
                head.y += head.vy * dt_scale;
                head.speed = (head.vx * head.vx + head.vy * head.vy).sqrt();

                // Chain nodes follow predecessor
                for i in 1..length {
                    let (prev_slice, current_slice) = self.trail_nodes.split_at_mut(i);
                    let cur = &mut current_slice[0];
                    let prev = &prev_slice[i - 1];

                    if i > 1 {
                        let prev_prev = &prev_slice[i - 2];
                        cur.vx += (prev_prev.x - cur.x) * body_spring * 0.3 * dt_scale;
                        cur.vy += (prev_prev.y - cur.y) * body_spring * 0.3 * dt_scale;
                    }

                    cur.vx += (prev.x - cur.x) * body_spring * dt_scale;
                    cur.vy += (prev.y - cur.y) * body_spring * dt_scale;
                    cur.vx *= body_fric;
                    cur.vy *= body_fric;
                    cur.x += cur.vx * dt_scale;
                    cur.y += cur.vy * dt_scale;
                    cur.speed = (cur.vx * cur.vx + cur.vy * cur.vy).sqrt();
                }

                // Track max speed for adaptive quality
                self.max_recent_speed *= 0.95;
                for p in &self.trail_nodes {
                    self.max_recent_speed = self.max_recent_speed.max(p.speed);
                }
            }
        }

        // --- 5. Update Squishy Head ---
        if config.head_enabled {
            let smoothing = config.head_squish_smoothing / 100.0;
            let dt_scale = dt / (1.0 / 120.0);
            let adaptive_smoothing = 1.0 - (1.0 - smoothing).powf(dt_scale);

            self.squishy.pos_x += (mouse_pos.0 - self.squishy.pos_x) * adaptive_smoothing;
            self.squishy.pos_y += (mouse_pos.1 - self.squishy.pos_y) * adaptive_smoothing;

            let dx = self.squishy.pos_x - self.squishy.prev_x;
            let dy = self.squishy.pos_y - self.squishy.prev_y;
            let velocity = (dx * dx + dy * dy).sqrt();
            self.squishy.prev_x = self.squishy.pos_x;
            self.squishy.prev_y = self.squishy.pos_y;

            let intensity = config.head_squish_intensity / 100.0;
            let amplified = (velocity * 8.0).min(200.0);
            self.squishy.target_scale = (amplified / 15.0) * intensity;
            self.squishy.current_scale += (self.squishy.target_scale - self.squishy.current_scale) * adaptive_smoothing;

            if velocity > 0.5 {
                self.squishy.target_angle = dy.atan2(dx);
            }
            let mut angle_diff = self.squishy.target_angle - self.squishy.current_angle;
            while angle_diff > std::f32::consts::PI {
                angle_diff -= 2.0 * std::f32::consts::PI;
            }
            while angle_diff < -std::f32::consts::PI {
                angle_diff += 2.0 * std::f32::consts::PI;
            }
            self.squishy.current_angle += angle_diff * adaptive_smoothing;
        }

        // --- 6. Update Satellites ---
        if config.satellite_enabled {
            let dt_scale = dt / (1.0 / 120.0);
            let base_speed = config.satellite_speed.to_radians() * dt_scale;
            let dual_speed = -config.satellite_dual_speed.to_radians() * dt_scale;

            if self.satellites.len() != config.satellite_count as usize {
                self.satellites.clear();
                let inc = 2.0 * std::f32::consts::PI / config.satellite_count as f32;
                for i in 0..config.satellite_count {
                    self.satellites.push(SatelliteState {
                        angle: (i as f32) * inc,
                        mirror_angle: (i as f32) * inc,
                    });
                }
            }

            for s in &mut self.satellites {
                s.angle = (s.angle + base_speed) % (2.0 * std::f32::consts::PI);
                if s.angle < 0.0 {
                    s.angle += 2.0 * std::f32::consts::PI;
                }

                s.mirror_angle = (s.mirror_angle + dual_speed) % (2.0 * std::f32::consts::PI);
                if s.mirror_angle < 0.0 {
                    s.mirror_angle += 2.0 * std::f32::consts::PI;
                }
            }
        }
    }

    /// Helper function to build spline-interpolated samples along the physics chain
    fn build_samples(&mut self, config: &AppConfig) {
        self.samples.clear();
        if self.trail_nodes.len() < 2 {
            return;
        }

        let n = self.trail_nodes.len();
        
        let mut steps = config.interpolation_steps;
        if config.adaptive_quality {
            let speed_factor = ((self.max_recent_speed - 15.0) / 50.0).clamp(0.0, 1.0);
            steps = (config.interpolation_steps as f32 * (1.0 - speed_factor * 0.8)) as u32;
            steps = steps.max(1);
        }

        self.samples.reserve((n - 1) * steps as usize + 1);

        let get_node = |idx: isize| -> &TrailNode {
            &self.trail_nodes[idx.clamp(0, n as isize - 1) as usize]
        };

        for i in 0..n - 1 {
            let p0 = get_node(i as isize - 1);
            let p1 = get_node(i as isize);
            let p2 = get_node(i as isize + 1);
            let p3 = get_node(i as isize + 2);

            for s in 0..steps {
                let t = s as f32 / steps as f32;
                let (x, y) = catmull_rom((p0.x, p0.y), (p1.x, p1.y), (p2.x, p2.y), (p3.x, p3.y), t);
                let speed = p1.speed * (1.0 - t) + p2.speed * t;
                let progress = (i as f32 + t) / (n - 1) as f32;

                self.samples.push(Sample {
                    x,
                    y,
                    nx: 0.0,
                    ny: 1.0,
                    speed,
                    progress,
                });
            }
        }

        // Emit final node sample
        let last = &self.trail_nodes[n - 1];
        self.samples.push(Sample {
            x: last.x,
            y: last.y,
            nx: 0.0,
            ny: 1.0,
            speed: last.speed,
            progress: 1.0,
        });

        // Stabilise ribbon normal vectors
        let m = self.samples.len();
        let mut prev_nx = 0.0;
        let mut prev_ny = 1.0;
        for i in 0..m {
            let (ax, ay, bx, by) = if i == 0 {
                (self.samples[0].x, self.samples[0].y, self.samples[1.min(m - 1)].x, self.samples[1.min(m - 1)].y)
            } else if i == m - 1 {
                (self.samples[i - 1].x, self.samples[i - 1].y, self.samples[i].x, self.samples[i].y)
            } else {
                (self.samples[i - 1].x, self.samples[i - 1].y, self.samples[i + 1].x, self.samples[i + 1].y)
            };

            let dx = bx - ax;
            let dy = by - ay;
            let len = (dx * dx + dy * dy).sqrt();

            if len < 0.1 {
                self.samples[i].nx = prev_nx;
                self.samples[i].ny = prev_ny;
            } else {
                let mut nx = -dy / len;
                let mut ny = dx / len;

                // Stabilisation pass (dot product normal stabilizer)
                if nx * prev_nx + ny * prev_ny < 0.0 {
                    nx = -nx;
                    ny = -ny;
                }

                self.samples[i].nx = nx;
                self.samples[i].ny = ny;
                prev_nx = nx;
                prev_ny = ny;
            }
        }
    }

    /// Assembles vertex data for one ribbon layer
    fn build_layer_vertices(
        &self,
        dst: &mut Vec<TrailVertex>,
        layer: &LayerConfig,
        config: &AppConfig,
    ) {
        if !layer.enabled || self.samples.len() < 2 {
            return;
        }

        let num_samples = self.samples.len();
        let min_w = config.min_trail_width;
        
        let start_c = if config.rainbow_mode {
            hsl_to_rgba(self.rainbow_hue, 1.0, 0.5)
        } else {
            layer.start_color
        };
        let end_c = if config.rainbow_mode {
            hsl_to_rgba(self.rainbow_hue + 180.0, 1.0, 0.3)
        } else if config.enable_gradient {
            layer.end_color
        } else {
            start_c
        };

        let mut left_points = Vec::with_capacity(num_samples);
        let mut right_points = Vec::with_capacity(num_samples);
        let mut colors = Vec::with_capacity(num_samples);
        let mut blurs = Vec::with_capacity(num_samples);

        for s in &self.samples {
            let fade = apply_fade_curve(s.progress, config.fade_mode);
            let current_blur = layer.start_blur + (layer.end_blur - layer.start_blur) * s.progress;
            let norm_speed = (s.speed / 20.0).min(1.0);
            
            let vel_width = 1.0 + norm_speed * config.velocity_width_multiplier;
            let vel_alpha = 1.0 + norm_speed * config.velocity_alpha_multiplier;

            let w = (config.trail_width * layer.width_factor * fade * vel_width).max(min_w);
            let half_w = w * 0.5;

            let mut c = if config.enable_gradient || config.rainbow_mode {
                lerp_rgba(start_c, end_c, s.progress)
            } else {
                start_c
            };
            // Premultiply color alpha channel
            let alpha = c[3] * layer.alpha_factor * fade * vel_alpha;
            c[0] *= alpha;
            c[1] *= alpha;
            c[2] *= alpha;
            c[3] = alpha;

            colors.push(c);
            blurs.push(current_blur);
            
            left_points.push([s.x + s.nx * half_w, s.y + s.ny * half_w]);
            right_points.push([s.x - s.nx * half_w, s.y - s.ny * half_w]);
        }

        // --- 1. ROUND HEAD CAP ---
        let s0 = &self.samples[0];
        let angle = s0.ny.atan2(s0.nx);
        let r0 = (config.trail_width * layer.width_factor * apply_fade_curve(s0.progress, config.fade_mode) * (1.0 + (s0.speed / 20.0).min(1.0) * config.velocity_width_multiplier)).max(min_w) * 0.5;
        let k_cap_steps = 16;

        for j in 0..k_cap_steps {
            let theta1 = angle + (j as f32) * 2.0 * std::f32::consts::PI / (k_cap_steps as f32);
            let theta2 = angle + ((j + 1) as f32) * 2.0 * std::f32::consts::PI / (k_cap_steps as f32);

            dst.push(TrailVertex {
                position: [s0.x, s0.y],
                color: colors[0],
                tex: [blurs[0], 0.0],
            });
            dst.push(TrailVertex {
                position: [s0.x + theta1.cos() * r0, s0.y + theta1.sin() * r0],
                color: colors[0],
                tex: [blurs[0], 1.0],
            });
            dst.push(TrailVertex {
                position: [s0.x + theta2.cos() * r0, s0.y + theta2.sin() * r0],
                color: colors[0],
                tex: [blurs[0], 1.0],
            });
        }

        // --- 2. MAIN RIBBON GEOMETRY ---
        for i in 0..num_samples - 1 {
            let a = left_points[i];
            let b = right_points[i];
            let c = left_points[i + 1];
            let d = right_points[i + 1];

            let col_i = colors[i];
            let col_next = colors[i + 1];
            let blur_i = blurs[i];
            let blur_next = blurs[i + 1];

            // Triangle 1: A, B, C
            dst.push(TrailVertex { position: a, color: col_i, tex: [blur_i, 1.0] });
            dst.push(TrailVertex { position: b, color: col_i, tex: [blur_i, -1.0] });
            dst.push(TrailVertex { position: c, color: col_next, tex: [blur_next, 1.0] });

            // Triangle 2: B, D, C
            dst.push(TrailVertex { position: b, color: col_i, tex: [blur_i, -1.0] });
            dst.push(TrailVertex { position: d, color: col_next, tex: [blur_next, -1.0] });
            dst.push(TrailVertex { position: c, color: col_next, tex: [blur_next, 1.0] });
        }

        // --- 3. ROUND TAIL CAP ---
        let sn = &self.samples[num_samples - 1];
        let angle_n = sn.ny.atan2(sn.nx);
        let rn = (config.trail_width * layer.width_factor * apply_fade_curve(sn.progress, config.fade_mode) * (1.0 + (sn.speed / 20.0).min(1.0) * config.velocity_width_multiplier)).max(min_w) * 0.5;

        for j in 0..k_cap_steps {
            let theta1 = angle_n + (j as f32) * 2.0 * std::f32::consts::PI / (k_cap_steps as f32);
            let theta2 = angle_n + ((j + 1) as f32) * 2.0 * std::f32::consts::PI / (k_cap_steps as f32);

            dst.push(TrailVertex {
                position: [sn.x, sn.y],
                color: colors[num_samples - 1],
                tex: [blurs[num_samples - 1], 0.0],
            });
            dst.push(TrailVertex {
                position: [sn.x + theta1.cos() * rn, sn.y + theta1.sin() * rn],
                color: colors[num_samples - 1],
                tex: [blurs[num_samples - 1], 1.0],
            });
            dst.push(TrailVertex {
                position: [sn.x + theta2.cos() * rn, sn.y + theta2.sin() * rn],
                color: colors[num_samples - 1],
                tex: [blurs[num_samples - 1], 1.0],
            });
        }
    }

    /// Renders the overlay into the given surface view.
    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        view: &wgpu::TextureView,
        screen_width: u32,
        screen_height: u32,
        config: &AppConfig,
    ) {
        // --- 1. Handle PRNG & Rainbow state updates ---
        if config.rainbow_mode {
            self.rainbow_hue = (self.rainbow_hue + config.rainbow_speed) % 360.0;
        }

        // --- 2. Initialize static quad once if needed ---
        if !self.quad_initialized {
            queue.write_buffer(&self.circle_quad_vertex_buffer, 0, bytemuck::cast_slice(&QUAD_VERTICES));
            self.quad_initialized = true;
        }

        // --- 3. Write Uniforms ---
        let uniforms = OverlayUniforms {
            screen_size: [screen_width as f32, screen_height as f32],
            time: self.start_time.elapsed().as_secs_f32(),
            padding: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // --- 4. Build and upload Ribbon vertices ---
        let mut ribbon_vertices = Vec::new();
        if config.enabled && config.effect_type == 0 {
            self.build_samples(config);
            for layer_idx in 0..4 {
                self.build_layer_vertices(&mut ribbon_vertices, &config.layers[layer_idx], config);
            }
        }

        let num_ribbon_verts = ribbon_vertices.len().min(60000);
        if num_ribbon_verts > 0 {
            queue.write_buffer(&self.ribbon_vertex_buffer, 0, bytemuck::cast_slice(&ribbon_vertices[..num_ribbon_verts]));
        }

        // --- 5. Assemble Circle instances ---
        let mut circle_instances = Vec::new();

        // 5a. Click Ripples (Rings)
        for r in &self.ripples {
            let progress = (r.time_elapsed / config.ripple_duration).clamp(0.0, 1.0);
            let fade = apply_fade_curve(progress, config.fade_mode);
            let progress_ease = 1.0 - (1.0 - progress).powi(3);
            let diameter = config.ripple_radius * progress_ease;
            let width = config.ripple_start_width * fade;
            
            if diameter > 0.1 && width >= 0.5 {
                let mut c = r.color;
                // Premultiply alphas
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

        // 5b. Squishy Cursor Head (Ellipse)
        if config.head_enabled && config.enabled && config.effect_type == 0 {
            let scale_x = 1.0 + self.squishy.current_scale;
            let scale_y = (1.0 - self.squishy.current_scale * 0.5).max(0.3);
            let base_radius = config.head_size * 0.5;

            let mut c = config.head_color;
            c[0] *= c[3];
            c[1] *= c[3];
            c[2] *= c[3];

            circle_instances.push(CircleInstance {
                center: [self.squishy.pos_x, self.squishy.pos_y],
                radius: [base_radius * scale_x, base_radius * scale_y],
                angle: self.squishy.current_angle,
                thickness: if config.head_filled { -1.0 } else { config.head_outline_width },
                color: c,
            });
        }

        // 5c. Satellite Orbitals
        if config.satellite_enabled {
            let orbit_radius = config.satellite_orbit_diameter * 0.5;
            let sat_radius = config.satellite_size * 0.5;
            let origin = self.last_mouse_pos;

            let mut c = config.satellite_color;
            c[0] *= c[3];
            c[1] *= c[3];
            c[2] *= c[3];

            // Background orbit ring
            if config.satellite_show_orbit_ring {
                let mut rc = config.satellite_ring_color;
                rc[0] *= rc[3];
                rc[1] *= rc[3];
                rc[2] *= rc[3];

                circle_instances.push(CircleInstance {
                    center: [origin.0, origin.1],
                    radius: [orbit_radius, orbit_radius],
                    angle: 0.0,
                    thickness: config.satellite_ring_width,
                    color: rc,
                });
            }

            // Primary orbitals
            for s in &self.satellites {
                let x = origin.0 + orbit_radius * s.angle.cos();
                let y = origin.1 + orbit_radius * s.angle.sin();

                circle_instances.push(CircleInstance {
                    center: [x, y],
                    radius: [sat_radius, sat_radius],
                    angle: 0.0,
                    thickness: if config.satellite_filled { -1.0 } else { config.satellite_outline_width },
                    color: c,
                });
            }

            // Mirrored orbitals
            if config.satellite_enable_dual_ring {
                for s in &self.satellites {
                    let x = origin.0 + orbit_radius * s.mirror_angle.cos();
                    let y = origin.1 + orbit_radius * s.mirror_angle.sin();

                    circle_instances.push(CircleInstance {
                        center: [x, y],
                        radius: [sat_radius, sat_radius],
                        angle: 0.0,
                        thickness: if config.satellite_filled { -1.0 } else { config.satellite_outline_width },
                        color: c,
                    });
                }
            }
        }

        // 5d. Click Particles (Soft circles)
        for p in &self.particles {
            let mut c = p.color;
            // Premultiply
            c[0] *= p.life;
            c[1] *= p.life;
            c[2] *= p.life;
            c[3] = p.life;

            let size = p.size * p.life;

            circle_instances.push(CircleInstance {
                center: [p.x, p.y],
                radius: [size * 0.5, size * 0.5],
                angle: 0.0,
                thickness: -1.0, // Filled
                color: c,
            });
        }

        // 5e. Glow Aura (Glow mode)
        if config.enabled && config.effect_type == 2 {
            let radius = config.trail_width * 2.0; // Glow radius
            let mut c = config.trail_color;
            c[0] *= c[3];
            c[1] *= c[3];
            c[2] *= c[3];

            circle_instances.push(CircleInstance {
                center: [self.last_mouse_pos.0, self.last_mouse_pos.1],
                radius: [radius, radius],
                angle: 0.0,
                thickness: -2.0, // Special sentinel in fragment shader or just draw filled circle
                color: c,
            });
        }

        let num_circle_instances = circle_instances.len().min(2048);
        if num_circle_instances > 0 {
            queue.write_buffer(&self.circle_instance_buffer, 0, bytemuck::cast_slice(&circle_instances[..num_circle_instances]));
        }

        // --- 6. Record and submit rendering commands ---
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("overlay command encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }), // Clear transparent
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // Set uniform bind group
            rpass.set_bind_group(0, &self.bind_group, &[]);

            // Draw Ribbon Trail
            if num_ribbon_verts > 0 {
                rpass.set_pipeline(&self.ribbon_pipeline);
                rpass.set_vertex_buffer(0, self.ribbon_vertex_buffer.slice(..));
                rpass.draw(0..num_ribbon_verts as u32, 0..1);
            }

            // Draw SDF Circles & Rings (Instanced)
            if num_circle_instances > 0 {
                rpass.set_pipeline(&self.circle_pipeline);
                rpass.set_vertex_buffer(0, self.circle_quad_vertex_buffer.slice(..));
                rpass.set_vertex_buffer(1, self.circle_instance_buffer.slice(..));
                rpass.draw(0..6, 0..num_circle_instances as u32);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
