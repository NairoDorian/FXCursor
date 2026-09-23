/**
 * Configuration types are generated from the Rust structs by tauri-specta into `./bindings`
 * (`src-tauri/src/lib.rs` → `crates/fxcursor-protocol/src/config.rs`). This module only keeps
 * the TypeScript defaults mirror used in browser preview mode (`test/config-parity.test.ts` guards
 * it against drift) and re-exports the built-in presets generated from Rust.
 */
import type { AppConfig } from './bindings';
import builtinPresets from './generated/builtin_presets.json';

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
  GpuCursorConfig,
} from './bindings';

export function getDefaultConfig(): AppConfig {
  return {
    enabled: true,
    effect_mode: 'Ribbon',
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
      // Trail-only default (mirrors Rust / V3): squishy head is opt-in.
      enabled: false,
      size: 18.0,
      squish_intensity: 3.0,
      squish_smoothing: 50.0,
      color: [1.0, 1.0, 1.0, 1.0],
      filled: true,
      thickness: -1.0,
    },
    ripple: {
      // Trail-only default: click shockwaves are opt-in.
      enabled: false,
      max_diameter: 100.0,
      duration_ms: 600,
      start_width: 8.0,
      color_left: [1.0, 1.0, 1.0, 1.0],
      color_right: [1.0, 0.47, 0.47, 1.0],
      color_middle: [1.0, 1.0, 0.4, 1.0],
    },
    particles: {
      // Trail-only default: click particle bursts are opt-in.
      enabled: false,
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
    gpu_cursor: {
      enabled: false,
      hide_system_cursor: true,
      rotate_with_movement: true,
      rotation_smoothing: 5.0,
      click_scale_percent: 150.0,
      click_scale_duration_ms: 200,
    },
  };
}

/** A built-in preset exactly as the Rust backend lists and applies it. */
export interface BuiltinPreset {
  id: string;
  name: string;
  description: string;
  config: AppConfig;
}

/**
 * Built-in presets, generated from `crates/fxcursor-protocol/src/presets.rs` by
 * `bun run fixtures` (`dump_presets` example). Used in browser preview mode and for the
 * "MODIFIED" badge; under Tauri the list comes from the `list_presets` command. Never edit the
 * JSON by hand: change the Rust presets and regenerate.
 */
export const BUILTIN_PRESETS: readonly BuiltinPreset[] = builtinPresets as BuiltinPreset[];

/** A deep copy of a built-in preset's config, or `null` for an unknown id. */
export function getPresetById(id: string): AppConfig | null {
  const preset = BUILTIN_PRESETS.find((p) => p.id === id);
  return preset ? structuredClone(preset.config) : null;
}
