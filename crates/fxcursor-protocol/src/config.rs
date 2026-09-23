use serde::{Deserialize, Serialize};
use specta::Type;

/// Global rendering preset/mode selector controlling which visual subsystems are active.
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize, Type)]
pub enum EffectMode {
    /// Everything may draw: 4-layer ribbon, head, ripples, particles and satellites (each still
    /// gated by its own `enabled` flag).
    #[default]
    FourLayerGlow,
    /// Ribbon trail and cursor head only; disables click ripples, particles, and satellites.
    Ribbon,
    /// Click feedback only: cursor head, shockwave ripples, and particle bursts.
    ParticlesOnly,
    /// Orbitals only: cursor head and orbiting satellites.
    SatellitesOnly,
    /// Minimal low-profile: crisp core and inner spine layers only with cursor head.
    Minimal,
}

/// Visual properties for one layer of the 4-layer master ribbon.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct LayerConfig {
    /// Whether this specific layer is rendered.
    pub enabled: bool,
    /// RGBA start color at the head of the trail (straight alpha, 0.0–1.0).
    pub start_color: [f32; 4],
    /// RGBA end color at the tail of the trail (straight alpha, 0.0–1.0).
    pub end_color: [f32; 4],
    /// Width multiplier relative to the base trail width (e.g. 1.50 for Outer Glow, 0.15 for Inner Spine).
    pub width_factor: f32,
    /// Opacity multiplier applied to this layer's colors (0.0–1.0).
    pub alpha_factor: f32,
    /// Blur/feathering fraction at the head of the trail (0.0 = razor sharp, 0.5 = soft feather).
    pub start_blur: f32,
    /// Blur/feathering fraction at the tail of the trail (0.0 = razor sharp, 0.5 = soft feather).
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

/// Physics, geometry, and layer parameters governing the cursor ribbon trail.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct TrailConfig {
    /// Master toggle for ribbon trail physics and rendering.
    pub enabled: bool,
    /// Number of simulated discrete nodes in the spring chain (4–150). Width, fade and blur
    /// are parameterised by node index, so this is also the length of the visible taper.
    pub length: u32,
    /// Spring constant for the trailing body nodes, Windhawk scale: `k = spring / 1000` per
    /// 1/120 s reference frame (1–500).
    pub spring: f32,
    /// Velocity friction percent for the body nodes (0–99): each reference frame keeps
    /// `1 − damping/100` of the velocity.
    pub damping: f32,
    /// Spring stiffness constant for the leading head node (same scale as `spring`: /1000).
    pub head_spring: f32,
    /// Velocity friction percent for the leading head node (0–99; higher = more damping).
    pub head_damping: f32,
    /// LazyBrush: engage the TD-style dead-zone pointer smoother.
    #[serde(default)]
    pub lazy_enabled: bool,
    /// Dead-zone radius in px the pointer must exceed before the brush starts moving.
    #[serde(default = "default_lazy_radius")]
    pub lazy_radius: f32,
    /// Brush friction 0–0.99: fraction of the excess distance NOT applied per frame
    /// (0 = snap to pointer, →1 = frozen). TD formula: factor = 1 - sqrt(1-(1-f)^2).
    #[serde(default = "default_lazy_friction")]
    pub lazy_friction: f32,
    /// Reference ribbon width in physical pixels at the head; each layer scales it by its
    /// `width_factor`.
    pub cursor_size: f32,
    /// Minimum clamping width in physical pixels at the tail of the trail.
    pub min_width: f32,
    /// Extra width at full speed: `width × (1 + mult × min(speed / 20, 1))`, speed in px per
    /// 1/120 s (saturates at 2400 px/s, as in Windhawk).
    pub velocity_width_mult: f32,
    /// Extra opacity at full speed, same normalisation as `velocity_width_mult`.
    pub velocity_alpha_mult: f32,
    /// Fixed Catmull-Rom sub-samples per node segment. Only used when `adaptive_quality` is
    /// off; the adaptive path picks its own count from the local curvature.
    pub interpolation_steps: u32,
    /// Falloff curve along the length of the trail
    /// (0=Linear, 1=EaseOut, 2=Exponential, 3=Sigmoid, 4=Smoothstep).
    pub fade_mode: u32,
    /// Whether gradient interpolation between start and end color is applied.
    pub enable_gradient: bool,
    /// Curvature-adaptive spline sampling (3–24 px spacing): dense in bends, sparse on straight
    /// runs. Overrides `interpolation_steps`.
    pub adaptive_quality: bool,
    /// 4-Layer Master Design: [0]=Outer Glow, [1]=Mid Shadow, [2]=Crisp Core, [3]=Inner Spine.
    pub layers: [LayerConfig; 4],
}

