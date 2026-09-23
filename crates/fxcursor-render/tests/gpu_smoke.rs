//! Headless GPU smoke test: builds every pipeline, binds a GPU-cursor texture and renders the
//! full effect stack into an offscreen target. wgpu's default error handler panics on any
//! validation error, so this catches pipeline / binding / usage mistakes that the GPU-free unit
//! tests cannot (e.g. a cursor texture created without `TEXTURE_BINDING`).
//!
//! Skips (passes) when the machine has no adapter at all; Windows CI usually still has WARP.

use fxcursor_protocol::{AppConfig, EffectMode};
use fxcursor_render::{CursorShape, OverlayRenderer};

const SIZE: u32 = 256;
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Bgra8Unorm;

fn device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter =
        pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default())).ok()
}

#[test]
fn renders_every_effect_and_the_gpu_cursor_offscreen() {
    let Some((device, queue)) = device() else {
        eprintln!("no GPU adapter available; skipping the smoke test");
        return;
    };

    let mut config = AppConfig {
        effect_mode: EffectMode::FourLayerGlow,
        ..Default::default()
    };
    config.head.enabled = true;
    config.ripple.enabled = true;
    config.particles.enabled = true;
    config.satellites.enabled = true;
    config.gpu_cursor.enabled = true;
    config.fps_counter.enabled = true;

    let mut renderer = OverlayRenderer::new(&device, FORMAT);
    // 4×4 opaque white arrow stand-in, hotspot at the top-left.
    let shape = CursorShape {
        width: 4,
        height: 4,
        hotspot: (0, 0),
        is_arrow: true,
        source_key: 1,
        pixels: vec![255; 4 * 4 * 4],
    };
    assert!(
        renderer.set_cursor_shape(&device, &queue, &shape),
        "shape uploaded"
    );

    let target = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("smoke_target"),
        size: wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = target.create_view(&wgpu::TextureViewDescriptor::default());

    // Sweep the pointer across the target, click once, render every frame.
    for frame in 0..30 {
        let x = 40.0 + frame as f32 * 6.0;
        let y = 128.0 + (frame as f32 * 0.4).sin() * 40.0;
        if frame == 10 {
            renderer.spawn_click(0, x, y, &config);
        }
        renderer.update_mouse(x, y, 1.0 / 60.0, &config);
        renderer.render(&device, &queue, &view, SIZE, SIZE, &config);
    }
    let (capsules, billboards) = renderer.last_counts;
    assert!(capsules > 0, "the ribbon produced capsules");
    assert!(
        billboards > 0,
        "head / ripple / particles / satellites produced billboards"
    );

    // Read back and make sure something non-transparent landed on the target.
    let bytes_per_row = SIZE * 4; // 1024: already a multiple of COPY_BYTES_PER_ROW_ALIGNMENT
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("smoke_readback"),
        size: u64::from(bytes_per_row * SIZE),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
    encoder.copy_texture_to_buffer(
        target.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width: SIZE,
            height: SIZE,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(Some(encoder.finish()));
    buffer
        .slice(..)
        .map_async(wgpu::MapMode::Read, |r| r.expect("map readback"));
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .expect("poll");
    let data = buffer.slice(..).get_mapped_range().expect("mapped range");
    let covered = data.as_chunks::<4>().0.iter().filter(|px| px[3] > 0).count();
    assert!(covered > 500, "only {covered} pixels were drawn");
}
