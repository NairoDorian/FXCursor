//! Overlay snapshot: renders the current frame into an offscreen texture, reads it back and
//! writes a PNG. Used by the `capture_overlay` IPC command, the `--capture <file>` CLI flag and
//! the Developer Hub, and it is the only reliable way to see the overlay: GDI screen capture
//! does not include the flip-model/Vulkan swapchain.

use fxcursor_protocol::AppConfig;
use fxcursor_render::OverlayRenderer;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

/// A pending snapshot, or a burst of them; fulfilled by the render thread.
///
/// A burst (`frames > 1`) writes `<stem>_<NN>.<ext>` every `interval` — a flip-book of a
/// transient such as a stop or a reversal, which a single snapshot can never catch reliably
/// because the request itself arrives with unpredictable latency.
pub struct CaptureRequest {
    pub path: PathBuf,
    /// Requested crop size in overlay pixels, centred on the cursor. `None` = whole overlay.
    pub size: Option<(u32, u32)>,
    /// Frames still to capture (at least 1).
    pub frames: u32,
    pub interval: Duration,
    /// Index of the next frame; a request created with a single frame never writes a suffix.
    pub index: u32,
    pub total: u32,
    /// The render thread skips the request until this instant (burst pacing).
    pub not_before: Option<Instant>,
    /// Files written so far, reported to the caller when the request completes.
    pub written: Vec<String>,
    pub reply: Sender<Result<String, String>>,
}

impl CaptureRequest {
    pub fn new(
        path: PathBuf,
        size: Option<(u32, u32)>,
        frames: u32,
        interval: Duration,
        reply: Sender<Result<String, String>>,
    ) -> Self {
        let frames = frames.max(1);
        Self {
            path,
            size,
            frames,
            interval,
            index: 0,
            total: frames,
            not_before: None,
            written: Vec::new(),
            reply,
        }
    }

    pub fn is_due(&self, now: Instant) -> bool {
        self.not_before.is_none_or(|t| now >= t)
    }

    /// Output file for the next frame.
    pub fn frame_path(&self) -> PathBuf {
        if self.total <= 1 {
            self.path.clone()
        } else {
            burst_frame_path(&self.path, self.index)
        }
    }

    /// Records a written frame. Returns `true` when the request is complete.
    pub fn frame_done(&mut self, saved: String, now: Instant) -> bool {
        self.written.push(saved);
        self.index += 1;
        self.frames -= 1;
        self.not_before = Some(now + self.interval);
        self.frames == 0
    }

    /// What the caller gets back: the file for a single snapshot, a summary for a burst.
    pub fn summary(&self) -> String {
        match self.written.as_slice() {
            [single] => single.clone(),
            [first, .., last] => format!("{} frames: {first} … {last}", self.written.len()),
            [] => String::new(),
        }
    }

    /// How long a caller should wait for this request: the base timeout plus the burst length.
    pub fn timeout(&self) -> Duration {
        CAPTURE_TIMEOUT + self.interval * self.total
    }
}

/// `<stem>_<NN>.<ext>` next to `path`.
pub fn burst_frame_path(path: &Path, index: u32) -> PathBuf {
    let stem = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "capture".to_string());
    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().into_owned())
        .unwrap_or_else(|| "png".to_string());
    path.with_file_name(format!("{stem}_{index:02}.{ext}"))
}

/// How long a caller waits for the render thread to fulfil a single snapshot.
pub const CAPTURE_TIMEOUT: Duration = Duration::from_secs(3);

/// Region in virtual-desktop coordinates: origin (x, y) and size (w, h).
pub fn crop_region(
    cursor: (f32, f32),
    virtual_origin: (f32, f32),
    virtual_size: (u32, u32),
    size: Option<(u32, u32)>,
) -> (f32, f32, u32, u32) {
    let (vw, vh) = virtual_size;
    match size {
        None => (virtual_origin.0, virtual_origin.1, vw.max(1), vh.max(1)),
        Some((w, h)) => {
            let w = w.clamp(16, vw.max(16));
            let h = h.clamp(16, vh.max(16));
            let max_x = virtual_origin.0 + (vw - w) as f32;
            let max_y = virtual_origin.1 + (vh - h) as f32;
            let x = (cursor.0 - w as f32 * 0.5)
                .clamp(virtual_origin.0, max_x.max(virtual_origin.0))
                .round();
            let y = (cursor.1 - h as f32 * 0.5)
                .clamp(virtual_origin.1, max_y.max(virtual_origin.1))
                .round();
            (x, y, w, h)
        }
    }
}

