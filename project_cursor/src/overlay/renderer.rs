use std::time::Instant;
use wgpu::{
    BindGroup, Buffer, Device, Queue, RenderPipeline, TextureFormat,
};
use crate::config::AppConfig;


#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ParticleVertex {
    pub position: [f32; 2],
    pub color: [f32; 4],
    pub size: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct OverlayUniforms {
    color: [f32; 4],
    screen_size: [f32; 2],
    mouse_pos: [f32; 2],
    time: f32,
    ripple_radius: f32,
    effect_type: u32,
    padding: f32,
}

struct Particle {
    x: f32,
    y: f32,
    vx: f32,
    vy: f32,
    life: f32, // 1.0 down to 0.0
    size: f32,
}

pub struct OverlayRenderer {
    pipeline: RenderPipeline,
    uniform_buffer: Buffer,
    bind_group: BindGroup,
    vertex_buffer: Buffer,
    max_particles: usize,
    
    // Physics / Particle State
    particles: Vec<Particle>,
    ripples: Vec<Particle>,
    last_mouse_pos: (f32, f32),
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

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("overlay pipeline layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("overlay pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ParticleVertex>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &wgpu::vertex_attr_array![
                        0 => Float32x2, // position
                        1 => Float32x4, // color
                        2 => Float32,   // size
                    ],
                }],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    // Enable full alpha blending for transparent overlay windows
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::SrcAlpha,
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
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let max_particles = 1024;
        let vertex_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("overlay vertex buffer"),
            size: (max_particles * 6 * std::mem::size_of::<ParticleVertex>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            uniform_buffer,
            bind_group,
            vertex_buffer,
            max_particles,
            particles: Vec::new(),
            ripples: Vec::new(),
            last_mouse_pos: (0.0, 0.0),
            start_time: Instant::now(),
            last_update: Instant::now(),
        }
    }

    /// Simulates physics for the cursor trail and clicks.
    pub fn update_physics(
        &mut self,
        mouse_pos: (f32, f32),
        click_pressed: bool,
        config: &AppConfig,
    ) {
        let now = Instant::now();
        let dt = (now - self.last_update).as_secs_f32().min(0.03); // Cap dt to prevent huge skips
        self.last_update = now;

        self.last_mouse_pos = mouse_pos;

        // --- 1. Spawn Ripple on Click ---
        if config.click_response && click_pressed && self.ripples.is_empty() {
            self.ripples.push(Particle {
                x: mouse_pos.0,
                y: mouse_pos.1,
                vx: 0.0,
                vy: 0.0,
                life: 1.0,
                size: 1.0,
            });
        }

        // Update Ripples
        for ripple in &mut self.ripples {
            // Ripples expand and fade
            ripple.size += config.ripple_radius * dt * 2.0; // expand speed
            ripple.life -= dt * 1.5; // decay
        }
        self.ripples.retain(|r| r.life > 0.0);

        // --- 2. Particle Trail Simulation ---
        if config.enabled {
            if config.effect_type == 0 {
                // Spawn new particle at mouse position
                if self.particles.len() < config.trail_length as usize {
                    // Let's spawn new particles
                    self.particles.push(Particle {
                        x: mouse_pos.0,
                        y: mouse_pos.1,
                        vx: 0.0,
                        vy: 0.0,
                        life: 1.0,
                        size: config.trail_width,
                    });
                }

                // Update particles as a ribbon chain
                if !self.particles.is_empty() {
                    // Head follows mouse
                    self.particles[0].x = mouse_pos.0;
                    self.particles[0].y = mouse_pos.1;
                    self.particles[0].life = 1.0;
                    self.particles[0].size = config.trail_width;

                    // Rest of the chain follows previous element with easing
                    for i in 1..self.particles.len() {
                        let target_x = self.particles[i - 1].x;
                        let target_y = self.particles[i - 1].y;
                        
                        let dx = target_x - self.particles[i].x;
                        let dy = target_y - self.particles[i].y;

                        // Smooth interpolation driven by config.speed and friction
                        let speed_factor = config.speed * 10.0 * dt;
                        self.particles[i].vx += dx * speed_factor;
                        self.particles[i].vy += dy * speed_factor + config.gravity * dt * 50.0;

                        // Apply friction
                        self.particles[i].vx *= 1.0 - config.friction;
                        self.particles[i].vy *= 1.0 - config.friction;

                        // Apply velocity
                        self.particles[i].x += self.particles[i].vx;
                        self.particles[i].y += self.particles[i].vy;

                        // Scale size and life down the chain
                        let progress = i as f32 / config.trail_length as f32;
                        self.particles[i].life = 1.0 - progress;
                        self.particles[i].size = config.trail_width * (1.0 - progress * 0.7);
                    }
                }
            } else if config.effect_type == 2 {
                // Glow Mode - single large particle centered at cursor
                self.particles.clear();
                self.particles.push(Particle {
                    x: mouse_pos.0,
                    y: mouse_pos.1,
                    vx: 0.0,
                    vy: 0.0,
                    life: 1.0,
                    size: config.trail_width * 4.0,
                });
            } else {
                // Ripple Mode (only render active click ripples)
                self.particles.clear();
            }
        } else {
            self.particles.clear();
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
        // Prepare Uniforms
        let uniforms = OverlayUniforms {
            color: config.trail_color,
            screen_size: [screen_width as f32, screen_height as f32],
            mouse_pos: [self.last_mouse_pos.0, self.last_mouse_pos.1],
            time: self.start_time.elapsed().as_secs_f32(),
            ripple_radius: config.ripple_radius,
            effect_type: config.effect_type,
            padding: 0.0,
        };
        queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&uniforms));

        // Assemble Vertex Data
        let mut vertices = Vec::new();

        // 1. Draw trail particles (or glow particle)
        if config.effect_type != 1 {
            for particle in &self.particles {
                let p_color = [
                    config.trail_color[0],
                    config.trail_color[1],
                    config.trail_color[2],
                    config.trail_color[3] * particle.life,
                ];

                // Append 6 vertices (billboard quad)
                for _ in 0..6 {
                    vertices.push(ParticleVertex {
                        position: [particle.x, particle.y],
                        color: p_color,
                        size: particle.size,
                    });
                }
            }
        }

        // 2. Draw active click ripples
        for ripple in &self.ripples {
            let r_color = [
                config.trail_color[0],
                config.trail_color[1],
                config.trail_color[2],
                config.trail_color[3] * ripple.life,
            ];

            // Render ripple as a large quad
            for _ in 0..6 {
                vertices.push(ParticleVertex {
                    position: [ripple.x, ripple.y],
                    color: r_color,
                    size: ripple.size,
                });
            }
        }

        // Cap to buffer size limit
        let num_vertices = vertices.len().min(self.max_particles * 6);
        if num_vertices > 0 {
            queue.write_buffer(&self.vertex_buffer, 0, bytemuck::cast_slice(&vertices[..num_vertices]));
        }

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("overlay encoder"),
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("overlay render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        // Crucial: Clear surface to fully transparent color
                        load: wgpu::LoadOp::Clear(wgpu::Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if num_vertices > 0 {
                rpass.set_pipeline(&self.pipeline);
                rpass.set_bind_group(0, &self.bind_group, &[]);
                rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
                rpass.draw(0..num_vertices as u32, 0..1);
            }
        }

        queue.submit(std::iter::once(encoder.finish()));
    }
}
