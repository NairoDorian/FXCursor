use serde::{Deserialize, Serialize};
use specta::Type;

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Type)]
pub enum EffectMode {
    #[default]
    FourLayerGlow,
    Ribbon,
    ParticlesOnly,
    SatellitesOnly,
    Minimal,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct TrailConfig {
    pub enabled: bool,
    pub length: u32,
    pub spring: f32,
    pub damping: f32,
    pub head_spring: f32,
    pub head_damping: f32,
    /// Number of leading nodes (including the head) that follow without overshoot; the rest of
    /// the chain is spring-driven and may whip. Keeps the ribbon clean right at the pointer.
    #[serde(default = "default_lead_nodes")]
    pub lead_nodes: u32,
    pub cursor_size: f32,
    pub min_width: f32,
    pub velocity_width_mult: f32,
    pub velocity_alpha_mult: f32,
    pub interpolation_steps: u32,
    pub fade_mode: u32,
    pub enable_gradient: bool,
    pub adaptive_quality: bool,
    pub layers: [LayerConfig; 4],
}

/// Default number of overshoot-free leading nodes (head + 3 followers).
fn default_lead_nodes() -> u32 {
    4
}

impl Default for TrailConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            length: 80,
            spring: 50.0,
            damping: 30.0,
            head_spring: 50.0,
            head_damping: 30.0,
            lead_nodes: 4,
            cursor_size: 40.0,
            min_width: 2.0,
            velocity_width_mult: 0.5,
            velocity_alpha_mult: 0.1,
            interpolation_steps: 2,
            fade_mode: 3, // 0=Linear, 1=EaseOut, 2=Exponential, 3=Sigmoid
            enable_gradient: true,
            adaptive_quality: true,
            layers: [
                // Layer 1: Outer Glow (Soft White Glow)
                LayerConfig {
                    enabled: true,
                    start_color: [1.0, 1.0, 1.0, 1.0],
                    end_color: [1.0, 1.0, 1.0, 0.0],
                    width_factor: 1.50,
                    alpha_factor: 1.0,
                    start_blur: 0.39,
                    end_blur: 0.50,
                },
                // Layer 2: Mid Layer (Black Shadow)
                LayerConfig {
                    enabled: true,
                    start_color: [0.0, 0.0, 0.0, 1.0],
                    end_color: [0.0, 0.0, 0.0, 0.0],
                    width_factor: 0.90,
                    alpha_factor: 1.0,
                    start_blur: 0.10,
                    end_blur: 0.10,
                },
                // Layer 3: Core (Crisp White Core)
                LayerConfig {
                    enabled: true,
                    start_color: [1.0, 1.0, 1.0, 1.0],
                    end_color: [1.0, 1.0, 1.0, 0.0],
                    width_factor: 0.50,
                    alpha_factor: 1.0,
                    start_blur: 0.10,
                    end_blur: 0.10,
                },
                // Layer 4: Inner Core (Thin Black Spine)
                LayerConfig {
                    enabled: true,
                    start_color: [0.0, 0.0, 0.0, 1.0],
                    end_color: [0.0, 0.0, 0.0, 1.0],
                    width_factor: 0.15,
                    alpha_factor: 1.0,
                    start_blur: 0.10,
                    end_blur: 0.10,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct HeadConfig {
    pub enabled: bool,
    pub size: f32,
    pub squish_intensity: f32,
    pub squish_smoothing: f32,
    pub color: [f32; 4],
    pub filled: bool,
    pub thickness: f32,
}

impl Default for HeadConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            size: 18.0,
            squish_intensity: 3.0,
            squish_smoothing: 50.0,
            color: [1.0, 1.0, 1.0, 1.0],
            filled: true,
            thickness: -1.0, // <0 filled
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RippleConfig {
    pub enabled: bool,
    pub max_diameter: f32,
    pub duration_ms: u32,
    pub start_width: f32,
    pub color_left: [f32; 4],
    pub color_right: [f32; 4],
    pub color_middle: [f32; 4],
}

impl Default for RippleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_diameter: 100.0,
            duration_ms: 600,
            start_width: 8.0,
            color_left: [1.0, 1.0, 1.0, 1.0],
            color_right: [1.0, 0.47, 0.47, 1.0],
            color_middle: [1.0, 1.0, 0.4, 1.0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ParticleConfig {
    pub enabled: bool,
    pub count_per_click: u32,
    pub duration_ms: u32,
    pub base_speed: f32,
    pub gravity: f32,
    pub friction: f32,
    pub size: f32,
    pub color: [f32; 4],
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            count_per_click: 16,
            duration_ms: 500,
            base_speed: 300.0,
            gravity: 120.0,
            friction: 0.90,
            size: 3.0,
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SatelliteConfig {
    pub enabled: bool,
    pub count: u32,
    pub orbit_diameter: f32,
    pub size: f32,
    pub speed: f32,
    pub dual_ring: bool,
    pub show_orbit_ring: bool,
    pub orbit_ring_thickness: f32,
    pub color: [f32; 4],
}

impl Default for SatelliteConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            count: 3,
            orbit_diameter: 50.0,
            size: 5.0,
            speed: 3.0,
            dual_ring: false,
            show_orbit_ring: true,
            orbit_ring_thickness: 1.0,
            color: [1.0, 1.0, 1.0, 0.8],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RainbowConfig {
    pub enabled: bool,
    pub speed: f32,
    pub saturation: f32,
    pub lightness: f32,
}

impl Default for RainbowConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            speed: 2.0,
            saturation: 1.0,
            lightness: 0.5,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct FpsCounterConfig {
    pub enabled: bool,
    pub refresh_rate_ms: u32,
    pub align_right: bool,
    pub align_bottom: bool,
}

impl Default for FpsCounterConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            refresh_rate_ms: 500,
            align_right: true,
            align_bottom: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GeneralConfig {
    pub autostart: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
    pub global_hotkey: String,
    pub selected_preset: String,
    /// Frame-rate cap for the overlay render loop. `0` = match the display refresh rate.
    #[serde(default)]
    pub max_fps: u32,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            autostart: false,
            minimize_to_tray: true,
            start_minimized: false,
            global_hotkey: "Ctrl+Shift+E".to_string(),
            selected_preset: "master_4layer".to_string(),
            max_fps: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AppConfig {
    pub enabled: bool,
    pub effect_mode: EffectMode,
    pub general: GeneralConfig,
    pub trail: TrailConfig,
    pub head: HeadConfig,
    pub ripple: RippleConfig,
    pub particles: ParticleConfig,
    pub satellites: SatelliteConfig,
    pub rainbow: RainbowConfig,
    pub fps_counter: FpsCounterConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            effect_mode: EffectMode::default(),
            general: GeneralConfig::default(),
            trail: TrailConfig::default(),
            head: HeadConfig::default(),
            ripple: RippleConfig::default(),
            particles: ParticleConfig::default(),
            satellites: SatelliteConfig::default(),
            rainbow: RainbowConfig::default(),
            fps_counter: FpsCounterConfig::default(),
        }
    }
}
