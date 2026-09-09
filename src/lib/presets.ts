/**
 * Configuration types are generated from the Rust structs by tauri-specta into `./bindings`
 * (`src-tauri/src/lib.rs` → `crates/fxcursor-protocol/src/config.rs`). This module only keeps
 * the TypeScript defaults/presets mirror used in browser preview mode; `test/config-parity.test.ts`
 * guards the defaults against drift.
 */
import type { AppConfig } from './bindings';

export type {
  AppConfig,
  LayerConfig,
  EffectMode,
  GeneralConfig,
  TrailConfig,
  HeadConfig,
  RippleConfig,
  ParticleConfig,
  SatelliteConfig,
  RainbowConfig,
  FpsCounterConfig,
} from './bindings';

export function getDefaultConfig(): AppConfig {
  return {
    enabled: true,
    effect_mode: 'FourLayerGlow',
    general: {
      autostart: false,
      minimize_to_tray: true,
      start_minimized: false,
      global_hotkey: 'Ctrl+Shift+E',
      selected_preset: 'master_4layer',
      max_fps: 0,
    },
    trail: {
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
      fade_mode: 3,
      enable_gradient: true,
      adaptive_quality: true,
      layers: [
        {
          enabled: true,
          start_color: [1.0, 1.0, 1.0, 1.0],
          end_color: [1.0, 1.0, 1.0, 0.0],
          width_factor: 1.5,
          alpha_factor: 1.0,
          start_blur: 0.39,
          end_blur: 0.5,
        },
        {
          enabled: true,
          start_color: [0.0, 0.0, 0.0, 1.0],
          end_color: [0.0, 0.0, 0.0, 0.0],
          width_factor: 0.9,
          alpha_factor: 1.0,
          start_blur: 0.1,
          end_blur: 0.1,
        },
        {
          enabled: true,
          start_color: [1.0, 1.0, 1.0, 1.0],
          end_color: [1.0, 1.0, 1.0, 0.0],
          width_factor: 0.5,
          alpha_factor: 1.0,
          start_blur: 0.1,
          end_blur: 0.1,
        },
        {
          enabled: true,
          start_color: [0.0, 0.0, 0.0, 1.0],
          end_color: [0.0, 0.0, 0.0, 1.0],
          width_factor: 0.15,
          alpha_factor: 1.0,
          start_blur: 0.1,
          end_blur: 0.1,
        },
      ],
    },
    head: {
      enabled: true,
      size: 18.0,
      squish_intensity: 3.0,
      squish_smoothing: 50.0,
      color: [1.0, 1.0, 1.0, 1.0],
      filled: true,
      thickness: -1.0,
    },
    ripple: {
      enabled: true,
      max_diameter: 100.0,
      duration_ms: 600,
      start_width: 8.0,
      color_left: [1.0, 1.0, 1.0, 1.0],
      color_right: [1.0, 0.47, 0.47, 1.0],
      color_middle: [1.0, 1.0, 0.4, 1.0],
    },
    particles: {
      enabled: true,
      count_per_click: 16,
      duration_ms: 500,
      base_speed: 300.0,
      gravity: 120.0,
      friction: 0.9,
      size: 3.0,
      color: [1.0, 1.0, 1.0, 1.0],
    },
    satellites: {
      enabled: false,
      count: 3,
      orbit_diameter: 50.0,
      size: 5.0,
      speed: 3.0,
      dual_ring: false,
      show_orbit_ring: true,
      orbit_ring_thickness: 1.0,
      color: [1.0, 1.0, 1.0, 0.8],
    },
    rainbow: {
      enabled: false,
      speed: 2.0,
      saturation: 1.0,
      lightness: 0.5,
    },
    fps_counter: {
      enabled: false,
      refresh_rate_ms: 500,
      align_right: true,
      align_bottom: false,
    },
  };
}

