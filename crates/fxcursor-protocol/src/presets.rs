use crate::config::{AppConfig, LayerConfig, TrailConfig, HeadConfig, RippleConfig, ParticleConfig, SatelliteConfig, RainbowConfig, EffectMode};
use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct PresetInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: AppConfig,
}

pub fn get_builtin_presets() -> Vec<PresetInfo> {
    vec![
        // 1. Master 4-Layer Glow (Authentic D3D11 Master Design)
        PresetInfo {
            id: "master_4layer".to_string(),
            name: "Master 4-Layer Glow".to_string(),
            description: "Authentic D3D11 master design (Outer Glow + Mid Shadow + Crisp Core + Inner Spine)".to_string(),
            config: AppConfig::default(),
        },

        // 2. Neon Cyberpunk
        PresetInfo {
            id: "neon_cyberpunk".to_string(),
            name: "Neon Cyberpunk".to_string(),
            description: "Electric cyan outer glow with deep violet contrast, hot plasma core, and high-velocity sparks".to_string(),
            config: AppConfig {
                enabled: true,
                effect_mode: EffectMode::FourLayerGlow,
                trail: TrailConfig {
                    enabled: true,
                    length: 90,
                    spring: 60.0,
                    damping: 28.0,
                    head_spring: 60.0,
                    head_damping: 28.0,
                    lazy_enabled: false,
                    lazy_radius: 30.0,
                    lazy_friction: 0.4,
                    cursor_size: 44.0,
                    min_width: 2.0,
                    velocity_width_mult: 0.6,
                    velocity_alpha_mult: 0.15,
                    interpolation_steps: 3,
                    fade_mode: 3,
                    enable_gradient: true,
                    adaptive_quality: true,
                    layers: [
                        LayerConfig {
                            enabled: true,
                            start_color: [0.0, 0.95, 1.0, 1.0], // Electric Cyan
                            end_color: [0.6, 0.0, 1.0, 0.0],   // Neon Violet
                            width_factor: 1.80,
                            alpha_factor: 1.0,
                            start_blur: 0.45,
                            end_blur: 0.60,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.4, 0.0, 0.9, 1.0], // Deep Violet
                            end_color: [0.1, 0.0, 0.3, 0.0],
                            width_factor: 1.10,
                            alpha_factor: 1.0,
                            start_blur: 0.15,
                            end_blur: 0.15,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.0, 1.0, 0.85, 1.0], // Bright Plasma Core
                            end_color: [0.0, 0.8, 1.0, 0.0],
                            width_factor: 0.55,
                            alpha_factor: 1.0,
                            start_blur: 0.08,
                            end_blur: 0.08,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.05, 0.0, 0.15, 1.0], // Dark Core Needle
                            end_color: [0.05, 0.0, 0.15, 1.0],
                            width_factor: 0.18,
                            alpha_factor: 1.0,
                            start_blur: 0.05,
                            end_blur: 0.05,
                        },
                    ],
                },
                head: HeadConfig {
                    enabled: true,
                    size: 20.0,
                    squish_intensity: 3.5,
                    squish_smoothing: 60.0,
                    color: [0.0, 0.95, 1.0, 1.0],
                    filled: true,
                    thickness: -1.0,
                },
                ripple: RippleConfig {
                    enabled: true,
                    max_diameter: 120.0,
                    duration_ms: 550,
                    start_width: 9.0,
                    color_left: [0.0, 0.95, 1.0, 1.0],
                    color_right: [1.0, 0.0, 0.6, 1.0],
                    color_middle: [0.6, 0.0, 1.0, 1.0],
                },
                particles: ParticleConfig {
                    enabled: true,
                    count_per_click: 24,
                    duration_ms: 600,
                    base_speed: 350.0,
                    gravity: 80.0,
                    friction: 0.92,
                    size: 3.5,
                    color: [0.0, 0.95, 1.0, 1.0],
                },
                satellites: SatelliteConfig::default(),
                rainbow: RainbowConfig::default(),
                ..AppConfig::default()
            },
        },

        // 3. Razor Minimalist Spine
        PresetInfo {
            id: "razor_spine".to_string(),
            name: "Razor Minimalist Spine".to_string(),
            description: "Ultra-thin, zero-blur high-precision centerline needle for esports and minimalists".to_string(),
            config: AppConfig {
                enabled: true,
                // FourLayerGlow: the mode mask must not hide what the preset configures (its
                // shadow layer and ripples); the per-layer `enabled` flags drop the glow.
                effect_mode: EffectMode::FourLayerGlow,
                trail: TrailConfig {
                    enabled: true,
                    length: 50,
                    spring: 80.0,
                    damping: 35.0,
                    head_spring: 80.0,
                    head_damping: 35.0,
                    lazy_enabled: false,
                    lazy_radius: 30.0,
                    lazy_friction: 0.4,
                    cursor_size: 28.0,
                    min_width: 1.5,
                    velocity_width_mult: 0.3,
                    velocity_alpha_mult: 0.05,
                    interpolation_steps: 2,
                    fade_mode: 1, // EaseOut
                    enable_gradient: false,
                    adaptive_quality: true,
                    layers: [
                        LayerConfig {
                            enabled: false,
                            ..LayerConfig::default()
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.0, 0.0, 0.0, 0.9],
                            end_color: [0.0, 0.0, 0.0, 0.0],
                            width_factor: 0.40,
                            alpha_factor: 1.0,
                            start_blur: 0.04,
                            end_blur: 0.04,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [1.0, 1.0, 1.0, 1.0],
                            end_color: [1.0, 1.0, 1.0, 0.0],
                            width_factor: 0.22,
                            alpha_factor: 1.0,
                            start_blur: 0.02,
                            end_blur: 0.02,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.0, 0.0, 0.0, 1.0],
                            end_color: [0.0, 0.0, 0.0, 1.0],
                            width_factor: 0.08,
                            alpha_factor: 1.0,
                            start_blur: 0.01,
                            end_blur: 0.01,
                        },
                    ],
                },
                head: HeadConfig {
                    enabled: true,
                    size: 14.0,
                    squish_intensity: 2.0,
                    squish_smoothing: 80.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    filled: false,
                    thickness: 2.0,
                },
                ripple: RippleConfig {
                    enabled: true,
                    max_diameter: 70.0,
                    duration_ms: 400,
                    start_width: 4.0,
                    color_left: [1.0, 1.0, 1.0, 0.8],
                    color_right: [1.0, 0.3, 0.3, 0.8],
                    color_middle: [0.3, 0.8, 1.0, 0.8],
                },
                particles: ParticleConfig {
                    enabled: false,
                    ..ParticleConfig::default()
                },
                satellites: SatelliteConfig::default(),
                rainbow: RainbowConfig::default(),
                ..AppConfig::default()
            },
        },

        // 4. Celestial Orbit
        PresetInfo {
            id: "celestial_orbit".to_string(),
            name: "Celestial Orbit".to_string(),
            description: "Glowing solar core with 4 revolving celestial satellites and gentle lunar ripples".to_string(),
            config: AppConfig {
                enabled: true,
                // FourLayerGlow: SatellitesOnly hid the trail and ripples this preset configures.
                effect_mode: EffectMode::FourLayerGlow,
                trail: TrailConfig {
                    enabled: true,
                    length: 70,
                    spring: 45.0,
                    damping: 32.0,
                    head_spring: 45.0,
                    head_damping: 32.0,
                    lazy_enabled: false,
                    lazy_radius: 30.0,
                    lazy_friction: 0.4,
                    cursor_size: 36.0,
                    min_width: 2.0,
                    velocity_width_mult: 0.4,
                    velocity_alpha_mult: 0.1,
                    interpolation_steps: 2,
                    fade_mode: 3,
                    enable_gradient: true,
                    adaptive_quality: true,
                    layers: [
                        LayerConfig {
                            enabled: true,
                            start_color: [0.9, 0.7, 1.0, 0.9],
                            end_color: [0.4, 0.2, 0.8, 0.0],
                            width_factor: 1.40,
                            alpha_factor: 1.0,
                            start_blur: 0.35,
                            end_blur: 0.45,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.2, 0.0, 0.4, 0.9],
                            end_color: [0.1, 0.0, 0.2, 0.0],
                            width_factor: 0.85,
                            alpha_factor: 1.0,
                            start_blur: 0.10,
                            end_blur: 0.10,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [1.0, 0.95, 0.8, 1.0],
                            end_color: [1.0, 0.7, 0.3, 0.0],
                            width_factor: 0.45,
                            alpha_factor: 1.0,
                            start_blur: 0.08,
                            end_blur: 0.08,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.3, 0.1, 0.0, 1.0],
                            end_color: [0.3, 0.1, 0.0, 1.0],
                            width_factor: 0.12,
                            alpha_factor: 1.0,
                            start_blur: 0.08,
                            end_blur: 0.08,
                        },
                    ],
                },
                head: HeadConfig {
                    enabled: true,
                    size: 16.0,
                    squish_intensity: 2.5,
                    squish_smoothing: 50.0,
                    color: [1.0, 0.95, 0.8, 1.0],
                    filled: true,
                    thickness: -1.0,
                },
                ripple: RippleConfig {
                    enabled: true,
                    max_diameter: 110.0,
                    duration_ms: 650,
                    start_width: 6.0,
                    color_left: [1.0, 0.85, 0.4, 0.9],
                    color_right: [0.9, 0.5, 1.0, 0.9],
                    color_middle: [0.4, 0.9, 1.0, 0.9],
                },
                particles: ParticleConfig {
                    enabled: true,
                    count_per_click: 12,
                    duration_ms: 500,
                    base_speed: 250.0,
                    gravity: 60.0,
                    friction: 0.94,
                    size: 3.0,
                    color: [1.0, 0.85, 0.4, 1.0],
                },
                satellites: SatelliteConfig {
                    enabled: true,
                    count: 4,
                    orbit_diameter: 55.0,
                    size: 5.5,
                    speed: 3.5,
                    dual_ring: true,
                    show_orbit_ring: true,
                    orbit_ring_thickness: 1.2,
                    color: [1.0, 0.88, 0.4, 0.95],
                },
                rainbow: RainbowConfig::default(),
                ..AppConfig::default()
            },
        },

        // 5. Particle Firestorm
        PresetInfo {
            id: "particle_firestorm".to_string(),
            name: "Particle Firestorm".to_string(),
            description: "Blazing ember trail with dynamic gravity sparks and explosive shockwave bursts".to_string(),
            config: AppConfig {
                enabled: true,
                // FourLayerGlow: ParticlesOnly hid the "blazing ember trail" itself.
                effect_mode: EffectMode::FourLayerGlow,
                trail: TrailConfig {
                    enabled: true,
                    length: 85,
                    spring: 55.0,
                    damping: 25.0,
                    head_spring: 55.0,
                    head_damping: 25.0,
                    lazy_enabled: false,
                    lazy_radius: 30.0,
                    lazy_friction: 0.4,
                    cursor_size: 42.0,
                    min_width: 2.0,
                    velocity_width_mult: 0.7,
                    velocity_alpha_mult: 0.2,
                    interpolation_steps: 2,
                    fade_mode: 2, // Exponential
                    enable_gradient: true,
                    adaptive_quality: true,
                    layers: [
                        LayerConfig {
                            enabled: true,
                            start_color: [1.0, 0.4, 0.0, 1.0], // Fiery Orange
                            end_color: [0.8, 0.1, 0.0, 0.0],
                            width_factor: 1.60,
                            alpha_factor: 1.0,
                            start_blur: 0.40,
                            end_blur: 0.55,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.3, 0.05, 0.0, 1.0],
                            end_color: [0.1, 0.0, 0.0, 0.0],
                            width_factor: 0.95,
                            alpha_factor: 1.0,
                            start_blur: 0.12,
                            end_blur: 0.12,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [1.0, 0.9, 0.2, 1.0], // Molten Yellow Core
                            end_color: [1.0, 0.3, 0.0, 0.0],
                            width_factor: 0.50,
                            alpha_factor: 1.0,
                            start_blur: 0.10,
                            end_blur: 0.10,
                        },
                        LayerConfig {
                            enabled: true,
                            start_color: [0.2, 0.0, 0.0, 1.0],
                            end_color: [0.2, 0.0, 0.0, 1.0],
                            width_factor: 0.15,
                            alpha_factor: 1.0,
                            start_blur: 0.08,
                            end_blur: 0.08,
                        },
                    ],
                },
                head: HeadConfig {
                    enabled: true,
                    size: 20.0,
                    squish_intensity: 3.2,
                    squish_smoothing: 55.0,
                    color: [1.0, 0.8, 0.1, 1.0],
                    filled: true,
                    thickness: -1.0,
                },
                ripple: RippleConfig {
                    enabled: true,
                    max_diameter: 130.0,
                    duration_ms: 600,
                    start_width: 10.0,
                    color_left: [1.0, 0.5, 0.0, 1.0],
                    color_right: [1.0, 0.1, 0.0, 1.0],
                    color_middle: [1.0, 0.9, 0.2, 1.0],
                },
                particles: ParticleConfig {
                    enabled: true,
                    count_per_click: 36,
                    duration_ms: 650,
                    base_speed: 420.0,
                    gravity: 180.0,
                    friction: 0.91,
                    size: 4.0,
                    color: [1.0, 0.6, 0.1, 1.0],
                },
                satellites: SatelliteConfig::default(),
                rainbow: RainbowConfig::default(),
                ..AppConfig::default()
            },
        },

        // 6. Rainbow Aurora
        PresetInfo {
            id: "rainbow_aurora".to_string(),
            name: "Rainbow Aurora".to_string(),
            description: "Continuous HSL spectrum cycling across the 4-layer master ribbon".to_string(),
            config: AppConfig {
                enabled: true,
                effect_mode: EffectMode::FourLayerGlow,
                trail: TrailConfig {
                    enabled: true,
                    length: 80,
                    spring: 50.0,
                    damping: 30.0,
                    head_spring: 50.0,
                    head_damping: 30.0,
                    lazy_enabled: false,
                    lazy_radius: 30.0,
                    lazy_friction: 0.4,
                    cursor_size: 40.0,
                    min_width: 2.0,
                    velocity_width_mult: 0.5,
                    velocity_alpha_mult: 0.1,
                    interpolation_steps: 2,
                    fade_mode: 3,
                    enable_gradient: true,
                    adaptive_quality: true,
                    layers: AppConfig::default().trail.layers,
                },
                head: HeadConfig {
                    enabled: true,
                    size: 18.0,
                    squish_intensity: 3.0,
                    squish_smoothing: 50.0,
                    color: [1.0, 1.0, 1.0, 1.0],
                    filled: true,
                    thickness: -1.0,
                },
                ripple: RippleConfig::default(),
                particles: ParticleConfig::default(),
                satellites: SatelliteConfig::default(),
                rainbow: RainbowConfig {
                    enabled: true,
                    speed: 2.5,
                    saturation: 1.0,
                    lightness: 0.55,
                },
                ..AppConfig::default()
            },
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::self_healing::deserialize_with_self_healing;

    #[test]
    fn test_all_builtin_presets_are_valid() {
        let presets = get_builtin_presets();
        assert_eq!(presets.len(), 6);

        for preset in presets {
            assert!(!preset.id.is_empty());
            assert!(!preset.name.is_empty());
            let json = serde_json::to_string_pretty(&preset.config).expect("Failed to serialize preset");
            let outcome = deserialize_with_self_healing::<AppConfig>(&json).expect("Failed to deserialize preset");
            assert_eq!(outcome.value, preset.config);
            assert!(outcome.repaired_paths.is_empty());
        }
    }
}
