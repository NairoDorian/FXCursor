import type { AppConfig } from "./lib/bindings";

export function toHex(color: [number, number, number, number]): string {
  const r = Math.round(color[0] * 255)
    .toString(16)
    .padStart(2, "0");
  const g = Math.round(color[1] * 255)
    .toString(16)
    .padStart(2, "0");
  const b = Math.round(color[2] * 255)
    .toString(16)
    .padStart(2, "0");
  return `#${r}${g}${b}`;
}

export function fromHex(hex: string, alpha?: number): [number, number, number, number] {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  return [r, g, b, alpha ?? 1.0];
}

export const defaultConfig: AppConfig = {
  enabled: true,
  effect_type: 0,
  trail_color: [0.0, 0.8, 1.0, 1.0],
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
  fade_mode: 3,
  enable_gradient: true,
  rainbow_mode: false,
  rainbow_speed: 2.0,
  adaptive_quality: true,
  min_trail_width: 2.0,
  velocity_width_multiplier: 0.5,
  velocity_alpha_multiplier: 0.1,
  layers: [
    {
      enabled: true,
      start_color: [0.0, 0.8, 1.0, 1.0],
      end_color: [0.0, 0.8, 1.0, 0.0],
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
      end_color: [0.0, 0.8, 1.0, 0.0],
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
  head_enabled: false,
  head_filled: false,
  head_color: [1.0, 1.0, 1.0, 1.0],
  head_size: 18.0,
  head_outline_width: 2.0,
  head_squish_intensity: 3.0,
  head_squish_smoothing: 50.0,
  ripple_left_color: [1.0, 1.0, 1.0, 1.0],
  ripple_right_color: [1.0, 0.46, 0.46, 1.0],
  ripple_middle_color: [1.0, 1.0, 0.4, 1.0],
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
  satellite_ring_color: [1.0, 1.0, 1.0, 0.27],
  start_minimized: false,
};
