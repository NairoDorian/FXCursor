/**
 * Helpers for the "reset to default" controls.
 *
 * The Studio offers the same reset three ways: one circular-arrow button per parameter, a
 * "Default" button per section, and one per tab. All of them need the same question answered —
 * "does this still differ from the shipped defaults?" — so the comparison and the deep copy
 * live here.
 *
 * Deep comparison is deliberate: a drag can land on 0.04999999 rather than 0.05, and a section
 * holding any one such value must still read as "modified" or its reset button would be stuck
 * disabled.
 */

import { getDefaultConfig } from './presets';
import type { AppConfig } from './presets';

/** Every section/tab of the config, keyed the way the UI names them. */
export const CONFIG_SECTIONS = [
  'general',
  'trail',
  'head',
  'ripple',
  'particles',
  'satellites',
  'rainbow',
  'fps_counter',
  'gpu_cursor',
] as const;

export type ConfigSection = (typeof CONFIG_SECTIONS)[number];

/** Tolerance for float comparison — smaller than any step the sliders offer. */
const EPSILON = 1e-6;

function sameValue(a: unknown, b: unknown): boolean {
  if (typeof a === 'number' && typeof b === 'number') {
    if (!Number.isFinite(a) || !Number.isFinite(b)) return a === b;
    return Math.abs(a - b) <= EPSILON;
  }
  if (Array.isArray(a) && Array.isArray(b)) {
    if (a.length !== b.length) return false;
    return a.every((item, i) => sameValue(item, b[i]));
  }
  if (a && b && typeof a === 'object' && typeof b === 'object') {
    const left = a as Record<string, unknown>;
    const right = b as Record<string, unknown>;
    const keys = Object.keys(left);
    if (keys.length !== Object.keys(right).length) return false;
    return keys.every((key) => sameValue(left[key], right[key]));
  }
  return a === b;
}

/**
 * True when `value` differs from `defaults`, optionally only looking at `keys` — which is how a
 * section card asks "is *my* part of this section still default?" without the neighbouring
 * sliders deciding the button's state.
 */
export function isModified<T extends object>(
  value: T,
  defaults: T,
  keys?: readonly (keyof T)[]
): boolean {
  if (!keys) return !sameValue(value, defaults);
  return keys.some((key) => !sameValue(value[key], defaults[key]));
}

/** True when one section of the config differs from its shipped defaults. */
export function isSectionModified(config: AppConfig, section: ConfigSection): boolean {
  const defaults = getDefaultConfig();
  const bag = config as unknown as Record<string, unknown>;
  const source = defaults as unknown as Record<string, unknown>;
  return !sameValue(bag[section], source[section]);
}

/** The default value of one parameter, for the per-parameter arrow. */
export function defaultOf<T extends object>(defaults: T, key: keyof T): T[keyof T] {
  return defaults[key];
}

/** A copy of `section` with the listed fields put back to their defaults. */
export function resetFields<T extends object>(section: T, defaults: T, keys: readonly (keyof T)[]): T {
  const next = { ...section };
  for (const key of keys) {
    (next as Record<string, unknown>)[key as string] = structuredClone(
      (defaults as Record<string, unknown>)[key as string]
    );
  }
  return next;
}

/** A copy of `config` with the listed sections put back to their defaults. */
export function resetSections(
  config: AppConfig,
  sections: readonly ConfigSection[]
): AppConfig {
  const defaults = getDefaultConfig();
  const next = structuredClone(config) as AppConfig;
  const bag = next as unknown as Record<string, unknown>;
  const source = defaults as unknown as Record<string, unknown>;
  for (const section of sections) bag[section] = structuredClone(source[section]);
  return next;
}

/** A copy of `config` with the given sections reset, or the whole config when none are given. */
export function resetToDefaults(config: AppConfig, sections?: readonly ConfigSection[]): AppConfig {
  return sections && sections.length > 0
    ? resetSections(config, sections)
    : structuredClone(getDefaultConfig());
}

/** The sections that currently differ from the defaults (drives the "N changed" hint). */
export function modifiedSections(config: AppConfig): ConfigSection[] {
  const defaults = getDefaultConfig();
  const bag = config as unknown as Record<string, unknown>;
  const source = defaults as unknown as Record<string, unknown>;
  return CONFIG_SECTIONS.filter((section) => !sameValue(bag[section], source[section]));
}

/**
 * Which config sections each Studio tab owns, so the per-page reset only touches what is on
 * screen. Tabs that are not parameter pages (presets, dev console, developer, about) own
 * nothing and get no reset button.
 *
 * `layers` is special: the 4-layer design lives inside `trail.layers`, so resetting that page
 * restores only the layer colours, widths and blurs — the physics on the Trail tab survives.
 */
export const TAB_SECTIONS: Record<string, readonly ConfigSection[]> = {
  layers: [],
  trail: ['trail'],
  head: ['head', 'gpu_cursor'],
  ripples: ['ripple'],
  particles: ['particles'],
  satellites: ['satellites'],
  hotkeys: ['general', 'fps_counter'],
  presets: [],
  console: [],
  developer: [],
  about: [],
};

/** True when the layers on the 4-Layer tab differ from their defaults. */
export function areLayersModified(config: AppConfig): boolean {
  return !sameValue(config.trail.layers, getDefaultConfig().trail.layers);
}

/** A copy of `config` with the four ribbon layers put back to their defaults. */
export function resetLayers(config: AppConfig): AppConfig {
  const next = structuredClone(config) as AppConfig;
  next.trail.layers = structuredClone(getDefaultConfig().trail.layers);
  return next;
}
