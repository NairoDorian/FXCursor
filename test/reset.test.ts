import { describe, expect, it } from 'bun:test';
import { getDefaultConfig } from '../src/lib/presets';
import {
  areLayersModified,
  isModified,
  isSectionModified,
  modifiedSections,
  resetFields,
  resetLayers,
  resetSections,
  resetToDefaults,
  TAB_SECTIONS,
} from '../src/lib/reset';

/** Float noise a slider drag can leave behind. */
const almost = (v: number) => v + 5e-7;

describe('reset-to-default helpers', () => {
  const defaults = getDefaultConfig();

  it('a pristine config reads as unmodified', () => {
    expect(modifiedSections(defaults)).toEqual([]);
    expect(isSectionModified(defaults, 'trail')).toBe(false);
    expect(areLayersModified(defaults)).toBe(false);
  });

  it('detects a single edited field in a section', () => {
    const config = structuredClone(defaults);
    config.trail.spring = 0.5;
    expect(isSectionModified(config, 'trail')).toBe(true);
    expect(modifiedSections(config)).toEqual(['trail']);
  });

  it('tolerates float noise so a reset button is not stuck lit', () => {
    const config = structuredClone(defaults);
    config.trail.spring = almost(config.trail.spring);
    expect(isSectionModified(config, 'trail')).toBe(false);
    config.trail.spring = almost(config.trail.spring) + 0.01;
    expect(isSectionModified(config, 'trail')).toBe(true);
  });

  it('isModified only looks at the keys it is given', () => {
    const config = structuredClone(defaults);
    config.trail.spring = 0.5;
    // The spring is not one of the lazy-brush keys, so the Lazy Brush card stays clean.
    expect(isModified(config.trail, defaults.trail, ['lazy_radius', 'lazy_friction'])).toBe(false);
    expect(isModified(config.trail, defaults.trail, ['spring'])).toBe(true);
  });

  it('resetFields restores only the listed fields', () => {
    const config = structuredClone(defaults);
    config.trail.spring = 0.9;
    config.trail.damping = 0.1;
    const next = resetFields(config.trail, defaults.trail, ['spring']);
    expect(next.spring).toBe(defaults.trail.spring);
    expect(next.damping).toBe(0.1);
  });

  it('resetSections restores whole sections and leaves the rest alone', () => {
    const config = structuredClone(defaults);
    config.trail.spring = 0.9;
    config.ripple.start_width = 20;
    const next = resetSections(config, ['ripple']);
    expect(next.ripple).toEqual(defaults.ripple);
    expect(next.trail.spring).toBe(0.9);
  });

  it('resetToDefaults with no sections returns a pristine copy', () => {
    const config = structuredClone(defaults);
    config.trail.spring = 0.9;
    const next = resetToDefaults(config);
    expect(next).toEqual(defaults);
    expect(next).not.toBe(config);
  });

  it('resetLayers restores the ribbon layers but not the trail physics', () => {
    const config = structuredClone(defaults);
    config.trail.layers[0].width_factor = 3;
    config.trail.spring = 0.4;
    const next = resetLayers(config);
    expect(next.trail.layers).toEqual(defaults.trail.layers);
    expect(next.trail.spring).toBe(0.4);
  });

  it('every parameter tab owns at least one section and non-parameter tabs own none', () => {
    for (const [tab, sections] of Object.entries(TAB_SECTIONS)) {
      const expectSections = ['presets', 'console', 'developer', 'about', 'layers'].includes(tab)
        ? 0
        : sections.length;
      expect(sections.length).toBe(expectSections);
    }
  });
});