fn default_lazy_radius() -> f32 {
    30.0
}

fn default_lazy_friction() -> f32 {
    0.4
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
            lazy_enabled: false,
            lazy_radius: 30.0,
            lazy_friction: 0.4,
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

/// Configuration for the velocity-elongated SDF cursor head.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct HeadConfig {
    /// Whether the head shape is rendered at the cursor pointer.
    pub enabled: bool,
    /// Resting diameter in physical pixels.
    pub size: f32,
    /// Elongation along the direction of motion, in percent per unit of eased speed (Windhawk
    /// `squishIntensity`: `scale = min(v × 8, 200) / 15 × intensity / 100`).
    pub squish_intensity: f32,
    /// Percent of the remaining gap the head (position, squish and angle) closes per 1/120 s
    /// (1–100; 100 = locked to the pointer).
    pub squish_smoothing: f32,
    /// RGBA color of the cursor head (straight alpha, 0.0–1.0).
    pub color: [f32; 4],
    /// If true, renders a solid filled ellipse; if false, renders an outlined ring.
    pub filled: bool,
    /// Stroke thickness when `filled` is false (negative value for filled).
    pub thickness: f32,
}

impl Default for HeadConfig {
    fn default() -> Self {
        Self {
            // Trail-only default (mirrors V3): the ribbon is the product; the squishy
            // head blob is opt-in from the Studio.
            enabled: false,
            size: 18.0,
            squish_intensity: 3.0,
            squish_smoothing: 50.0,
            color: [1.0, 1.0, 1.0, 1.0],
            filled: true,
            thickness: -1.0, // <0 filled
        }
    }
}

/// Click-triggered expanding shockwave ripple effects.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RippleConfig {
    /// Whether click ripples are spawned on mouse button presses.
    pub enabled: bool,
    /// Maximum expansion diameter in physical pixels before fading out.
    pub max_diameter: f32,
    /// Lifetime duration of a ripple animation in milliseconds.
    pub duration_ms: u32,
    /// Initial ring stroke width at expansion start.
    pub start_width: f32,
    /// RGBA color for Left mouse button clicks.
    pub color_left: [f32; 4],
    /// RGBA color for Right mouse button clicks.
    pub color_right: [f32; 4],
    /// RGBA color for Middle (wheel) mouse button clicks.
    pub color_middle: [f32; 4],
}

impl Default for RippleConfig {
    fn default() -> Self {
        Self {
            // Trail-only default: click shockwaves are opt-in.
            enabled: false,
            max_diameter: 100.0,
            duration_ms: 600,
            start_width: 8.0,
            color_left: [1.0, 1.0, 1.0, 1.0],
            color_right: [1.0, 0.47, 0.47, 1.0],
            color_middle: [1.0, 1.0, 0.4, 1.0],
        }
    }
}

/// Click-triggered particle burst physics and rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct ParticleConfig {
    /// Whether particle bursts are emitted on mouse clicks.
    pub enabled: bool,
    /// Number of particles spawned per click event.
    pub count_per_click: u32,
    /// Lifetime duration of each particle in milliseconds.
    pub duration_ms: u32,
    /// Initial ejection speed in pixels per second.
    pub base_speed: f32,
    /// Downward gravity acceleration in pixels per second squared.
    pub gravity: f32,
    /// Velocity kept per 1/60 s (0.0–1.0; frame-rate independent via `friction^(dt×60)`).
    pub friction: f32,
    /// Particle diameter in physical pixels.
    pub size: f32,
    /// RGBA tint color for particles.
    pub color: [f32; 4],
}

