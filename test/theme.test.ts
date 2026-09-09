import { describe, it, expect } from 'bun:test';
import { THEME_ACCENTS } from '../src/lib/theme';

describe('FXCursor V4 Theme Accent Suite', () => {
  it('defines 5 curated neon accent themes', () => {
    expect(THEME_ACCENTS.length).toBe(5);
    const ids = THEME_ACCENTS.map((t) => t.id);
    expect(ids).toContain('cyan');
    expect(ids).toContain('violet');
    expect(ids).toContain('solar');
    expect(ids).toContain('emerald');
    expect(ids).toContain('razor');
  });

  it('ensures each theme has valid hex primary color and glow tokens', () => {
    for (const theme of THEME_ACCENTS) {
      expect(theme.primary.startsWith('#')).toBe(true);
      expect(theme.glow.startsWith('rgba')).toBe(true);
      expect(theme.subtle.startsWith('rgba')).toBe(true);
    }
  });
});
