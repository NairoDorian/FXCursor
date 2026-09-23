//! Range validation for [`AppConfig`].
//!
//! The self-healing deserializer only repairs *types*. Values still arrive from places the
//! Studio sliders do not guard — a hand-edited `config.json`, `--apply` patches, imported or
//! user presets, raw IPC — and some of them break the renderer: a spring constant far above
//! Windhawk's 500 makes the explicit integrator explode, a non-finite float poisons the trail
//! chain, an enormous `interpolation_steps` or `satellites.count` allocates without bound.
//!
//! [`AppConfig::sanitize`] clamps every numeric field to a range the renderer handles safely.
//! The bounds are deliberately wider than the UI sliders: they reject what is unsafe, not what
//! is unusual.

use crate::config::{AppConfig, LayerConfig};

/// Clamps `value` into `[min, max]`; a non-finite value becomes `fallback`.
fn clamp_f32(
    value: &mut f32,
    min: f32,
    max: f32,
    fallback: f32,
    path: &'static str,
    changed: &mut Vec<&'static str>,
) {
    let fixed = if value.is_finite() {
        value.clamp(min, max)
    } else {
        fallback
    };
    if fixed.to_bits() != value.to_bits() {
        *value = fixed;
        changed.push(path);
    }
}

fn clamp_u32(
    value: &mut u32,
    min: u32,
    max: u32,
    path: &'static str,
    changed: &mut Vec<&'static str>,
) {
    let fixed = (*value).clamp(min, max);
    if fixed != *value {
        *value = fixed;
        changed.push(path);
    }
}

/// RGBA channels in 0–1 (non-finite → opaque white channel value 1).
fn clamp_color(color: &mut [f32; 4], path: &'static str, changed: &mut Vec<&'static str>) {
    let mut touched = false;
    for c in color.iter_mut() {
        let fixed = if c.is_finite() {
            c.clamp(0.0, 1.0)
        } else {
            1.0
        };
        if fixed.to_bits() != c.to_bits() {
            *c = fixed;
            touched = true;
        }
    }
    if touched {
        changed.push(path);
    }
}

fn sanitize_layer(layer: &mut LayerConfig, changed: &mut Vec<&'static str>) {
    clamp_color(
        &mut layer.start_color,
        "trail.layers[].start_color",
        changed,
    );
    clamp_color(&mut layer.end_color, "trail.layers[].end_color", changed);
    clamp_f32(
        &mut layer.width_factor,
        0.0,
        10.0,
        1.0,
        "trail.layers[].width_factor",
        changed,
    );
    clamp_f32(
        &mut layer.alpha_factor,
        0.0,
        4.0,
        1.0,
        "trail.layers[].alpha_factor",
        changed,
    );
    clamp_f32(
        &mut layer.start_blur,
        0.0,
        1.0,
        0.1,
        "trail.layers[].start_blur",
        changed,
    );
    clamp_f32(
        &mut layer.end_blur,
        0.0,
        1.0,
        0.1,
        "trail.layers[].end_blur",
        changed,
    );
}

impl AppConfig {
    /// Clamps every numeric field into a safe range and replaces non-finite floats. Returns the
    /// paths that were changed (empty for any configuration the Studio can produce).
    pub fn sanitize(&mut self) -> Vec<&'static str> {
        let mut changed = Vec::new();
        let c = &mut changed;

        let t = &mut self.trail;
        clamp_u32(&mut t.length, 4, 150, "trail.length", c);
        // Windhawk clamps springs to 1–500 (÷1000) and friction to 0–99 %.
        clamp_f32(&mut t.spring, 1.0, 500.0, 50.0, "trail.spring", c);
        clamp_f32(&mut t.damping, 0.0, 99.0, 30.0, "trail.damping", c);
        clamp_f32(&mut t.head_spring, 1.0, 500.0, 50.0, "trail.head_spring", c);
        clamp_f32(
            &mut t.head_damping,
            0.0,
            99.0,
            30.0,
            "trail.head_damping",
            c,
        );
        clamp_f32(
            &mut t.lazy_radius,
            0.0,
            1000.0,
            30.0,
            "trail.lazy_radius",
            c,
        );
        clamp_f32(
            &mut t.lazy_friction,
            0.0,
            0.99,
            0.4,
            "trail.lazy_friction",
            c,
        );
        clamp_f32(
            &mut t.cursor_size,
            0.0,
            1000.0,
            40.0,
            "trail.cursor_size",
            c,
        );
        clamp_f32(&mut t.min_width, 0.0, 200.0, 2.0, "trail.min_width", c);
        clamp_f32(
            &mut t.velocity_width_mult,
            0.0,
            20.0,
            0.5,
            "trail.velocity_width_mult",
            c,
        );
        clamp_f32(
            &mut t.velocity_alpha_mult,
            0.0,
            20.0,
            0.1,
            "trail.velocity_alpha_mult",
            c,
        );
        clamp_u32(
            &mut t.interpolation_steps,
            1,
            32,
            "trail.interpolation_steps",
            c,
        );
        clamp_u32(&mut t.fade_mode, 0, 4, "trail.fade_mode", c);
        for layer in &mut t.layers {
            sanitize_layer(layer, c);
        }

        let h = &mut self.head;
        clamp_f32(&mut h.size, 0.0, 1000.0, 18.0, "head.size", c);
        clamp_f32(
            &mut h.squish_intensity,
            0.0,
            100.0,
            3.0,
            "head.squish_intensity",
            c,
        );
        clamp_f32(
            &mut h.squish_smoothing,
            1.0,
            100.0,
            50.0,
            "head.squish_smoothing",
            c,
        );
        // Negative thickness is the legacy "filled" marker (see `HeadConfig::thickness`).
        clamp_f32(&mut h.thickness, -1.0, 200.0, 2.0, "head.thickness", c);
        clamp_color(&mut h.color, "head.color", c);

