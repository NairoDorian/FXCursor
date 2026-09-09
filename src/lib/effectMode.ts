import type { EffectMode } from './bindings';

/**
 * Which subsystems an effect mode allows. Mirrors `ModeMask::from_mode` in
 * `crates/fxcursor-render/src/renderer.rs`; `test/effect-mode.test.ts` keeps the two in step.
 * Individual `enabled` flags in the config still apply on top of the mask.
 */
export interface ModeMask {
  trail: boolean;
  /** Per-layer gate for the 4-layer ribbon (Outer Glow, Mid Shadow, Crisp Core, Inner Spine). */
  layers: [boolean, boolean, boolean, boolean];
  head: boolean;
  ripples: boolean;
  particles: boolean;
  satellites: boolean;
}

export const MODE_ALL: ModeMask = Object.freeze({
  trail: true,
  layers: [true, true, true, true],
  head: true,
  ripples: true,
  particles: true,
  satellites: true,
}) as ModeMask;

export function modeMask(mode: EffectMode): ModeMask {
  switch (mode) {
    // Trail and head only: no click effects, no orbitals.
    case 'Ribbon':
      return { ...MODE_ALL, ripples: false, particles: false, satellites: false };
    // Click feedback only: head, ripples and particle bursts.
    case 'ParticlesOnly':
      return { ...MODE_ALL, trail: false, satellites: false };
    // Orbitals only: head and satellites.
    case 'SatellitesOnly':
      return { ...MODE_ALL, trail: false, ripples: false, particles: false };
    // Minimal: crisp core + inner spine with the head; no glow/shadow, no extras.
    case 'Minimal':
      return {
        ...MODE_ALL,
        layers: [false, false, true, true],
        ripples: false,
        particles: false,
        satellites: false,
      };
    case 'FourLayerGlow':
    default:
      return { ...MODE_ALL };
  }
}

/** Effect modes exposed in the Studio header, in display order. */
export const EFFECT_MODES: ReadonlyArray<{ id: EffectMode; label: string; hint: string }> = [
  { id: 'FourLayerGlow', label: 'Full (4-layer glow)', hint: 'Everything the tabs enable.' },
  { id: 'Ribbon', label: 'Ribbon only', hint: 'Trail and head; no click effects or satellites.' },
  { id: 'ParticlesOnly', label: 'Click effects only', hint: 'Head, ripples and particles.' },
  { id: 'SatellitesOnly', label: 'Satellites only', hint: 'Head and orbiting satellites.' },
  { id: 'Minimal', label: 'Minimal (core + spine)', hint: 'Crisp core and inner spine only.' },
];