export function getPresetById(id: string): AppConfig | null {
  const base = getDefaultConfig();
  switch (id) {
    case 'master_4layer':
      return base;

    case 'neon_cyberpunk':
      return {
        ...base,
        general: { ...base.general, selected_preset: 'neon_cyberpunk' },
        trail: {
          ...base.trail,
          length: 90,
          spring: 60.0,
          damping: 28.0,
          cursor_size: 44.0,
          layers: [
            {
              enabled: true,
              start_color: [0.0, 0.95, 1.0, 1.0],
              end_color: [0.6, 0.0, 1.0, 0.0],
              width_factor: 1.8,
              alpha_factor: 1.0,
              start_blur: 0.45,
              end_blur: 0.6,
            },
            {
              enabled: true,
              start_color: [0.4, 0.0, 0.9, 1.0],
              end_color: [0.1, 0.0, 0.3, 0.0],
              width_factor: 1.1,
              alpha_factor: 1.0,
              start_blur: 0.15,
              end_blur: 0.15,
            },
            {
              enabled: true,
              start_color: [0.0, 1.0, 0.85, 1.0],
              end_color: [0.0, 0.8, 1.0, 0.0],
              width_factor: 0.55,
              alpha_factor: 1.0,
              start_blur: 0.08,
              end_blur: 0.08,
            },
            {
              enabled: true,
              start_color: [0.05, 0.0, 0.15, 1.0],
              end_color: [0.05, 0.0, 0.15, 1.0],
              width_factor: 0.18,
              alpha_factor: 1.0,
              start_blur: 0.05,
              end_blur: 0.05,
            },
          ],
        },
        head: {
          ...base.head,
          size: 20.0,
          squish_intensity: 3.5,
          color: [0.0, 0.95, 1.0, 1.0],
        },
        ripple: {
          ...base.ripple,
          max_diameter: 120.0,
          color_left: [0.0, 0.95, 1.0, 1.0],
          color_right: [1.0, 0.0, 0.6, 1.0],
          color_middle: [0.6, 0.0, 1.0, 1.0],
        },
        particles: {
          ...base.particles,
          count_per_click: 24,
          base_speed: 350.0,
          color: [0.0, 0.95, 1.0, 1.0],
        },
      };

    case 'razor_spine':
      return {
        ...base,
        general: { ...base.general, selected_preset: 'razor_spine' },
        trail: {
          ...base.trail,
          length: 50,
          spring: 80.0,
          damping: 35.0,
          cursor_size: 28.0,
          fade_mode: 1,
          layers: [
            {
              enabled: false,
              start_color: [1, 1, 1, 1],
              end_color: [1, 1, 1, 0],
              width_factor: 1,
              alpha_factor: 1,
              start_blur: 0.1,
              end_blur: 0.1,
            },
            {
              enabled: true,
              start_color: [0.0, 0.0, 0.0, 0.9],
              end_color: [0.0, 0.0, 0.0, 0.0],
              width_factor: 0.4,
              alpha_factor: 1.0,
              start_blur: 0.04,
              end_blur: 0.04,
            },
            {
              enabled: true,
              start_color: [1.0, 1.0, 1.0, 1.0],
              end_color: [1.0, 1.0, 1.0, 0.0],
              width_factor: 0.22,
              alpha_factor: 1.0,
              start_blur: 0.02,
              end_blur: 0.02,
            },
            {
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
        head: {
          ...base.head,
          size: 14.0,
          squish_intensity: 2.0,
          filled: false,
          thickness: 2.0,
        },
        particles: {
          ...base.particles,
          enabled: false,
        },
      };

    case 'celestial_orbit':
      return {
        ...base,
        general: { ...base.general, selected_preset: 'celestial_orbit' },
        trail: {
          ...base.trail,
          length: 70,
          cursor_size: 36.0,
          layers: [
            {
              enabled: true,
              start_color: [0.9, 0.7, 1.0, 0.9],
              end_color: [0.4, 0.2, 0.8, 0.0],
              width_factor: 1.4,
              alpha_factor: 1.0,
              start_blur: 0.35,
              end_blur: 0.45,
            },
            {
              enabled: true,
              start_color: [0.2, 0.0, 0.4, 0.9],
              end_color: [0.1, 0.0, 0.2, 0.0],
              width_factor: 0.85,
              alpha_factor: 1.0,
              start_blur: 0.1,
              end_blur: 0.1,
            },
            {
              enabled: true,
              start_color: [1.0, 0.95, 0.8, 1.0],
              end_color: [1.0, 0.7, 0.3, 0.0],
              width_factor: 0.45,
              alpha_factor: 1.0,
              start_blur: 0.08,
              end_blur: 0.08,
            },
            {
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
        head: {
          ...base.head,
          size: 16.0,
          color: [1.0, 0.95, 0.8, 1.0],
        },
        satellites: {
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
      };

    case 'particle_firestorm':
      return {
        ...base,
        general: { ...base.general, selected_preset: 'particle_firestorm' },
        trail: {
          ...base.trail,
          length: 85,
          cursor_size: 42.0,
          fade_mode: 2,
          layers: [
            {
              enabled: true,
              start_color: [1.0, 0.4, 0.0, 1.0],
              end_color: [0.8, 0.1, 0.0, 0.0],
              width_factor: 1.6,
              alpha_factor: 1.0,
              start_blur: 0.4,
              end_blur: 0.55,
            },
            {
              enabled: true,
              start_color: [0.3, 0.05, 0.0, 1.0],
              end_color: [0.1, 0.0, 0.0, 0.0],
              width_factor: 0.95,
              alpha_factor: 1.0,
              start_blur: 0.12,
              end_blur: 0.12,
            },
            {
              enabled: true,
              start_color: [1.0, 0.9, 0.2, 1.0],
              end_color: [1.0, 0.3, 0.0, 0.0],
              width_factor: 0.5,
              alpha_factor: 1.0,
              start_blur: 0.1,
              end_blur: 0.1,
            },
            {
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
        head: {
          ...base.head,
          color: [1.0, 0.8, 0.1, 1.0],
        },
        particles: {
          enabled: true,
          count_per_click: 36,
          duration_ms: 650,
          base_speed: 420.0,
          gravity: 180.0,
          friction: 0.91,
          size: 4.0,
          color: [1.0, 0.6, 0.1, 1.0],
        },
      };

    case 'rainbow_aurora':
      return {
        ...base,
        general: { ...base.general, selected_preset: 'rainbow_aurora' },
        rainbow: {
          enabled: true,
          speed: 2.5,
          saturation: 1.0,
          lightness: 0.55,
        },
      };

    default:
      return null;
  }
}

export const BUILTIN_PRESET_IDS = [
  'master_4layer',
  'neon_cyberpunk',
  'razor_spine',
  'celestial_orbit',
  'particle_firestorm',
  'rainbow_aurora',
] as const;

export type BuiltinPresetId = (typeof BUILTIN_PRESET_IDS)[number];

/** Display metadata mirrored from `crates/fxcursor-protocol/src/presets.rs` for browser preview mode. */
export const BUILTIN_PRESET_META: Record<BuiltinPresetId, { name: string; description: string }> = {
  master_4layer: {
    name: 'Master 4-Layer Glow',
    description:
      'Authentic D3D11 master design (Outer Glow + Mid Shadow + Crisp Core + Inner Spine)',
  },
  neon_cyberpunk: {
    name: 'Neon Cyberpunk',
    description:
      'Electric cyan outer glow with deep violet contrast, hot plasma core, and high-velocity sparks',
  },
  razor_spine: {
    name: 'Razor Minimalist Spine',
    description:
      'Ultra-thin, zero-blur high-precision centerline needle for esports and minimalists',
  },
  celestial_orbit: {
    name: 'Celestial Orbit',
    description:
      'Glowing solar core with 4 revolving celestial satellites and gentle lunar ripples',
  },
  particle_firestorm: {
    name: 'Particle Firestorm',
    description: 'Blazing ember trail with dynamic gravity sparks and explosive shockwave bursts',
  },
  rainbow_aurora: {
    name: 'Rainbow Aurora',
    description: 'Continuous HSL spectrum cycling across the 4-layer master ribbon',
  },
};

export const BUILTIN_PRESETS = BUILTIN_PRESET_IDS.map((id) => ({
  id,
  name: BUILTIN_PRESET_META[id].name,
  description: BUILTIN_PRESET_META[id].description,
  config: getPresetById(id)!,
}));
