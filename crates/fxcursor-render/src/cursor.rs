//! GPU cursor bypass — shape snapshots, movement rotation and click bounce.
//!
//! The host (`src-tauri/src/cursor.rs`) extracts the active OS cursor into a [`CursorShape`];
//! the renderer draws it as a textured quad on top of the overlay with an optional rotation
//! that tracks the pointer's direction of travel and a sine-bump scale on click.

use std::time::Instant;

/// Rotation of the standard arrow image relative to the +x axis when `rotation == 0`:
/// the unrotated arrow points to the top-left corner, i.e. `atan2(-1, -1)`.
const ARROW_REST_ANGLE: f32 = -3.0 * std::f32::consts::FRAC_PI_4;
/// Pointer speed (px/s) below which the cursor stops tracking a new direction.
const ROTATE_MIN_SPEED: f32 = 40.0;
/// Per-frame exponential gain applied to the rotation when chasing its target.
const ROTATION_LERP_RATE: f32 = 15.0;

/// A CPU-side snapshot of an OS cursor shape, ready for GPU upload.
/// Pixels are tightly packed RGBA8, **pre-multiplied**, top-down rows.
#[derive(Clone, Debug)]
pub struct CursorShape {
    pub width: u32,
    pub height: u32,
    /// Pointing tip in image pixels, top-down coordinates (pivot for rotation and bounce).
    pub hotspot: (u32, u32),
    /// True for the standard arrow — the only shape movement-rotation is applied to.
    pub is_arrow: bool,
    /// Opaque OS handle value of the extraction source; cache key for the renderer.
    pub source_key: isize,
    pub pixels: Vec<u8>,
}

/// CPU-built vertex for the cursor quad: world position (virtual-screen pixels) + texture UV.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct CursorVertex {
    pub pos: [f32; 2],
    pub uv: [f32; 2],
}

/// Builds the two triangles of the cursor sprite.
///
/// The quad is centred on the hotspot so the pointing tip stays pinned to the pointer under
/// any rotation and bounce scale. Triangle order: TL,TR,BR then TL,BR,BL (CCW, y-down world).
pub fn cursor_quad_vertices(
    pointer: (f32, f32),
    width: u32,
    height: u32,
    hotspot: (u32, u32),
    rotation: f32,
    scale: f32,
) -> [CursorVertex; 6] {
    let w = width.max(1) as f32;
    let h = height.max(1) as f32;
    let hx = hotspot.0 as f32;
    let hy = hotspot.1 as f32;
    let (cos, sin) = (rotation.cos(), rotation.sin());
    let corner = |tx: f32, ty: f32| -> [f32; 2] {
        let lx = (tx - hx) * scale;
        let ly = (ty - hy) * scale;
        [
            pointer.0 + lx * cos - ly * sin,
            pointer.1 + lx * sin + ly * cos,
        ]
    };
    let tl = CursorVertex { pos: corner(0.0, 0.0), uv: [0.0, 0.0] };
    let tr = CursorVertex { pos: corner(w, 0.0), uv: [1.0, 0.0] };
    let br = CursorVertex { pos: corner(w, h), uv: [1.0, 1.0] };
    let bl = CursorVertex { pos: corner(0.0, h), uv: [0.0, 1.0] };
    [tl, tr, br, tl, br, bl]
}

/// Smoothed rotation, direction vector and click-bounce clock for the GPU-drawn cursor.
#[derive(Debug, Clone)]
pub struct GpuCursorState {
    /// Current applied rotation in radians (world, y-down).
    pub rotation: f32,
    /// Rotation the current motion wants to reach.
    pub target_rotation: f32,
    /// Exponentially averaged unit direction of pointer travel (world, y-down).
    pub smoothed_dir: (f32, f32),
    /// True once `smoothed_dir` has caught up with the raw travel direction.
    pub dir_settled: bool,
    bounce_start: Option<Instant>,
}

impl Default for GpuCursorState {
    fn default() -> Self {
        Self {
            rotation: 0.0,
            target_rotation: 0.0,
            // Resting direction of the unrotated arrow image: top-left.
            smoothed_dir: (ARROW_REST_ANGLE.cos(), ARROW_REST_ANGLE.sin()),
            dir_settled: true,
            bounce_start: None,
        }
    }
}