/// An overlay frame read back from the GPU: opaque RGB, composited over a dark grey background
/// so transparent areas are readable in any viewer. Encoding it to PNG is the slow part (tens of
/// milliseconds in debug builds), so callers do that off the render thread.
pub struct CapturedFrame {
    pub width: u32,
    pub height: u32,
    pub rgb: Vec<u8>,
}

impl CapturedFrame {
    /// Writes the frame as an 8-bit RGB PNG, creating parent directories as needed.
    pub fn write_png(&self, path: &Path) -> Result<String, String> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("create {}: {e}", parent.display()))?;
        }
        let file = std::fs::File::create(path).map_err(|e| format!("create {}: {e}", path.display()))?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), self.width, self.height);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().map_err(|e| e.to_string())?;
        writer.write_image_data(&self.rgb).map_err(|e| e.to_string())?;
        writer.finish().map_err(|e| e.to_string())?;
        Ok(path.display().to_string())
    }
}

/// Renders `config` with `renderer` into an offscreen texture covering `region` and writes it
/// to `path` as a PNG. Convenience wrapper over [`capture_frame`] + [`CapturedFrame::write_png`].
#[allow(clippy::too_many_arguments)]
pub fn capture_to_png(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut OverlayRenderer,
    config: &AppConfig,
    format: wgpu::TextureFormat,
    region: (f32, f32, u32, u32),
    path: &Path,
) -> Result<String, String> {
    capture_frame(device, queue, renderer, config, format, region)?.write_png(path)
}

