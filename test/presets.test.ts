import { describe, it, expect } from 'bun:test';
import { BUILTIN_PRESETS, getDefaultConfig, getPresetById } from '../src/lib/presets';

describe('FXCursor V4 Presets Integrity Suite', () => {
  it('loads default config with valid 4-layer master design', () => {
    const def = getDefaultConfig();
    expect(def.enabled).toBe(true);
    expect(def.trail.layers.length).toBe(4);

    // Out of the box only the ribbon trail runs (legacy V3 / user default).
    expect(def.effect_mode).toBe('Ribbon');
    expect(def.trail.enabled).toBe(true);
    expect(def.head.enabled).toBe(false);
    expect(def.ripple.enabled).toBe(false);
    expect(def.particles.enabled).toBe(false);
    expect(def.satellites.enabled).toBe(false);

    // Layer 1: Outer Glow (150% width)
    expect(def.trail.layers[0].width_factor).toBe(1.5);
    expect(def.trail.layers[0].start_blur).toBe(0.39);
    expect(def.trail.layers[0].end_blur).toBe(0.5);

    // Layer 2: Mid Shadow (90% width)
    expect(def.trail.layers[1].width_factor).toBe(0.9);

    // Layer 3: Core (50% width)
    expect(def.trail.layers[2].width_factor).toBe(0.5);

    // Layer 4: Inner Spine (15% width)
    expect(def.trail.layers[3].width_factor).toBe(0.15);
  });

  it('contains all 6 curated master presets', () => {
    expect(BUILTIN_PRESETS.length).toBeGreaterThanOrEqual(6);
    const ids = BUILTIN_PRESETS.map((p) => p.id);
    expect(ids).toContain('master_4layer');
    expect(ids).toContain('neon_cyberpunk');
    expect(ids).toContain('razor_spine');
    expect(ids).toContain('celestial_orbit');
    expect(ids).toContain('particle_firestorm');
    expect(ids).toContain('rainbow_aurora');
  });

  it('carries each config exactly as the backend applies it', () => {
    for (const p of BUILTIN_PRESETS) {
      expect(p.config.general.selected_preset).toBe(p.id);
    }
    // getPresetById hands out copies: editing one must not corrupt the shared list.
    const copy = getPresetById('neon_cyberpunk')!;
    copy.trail.length = 1;
    expect(getPresetById('neon_cyberpunk')!.trail.length).not.toBe(1);
  });

  it('retrieves presets correctly by ID', () => {
    const master = getPresetById('master_4layer');
    expect(master).toBeDefined();
    expect(master?.trail.length).toBeGreaterThan(0);

    const nonExistent = getPresetById('unknown_preset_123');
    expect(nonExistent).toBeNull();
  });
});