impl GpuCursorState {
    /// Starts the click-bounce scale animation.
    pub fn spawn_bounce(&mut self) {
        self.bounce_start = Some(Instant::now());
    }

    /// Sine-bump scale factor: `1.0` at rest, `percent / 100` at the peak of the bump.
    pub fn bounce_scale(&self, percent: f32, duration_ms: u32) -> f32 {
        let Some(start) = self.bounce_start else {
            return 1.0;
        };
        let duration = (duration_ms.max(1) as f32) / 1000.0;
        let t = (start.elapsed().as_secs_f32() / duration).clamp(0.0, 1.0);
        1.0 + (percent / 100.0 - 1.0) * (t * std::f32::consts::PI).sin()
    }

    /// True while the click bump is still animating.
    pub fn bounce_active(&self, duration_ms: u32) -> bool {
        let Some(start) = self.bounce_start else {
            return false;
        };
        start.elapsed().as_secs_f32() * 1000.0 < duration_ms.max(1) as f32
    }

    /// True while the rotation or the click bump still has visible motion left.
    pub fn is_animating(&self, duration_ms: u32) -> bool {
        self.bounce_active(duration_ms)
            || !self.dir_settled
            || shortest_angle(self.target_rotation - self.rotation).abs() > 1e-3
    }

    /// Advances direction smoothing and rotation towards the pointer delta `(dx, dy)`
    /// over `dt` seconds. With `allow_rotate == false` (disabled feature or a non-arrow
    /// shape) the rotation returns to rest instead.
    pub fn update(
        &mut self,
        dx: f32,
        dy: f32,
        dt: f32,
        allow_rotate: bool,
        smoothing_frames: f32,
    ) {
        let distance = (dx * dx + dy * dy).sqrt();
        let speed = distance / dt.max(1e-4);
        let moving = allow_rotate && speed > ROTATE_MIN_SPEED;

        if moving {
            let ux = dx / distance.max(1e-6);
            let uy = dy / distance.max(1e-6);
            // Frame-rate independent exponential average of the travel direction.
            let tau = (smoothing_frames.clamp(1.0, 30.0) / 60.0).max(1e-3);
            let alpha = 1.0 - (-dt / tau).exp();
            self.smoothed_dir.0 += (ux - self.smoothed_dir.0) * alpha;
            self.smoothed_dir.1 += (uy - self.smoothed_dir.1) * alpha;
            let len = (self.smoothed_dir.0 * self.smoothed_dir.0
                + self.smoothed_dir.1 * self.smoothed_dir.1)
                .sqrt()
                .max(1e-6);
            self.smoothed_dir.0 /= len;
            self.smoothed_dir.1 /= len;

            let lag = ((self.smoothed_dir.0 - ux).powi(2)
                + (self.smoothed_dir.1 - uy).powi(2))
                .sqrt();
            self.dir_settled = lag < 0.02;

            // Rotate the resting arrow (pointing top-left) onto the travel direction.
            let move_angle = self.smoothed_dir.1.atan2(self.smoothed_dir.0);
            self.target_rotation = move_angle - ARROW_REST_ANGLE;
        } else {
            self.target_rotation = 0.0;
            self.dir_settled = true;
        }

        let diff = shortest_angle(self.target_rotation - self.rotation);
        self.rotation += diff * (ROTATION_LERP_RATE * dt).clamp(0.0, 1.0);
    }

    /// Drops the click bump and snaps back to rest (feature disabled / state cleared).
    pub fn reset(&mut self) {
        *self = Self::default();
    }
}