        let r = &mut self.ripple;
        clamp_f32(
            &mut r.max_diameter,
            0.0,
            4000.0,
            100.0,
            "ripple.max_diameter",
            c,
        );
        clamp_u32(&mut r.duration_ms, 16, 20_000, "ripple.duration_ms", c);
        clamp_f32(&mut r.start_width, 0.0, 500.0, 8.0, "ripple.start_width", c);
        clamp_color(&mut r.color_left, "ripple.color_left", c);
        clamp_color(&mut r.color_right, "ripple.color_right", c);
        clamp_color(&mut r.color_middle, "ripple.color_middle", c);

        let p = &mut self.particles;
        clamp_u32(
            &mut p.count_per_click,
            0,
            1024,
            "particles.count_per_click",
            c,
        );
        clamp_u32(&mut p.duration_ms, 16, 20_000, "particles.duration_ms", c);
        clamp_f32(
            &mut p.base_speed,
            0.0,
            20_000.0,
            200.0,
            "particles.base_speed",
            c,
        );
        clamp_f32(
            &mut p.gravity,
            -20_000.0,
            20_000.0,
            120.0,
            "particles.gravity",
            c,
        );
        clamp_f32(&mut p.friction, 0.0, 1.0, 0.9, "particles.friction", c);
        clamp_f32(&mut p.size, 0.0, 500.0, 3.0, "particles.size", c);
        clamp_color(&mut p.color, "particles.color", c);

        let s = &mut self.satellites;
        clamp_u32(&mut s.count, 1, 64, "satellites.count", c);
        clamp_f32(
            &mut s.orbit_diameter,
            0.0,
            4000.0,
            60.0,
            "satellites.orbit_diameter",
            c,
        );
        clamp_f32(&mut s.size, 0.0, 500.0, 6.0, "satellites.size", c);
        clamp_f32(&mut s.speed, -200.0, 200.0, 2.0, "satellites.speed", c);
        clamp_f32(
            &mut s.orbit_ring_thickness,
            0.0,
            100.0,
            1.0,
            "satellites.orbit_ring_thickness",
            c,
        );
        clamp_color(&mut s.color, "satellites.color", c);

        let rb = &mut self.rainbow;
        clamp_f32(&mut rb.speed, -360.0, 360.0, 1.0, "rainbow.speed", c);
        clamp_f32(&mut rb.saturation, 0.0, 1.0, 1.0, "rainbow.saturation", c);
        clamp_f32(&mut rb.lightness, 0.0, 1.0, 0.5, "rainbow.lightness", c);

        clamp_u32(
            &mut self.fps_counter.refresh_rate_ms,
            100,
            60_000,
            "fps_counter.refresh_rate_ms",
            c,
        );

        let g = &mut self.gpu_cursor;
        clamp_f32(
            &mut g.rotation_smoothing,
            1.0,
            30.0,
            6.0,
            "gpu_cursor.rotation_smoothing",
            c,
        );
        clamp_f32(
            &mut g.click_scale_percent,
            50.0,
            400.0,
            150.0,
            "gpu_cursor.click_scale_percent",
            c,
        );
        clamp_u32(
            &mut g.click_scale_duration_ms,
            16,
            5000,
            "gpu_cursor.click_scale_duration_ms",
            c,
        );

        // 0 = follow the display; otherwise the render loop's own 24–1000 fps window.
        if self.general.max_fps != 0 {
            clamp_u32(&mut self.general.max_fps, 24, 1000, "general.max_fps", c);
        }

        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::presets::get_builtin_presets;

    #[test]
    fn defaults_and_builtin_presets_are_already_valid() {
        let mut cfg = AppConfig::default();
        let changed = cfg.sanitize();
        assert!(
            changed.is_empty(),
            "defaults must not be touched: {changed:?}"
        );
        for preset in get_builtin_presets() {
            let mut cfg = preset.config.clone();
            let changed = cfg.sanitize();
            assert!(
                changed.is_empty(),
                "preset {} changed: {changed:?}",
                preset.id
            );
        }
    }

    #[test]
    fn unsafe_values_are_clamped_and_non_finite_replaced() {
        let mut cfg = AppConfig::default();
        cfg.trail.spring = 5_000.0;
        cfg.trail.damping = f32::NAN;
        cfg.trail.interpolation_steps = 1_000_000;
        cfg.trail.layers[0].start_color[3] = f32::INFINITY;
        cfg.satellites.count = u32::MAX;
        cfg.general.max_fps = 5;
        let changed = cfg.sanitize();
        assert_eq!(cfg.trail.spring, 500.0);
        assert_eq!(cfg.trail.damping, 30.0);
        assert_eq!(cfg.trail.interpolation_steps, 32);
        assert_eq!(cfg.trail.layers[0].start_color[3], 1.0);
        assert_eq!(cfg.satellites.count, 64);
        assert_eq!(cfg.general.max_fps, 24);
        assert!(changed.contains(&"trail.spring") && changed.contains(&"trail.damping"));
        assert!(cfg.sanitize().is_empty(), "sanitize is idempotent");
    }

    #[test]
    fn uncapped_frame_rate_stays_zero() {
        let mut cfg = AppConfig::default();
        cfg.general.max_fps = 0;
        cfg.sanitize();
        assert_eq!(cfg.general.max_fps, 0);
    }
}
