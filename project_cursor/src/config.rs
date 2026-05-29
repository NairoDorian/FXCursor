use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    pub enabled: bool,
    pub start_color: [f32; 4],
    pub end_color: [f32; 4],
    pub width_factor: f32,
    pub alpha_factor: f32,
    pub start_blur: f32,
    pub end_blur: f32,
}

impl Default for LayerConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            start_color: [1.0, 1.0, 1.0, 1.0],
            end_color: [1.0, 1.0, 1.0, 0.0],
            width_factor: 1.0,
            alpha_factor: 1.0,
            start_blur: 0.1,
            end_blur: 0.1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct AppConfig {
    pub enabled: bool,
    pub effect_type: u32,       // 0: Ribbon Trail, 1: Ripple, 2: Glow
    pub trail_color: [f32; 4],  // RGBA [0.0 - 1.0] (Used for single glow or overall default)
    pub trail_length: u32,
    pub trail_width: f32,       // Master reference cursor size
    pub speed: f32,             // Not used directly in ribbon spring model, but preserved for general
    pub friction: f32,          // Preserved
    pub gravity: f32,           // Preserved
    pub ripple_radius: f32,     // Preserved
    pub click_response: bool,   // Ripple on click

    // Ribbon Trail specific
    pub head_spring: f32,
    pub head_friction: f32,
    pub body_spring: f32,
    pub body_friction: f32,
    pub position_skip: u32,
    pub interpolation_steps: u32,
    pub fade_mode: u32,         // 0: Linear, 1: Ease-Out, 2: Exponential, 3: Sigmoid
    pub enable_gradient: bool,
    pub rainbow_mode: bool,
    pub rainbow_speed: f32,
    pub adaptive_quality: bool,
    pub min_trail_width: f32,
    pub velocity_width_multiplier: f32,
    pub velocity_alpha_multiplier: f32,
    pub layers: [LayerConfig; 4],
    pub trail_style: u32,       // 0: Continuous Ribbon, 1: Overlapping Circles

    // Squishy Cursor Head
    pub head_enabled: bool,
    pub head_filled: bool,
    pub head_color: [f32; 4],
    pub head_size: f32,
    pub head_outline_width: f32,
    pub head_squish_intensity: f32,
    pub head_squish_smoothing: f32,

    // Click Ripples (custom colors)
    pub ripple_left_color: [f32; 4],
    pub ripple_right_color: [f32; 4],
    pub ripple_middle_color: [f32; 4],
    pub ripple_duration: f32,   // seconds
    pub ripple_start_width: f32,

    // Click Particles
    pub particle_enabled: bool,
    pub particle_count: u32,
    pub particle_speed: f32,
    pub particle_lifetime: f32, // seconds
    pub particle_size: f32,
    pub particle_friction: f32,
    pub particle_gravity: f32,

    // Satellite Orbitals
    pub satellite_enabled: bool,
    pub satellite_count: u32,
    pub satellite_orbit_diameter: f32,
    pub satellite_size: f32,
    pub satellite_filled: bool,
    pub satellite_outline_width: f32,
    pub satellite_color: [f32; 4],
    pub satellite_speed: f32,
    pub satellite_enable_dual_ring: bool,
    pub satellite_dual_speed: f32,
    pub satellite_show_orbit_ring: bool,
    pub satellite_ring_width: f32,
    pub satellite_ring_color: [f32; 4],
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            effect_type: 0,
            trail_color: [0.0, 0.8, 1.0, 1.0], // Neon cyan
            trail_length: 50,
            trail_width: 40.0,
            speed: 1.0,
            friction: 0.08,
            gravity: 0.0,
            ripple_radius: 100.0,
            click_response: true,

            head_spring: 50.0,
            head_friction: 30.0,
            body_spring: 50.0,
            body_friction: 30.0,
            position_skip: 0,
            interpolation_steps: 8,
            fade_mode: 3, // Sigmoid
            enable_gradient: true,
            rainbow_mode: false,
            rainbow_speed: 2.0,
            adaptive_quality: true,
            min_trail_width: 2.0,
            velocity_width_multiplier: 0.5,
            velocity_alpha_multiplier: 0.1,
            layers: [
                // Layer 1 (Outer Glow)
                LayerConfig {
                    enabled: true,
                    start_color: [0.0, 0.8, 1.0, 1.0], // Neon cyan
                    end_color: [0.0, 0.8, 1.0, 0.0],
                    width_factor: 1.5,
                    alpha_factor: 1.0,
                    start_blur: 0.39,
                    end_blur: 0.5,
                },
                // Layer 2 (Mid Layer)
                LayerConfig {
                    enabled: true,
                    start_color: [0.0, 0.0, 0.0, 1.0], // Black
                    end_color: [0.0, 0.0, 0.0, 0.0],
                    width_factor: 0.9,
                    alpha_factor: 1.0,
                    start_blur: 0.1,
                    end_blur: 0.1,
                },
                // Layer 3 (Core)
                LayerConfig {
                    enabled: true,
                    start_color: [1.0, 1.0, 1.0, 1.0], // White
                    end_color: [0.0, 0.8, 1.0, 0.0],   // Fade to transparent cyan
                    width_factor: 0.5,
                    alpha_factor: 1.0,
                    start_blur: 0.1,
                    end_blur: 0.1,
                },
                // Layer 4 (Inner Core)
                LayerConfig {
                    enabled: true,
                    start_color: [0.0, 0.0, 0.0, 1.0], // Black
                    end_color: [0.0, 0.0, 0.0, 1.0],
                    width_factor: 0.15,
                    alpha_factor: 1.0,
                    start_blur: 0.1,
                    end_blur: 0.1,
                },
            ],
            trail_style: 0,

            head_enabled: false,
            head_filled: false,
            head_color: [1.0, 1.0, 1.0, 1.0],
            head_size: 18.0,
            head_outline_width: 2.0,
            head_squish_intensity: 3.0,
            head_squish_smoothing: 50.0,

            ripple_left_color: [1.0, 1.0, 1.0, 1.0],
            ripple_right_color: [1.0, 0.46, 0.46, 1.0], // FFFF7777
            ripple_middle_color: [1.0, 1.0, 0.4, 1.0],  // FFFFFF66
            ripple_duration: 0.6,
            ripple_start_width: 8.0,

            particle_enabled: false,
            particle_count: 16,
            particle_speed: 300.0,
            particle_lifetime: 0.5,
            particle_size: 3.0,
            particle_friction: 90.0,
            particle_gravity: 200.0,

            satellite_enabled: false,
            satellite_count: 4,
            satellite_orbit_diameter: 50.0,
            satellite_size: 6.0,
            satellite_filled: false,
            satellite_outline_width: 2.0,
            satellite_color: [1.0, 1.0, 1.0, 1.0],
            satellite_speed: 2.0,
            satellite_enable_dual_ring: false,
            satellite_dual_speed: 1.0,
            satellite_show_orbit_ring: false,
            satellite_ring_width: 1.0,
            satellite_ring_color: [1.0, 1.0, 1.0, 0.27], // 44FFFFFF
        }
    }
}

impl AppConfig {
    pub fn load_or_default() -> Self {
        let path = Path::new("config.ron");
        if path.exists() {
            if let Ok(mut file) = File::open(path) {
                let mut contents = String::new();
                if file.read_to_string(&mut contents).is_ok() {
                    if let Ok(config) = ron::from_str(&contents) {
                        return config;
                    }
                }
            }
        }
        
        let default_config = Self::default();
        let _ = default_config.save();
        default_config
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let path = Path::new("config.ron");
        let serialized = ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default())?;
        let mut file = File::create(path)?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}