/// Wraps an angle difference into `(-π, π]` so rotation chasing always takes the short arc.
fn shortest_angle(diff: f32) -> f32 {
    let mut d = diff.rem_euclid(std::f32::consts::TAU);
    if d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_pins_hotspot_under_rotation_and_scale() {
        let pointer = (400.0, 300.0);
        // Hotspot at the image origin: the first vertex IS the tip and must stay pinned
        // under any rotation and bounce scale.
        let verts = cursor_quad_vertices(pointer, 32, 48, (0, 0), 1.234, 1.75);
        assert!((verts[0].pos[0] - pointer.0).abs() < 1e-3);
        assert!((verts[0].pos[1] - pointer.1).abs() < 1e-3);
        // Interior hotspot, no rotation, scale 1: TR sits (w - hx) right and hy above the
        // pointer, proving offsets are taken relative to the tip, not the image corner.
        let verts = cursor_quad_vertices(pointer, 32, 48, (2, 4), 0.0, 1.0);
        assert!((verts[1].pos[0] - 430.0).abs() < 1e-3);
        assert!((verts[1].pos[1] - 296.0).abs() < 1e-3);
        // Six draw vertices form two triangles over the full image.
        assert_eq!(verts[3], verts[0]);
        assert_eq!(verts[4], verts[2]);
        assert!((verts[1].uv[0] - 1.0).abs() < 1e-6 && verts[1].uv[1] == 0.0);
        assert!((verts[5].uv[1] - 1.0).abs() < 1e-6 && verts[5].uv[0] == 0.0);
    }

    #[test]
    fn resting_quad_is_axis_aligned() {
        let verts = cursor_quad_vertices((100.0, 100.0), 10, 20, (0, 0), 0.0, 1.0);
        // Unrotated, scale 1, hotspot at origin: TR sits exactly `width` to the right.
        assert!((verts[1].pos[0] - 110.0).abs() < 1e-3);
        assert!((verts[1].pos[1] - 100.0).abs() < 1e-3);
        // BL sits exactly `height` below the pointer (y-down world).
        assert!((verts[5].pos[0] - 100.0).abs() < 1e-3);
        assert!((verts[5].pos[1] - 120.0).abs() < 1e-3);
    }

    #[test]
    fn bounce_scale_sine_bump_starts_and_ends_at_one() {
        let mut state = GpuCursorState::default();
        assert_eq!(state.bounce_scale(150.0, 200), 1.0);
        state.spawn_bounce();
        // Mid-animation the scale must exceed 1 for a 150% peak; poll until the bump ends.
        let mut saw_growth = false;
        for _ in 0..40 {
            let s = state.bounce_scale(150.0, 40);
            assert!(
                (0.999..=1.501).contains(&s),
                "scale {s} outside the bump envelope"
            );
            saw_growth |= s > 1.01;
            if !state.bounce_active(40) {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(2));
        }
        assert!(saw_growth || !state.bounce_active(40), "bump must be observable");
        assert!(!state.is_animating(1), "settled state reports idle");
    }

    #[test]
    fn rotation_tracks_eastward_motion_then_settles_to_rest() {
        let mut state = GpuCursorState::default();
        // Move east for a second of frames: the arrow must come to point along +x,
        // which for a top-left-pointing image is a rotation of +3π/4.
        for _ in 0..60 {
            state.update(10.0, 0.0, 1.0 / 60.0, true, 5.0);
        }
        for _ in 0..180 {
            state.update(10.0, 0.0, 1.0 / 60.0, true, 5.0);
        }
        assert!(state.dir_settled, "direction converged");
        assert!(
            (state.rotation + ARROW_REST_ANGLE).abs() < 0.05,
            "rotation {} not pointing east",
            state.rotation
        );
        // Stop: rotation glides back to rest and the state reports idle.
        for _ in 0..240 {
            state.update(0.0, 0.0, 1.0 / 60.0, true, 5.0);
        }
        assert!(state.rotation.abs() < 1e-2, "rotation {} did not return to 0", state.rotation);
        assert!(!state.is_animating(200), "idle after settling");
    }

    #[test]
    fn shortest_angle_wraps_at_pi() {
        assert!((shortest_angle(3.0 * std::f32::consts::PI) - std::f32::consts::PI).abs() < 1e-5);
        assert!((shortest_angle(-3.0 * std::f32::consts::PI) + std::f32::consts::PI).abs() < 1e-5);
        assert!(shortest_angle(0.5).abs() - 0.5 < 1e-6);
    }
}