impl Default for ParticleConfig {
    fn default() -> Self {
        Self {
            // Trail-only default: click particle bursts are opt-in.
            enabled: false,
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

/// Orbiting satellite bodies around the cursor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct SatelliteConfig {
    /// Whether orbiting satellites are active.
    pub enabled: bool,
    /// Number of satellite bodies in the orbit.
    pub count: u32,
    /// Diameter of the circular orbit in physical pixels.
    pub orbit_diameter: f32,
    /// Diameter of each satellite body in physical pixels.
    pub size: f32,
    /// Orbit revolution speed in radians per second.
    pub speed: f32,
    /// If true, spawns a second counter-rotating ring of satellites.
    pub dual_ring: bool,
    /// Whether a faint circular orbit trajectory line is drawn.
    pub show_orbit_ring: bool,
    /// Stroke width of the orbit trajectory ring.
    pub orbit_ring_thickness: f32,
    /// RGBA color of the satellite bodies.
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

/// Dynamic rainbow chromatic hue cycling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct RainbowConfig {
    /// Whether rainbow color cycling replaces static layer colors.
    pub enabled: bool,
    /// Hue advance in degrees per 1/60 s (frame-rate independent; 2 ≈ one cycle every 3 s).
    pub speed: f32,
    /// HSL saturation level (0.0 = grey, 1.0 = full vibrancy).
    pub saturation: f32,
    /// HSL lightness level (0.0 = black, 0.5 = normal, 1.0 = white).
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

/// On-overlay FPS heads-up display (HUD) rendered with a 3×5 bitmap font.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct FpsCounterConfig {
    /// Whether the FPS counter HUD is drawn on the overlay.
    pub enabled: bool,
    /// Update interval for the HUD text in milliseconds. Frame statistics are gathered in
    /// 500 ms windows, so values below 500 behave like 500.
    pub refresh_rate_ms: u32,
    /// If true, aligns HUD to the right side of the screen; left otherwise.
    pub align_right: bool,
    /// If true, aligns HUD to the bottom of the screen; top otherwise.
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

/// GPU-rendered system-cursor bypass: the active OS cursor shape is extracted, drawn on the
/// overlay with optional movement rotation and a click bounce, and the real cursor can be hidden.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GpuCursorConfig {
    /// Master toggle for the GPU-drawn cursor shape.
    pub enabled: bool,
    /// Replace the system arrow with an invisible cursor while the bypass is active.
    /// Restored automatically on disable, exit and panic.
    pub hide_system_cursor: bool,
    /// Rotate the arrow to point along the direction of pointer movement (smoothed).
    pub rotate_with_movement: bool,
    /// Rotation easing time constant in 1/60 s frames (1–30): higher is smoother but lags
    /// further behind direction changes.
    pub rotation_smoothing: f32,
    /// Peak click-bounce scale in percent (100 = no bounce, 150 = 1.5×).
    pub click_scale_percent: f32,
    /// Click-bounce animation duration in milliseconds.
    pub click_scale_duration_ms: u32,
}

impl Default for GpuCursorConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            hide_system_cursor: true,
            rotate_with_movement: true,
            rotation_smoothing: 5.0,
            click_scale_percent: 150.0,
            click_scale_duration_ms: 200,
        }
    }
}

/// General application shell and system integration settings.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct GeneralConfig {
    /// Whether FXCursor launches automatically at operating system login.
    pub autostart: bool,
    /// Whether closing the main Studio window minimizes it to the system tray.
    pub minimize_to_tray: bool,
    /// Whether the Studio window starts hidden in the tray on initial launch.
    pub start_minimized: bool,
    /// Global keyboard shortcut string to toggle effects on/off (e.g. "Ctrl+Shift+E").
    pub global_hotkey: String,
    /// ID of the currently active preset (built-in or user preset).
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

/// Root configuration tree for FXCursor, persisted to `config.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Type)]
pub struct AppConfig {
    /// Master toggle for all overlay cursor effects.
    pub enabled: bool,
    /// Active effect mode gating visual components.
    pub effect_mode: EffectMode,
    /// General application and OS integration settings.
    pub general: GeneralConfig,
    /// Ribbon trail physics and 4-layer master design parameters.
    pub trail: TrailConfig,
    /// Velocity-elongated SDF cursor head settings.
    pub head: HeadConfig,
    /// Click-triggered expanding shockwave ripple settings.
    pub ripple: RippleConfig,
    /// Click-triggered particle burst settings.
    pub particles: ParticleConfig,
    /// Orbiting satellite bodies settings.
    pub satellites: SatelliteConfig,
    /// Rainbow chromatic hue cycling settings.
    pub rainbow: RainbowConfig,
    /// On-overlay FPS HUD counter settings.
    pub fps_counter: FpsCounterConfig,
    /// GPU-rendered system-cursor bypass settings (missing in older config files).
    #[serde(default)]
    pub gpu_cursor: GpuCursorConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            // Trail-only out of the box: the mode mask plus the per-effect `enabled`
            // flags below keep ripples / particles / satellites / head off until chosen.
            effect_mode: EffectMode::Ribbon,
            general: GeneralConfig::default(),
            trail: TrailConfig::default(),
            head: HeadConfig::default(),
            ripple: RippleConfig::default(),
            particles: ParticleConfig::default(),
            satellites: SatelliteConfig::default(),
            rainbow: RainbowConfig::default(),
            fps_counter: FpsCounterConfig::default(),
            gpu_cursor: GpuCursorConfig::default(),
        }
    }
}