/// Renders `config` with `renderer` into an offscreen texture covering `region` and reads the
/// pixels back. Blocks on the GPU for the readback only (a few milliseconds).
pub fn capture_frame(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    renderer: &mut OverlayRenderer,
    config: &AppConfig,
    format: wgpu::TextureFormat,
    region: (f32, f32, u32, u32),
) -> Result<CapturedFrame, String> {
    let (rx, ry, width, height) = region;

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("capture_target"),
        size: wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

    // Render with the region as the "screen": the shaders map world → clip through
    // `virtual_origin` and `screen_size`, so a crop is just a temporary origin change.
    let saved_origin = renderer.virtual_origin;
    renderer.virtual_origin = (rx, ry);
    renderer.render(device, queue, &view, width, height, config);
    renderer.virtual_origin = saved_origin;
    log::info!(
        "[capture] {}x{} at ({rx}, {ry}): {} capsules, {} billboards; chain {}",
        width,
        height,
        renderer.last_counts.0,
        renderer.last_counts.1,
        renderer.chain_summary()
    );

    // Read back (rows padded to 256 bytes as required by wgpu).
    let bytes_per_pixel = 4u32;
    let unpadded = width * bytes_per_pixel;
    let align = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
    let padded = unpadded.div_ceil(align) * align;
    let buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("capture_readback"),
        size: (padded * height) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("capture_encoder"),
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(padded),
                rows_per_image: None,
            },
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
    queue.submit(std::iter::once(encoder.finish()));

    let (tx, rx_map) = std::sync::mpsc::channel();
    buffer.map_async(wgpu::MapMode::Read, .., move |r| {
        let _ = tx.send(r);
    });
    device
        .poll(wgpu::PollType::wait_indefinitely())
        .map_err(|e| format!("device poll failed: {e:?}"))?;
    rx_map
        .recv_timeout(CAPTURE_TIMEOUT)
        .map_err(|_| "readback timed out".to_string())?
        .map_err(|e| format!("buffer map failed: {e:?}"))?;

    let bgra_order = matches!(
        format,
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );
    let mut rgb = Vec::with_capacity((width * height * 3) as usize);
    {
        let data = buffer
            .get_mapped_range(..)
            .map_err(|e| format!("mapped range unavailable: {e:?}"))?;
        // Background the overlay is composited over in the PNG.
        const BG: [f32; 3] = [40.0, 40.0, 48.0];
        for row in 0..height {
            let start = (row * padded) as usize;
            let line = &data[start..start + unpadded as usize];
            for px in line.as_chunks::<4>().0 {
                let (r, g, b, a) = if bgra_order {
                    (px[2], px[1], px[0], px[3])
                } else {
                    (px[0], px[1], px[2], px[3])
                };
                // Pre-multiplied source over opaque background.
                let a_f = a as f32 / 255.0;
                rgb.push((r as f32 + BG[0] * (1.0 - a_f)).round().min(255.0) as u8);
                rgb.push((g as f32 + BG[1] * (1.0 - a_f)).round().min(255.0) as u8);
                rgb.push((b as f32 + BG[2] * (1.0 - a_f)).round().min(255.0) as u8);
            }
        }
    }
    buffer.unmap();

    Ok(CapturedFrame { width, height, rgb })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crop_is_centred_on_cursor_and_clamped_to_the_desktop() {
        let (x, y, w, h) = crop_region((1000.0, 500.0), (0.0, 0.0), (2560, 1600), Some((800, 400)));
        assert_eq!((x, y, w, h), (600.0, 300.0, 800, 400));
        // Near the top-left corner the crop sticks to the edge.
        let (x, y, ..) = crop_region((10.0, 10.0), (0.0, 0.0), (2560, 1600), Some((800, 400)));
        assert_eq!((x, y), (0.0, 0.0));
        // Near the bottom-right it never overruns the desktop.
        let (x, y, ..) = crop_region((2550.0, 1590.0), (0.0, 0.0), (2560, 1600), Some((800, 400)));
        assert_eq!((x, y), (1760.0, 1200.0));
        // Negative virtual origin (monitor to the left) is respected.
        let (x, y, ..) = crop_region((-1500.0, 100.0), (-1920.0, 0.0), (4480, 1600), Some((800, 400)));
        assert_eq!((x, y), (-1900.0, 0.0));
    }

    #[test]
    fn full_capture_covers_the_whole_overlay() {
        assert_eq!(
            crop_region((0.0, 0.0), (-1920.0, -100.0), (4480, 1700), None),
            (-1920.0, -100.0, 4480, 1700)
        );
    }

    #[test]
    fn crop_larger_than_the_desktop_is_reduced() {
        let (_, _, w, h) = crop_region((0.0, 0.0), (0.0, 0.0), (1280, 720), Some((5000, 5000)));
        assert_eq!((w, h), (1280, 720));
    }

    #[test]
    fn burst_frames_are_numbered_next_to_the_requested_file() {
        let p = burst_frame_path(Path::new("C:/shots/trail.png"), 7);
        assert_eq!(p, PathBuf::from("C:/shots/trail_07.png"));
        let (tx, _rx) = std::sync::mpsc::channel();
        let mut req = CaptureRequest::new(PathBuf::from("out.png"), None, 3, Duration::from_millis(50), tx);
        assert_eq!(req.frame_path(), PathBuf::from("out_00.png"));
        let t0 = Instant::now();
        assert!(req.is_due(t0));
        assert!(!req.frame_done("out_00.png".into(), t0));
        assert!(!req.is_due(t0), "the next frame waits for the interval");
        assert!(req.is_due(t0 + Duration::from_millis(50)));
        assert_eq!(req.frame_path(), PathBuf::from("out_01.png"));
        assert!(!req.frame_done("out_01.png".into(), t0));
        assert!(req.frame_done("out_02.png".into(), t0), "third frame completes the burst");
        assert_eq!(req.summary(), "3 frames: out_00.png … out_02.png");
        assert_eq!(req.timeout(), CAPTURE_TIMEOUT + Duration::from_millis(150));

        let (tx, _rx) = std::sync::mpsc::channel();
        let mut single = CaptureRequest::new(PathBuf::from("one.png"), None, 1, Duration::ZERO, tx);
        assert_eq!(single.frame_path(), PathBuf::from("one.png"), "single snapshots keep their name");
        assert!(single.frame_done("one.png".into(), t0));
        assert_eq!(single.summary(), "one.png");
    }
}
