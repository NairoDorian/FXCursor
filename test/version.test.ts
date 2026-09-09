import { describe, it, expect } from 'bun:test';
import { APP_VERSION } from '../scripts/version';
import pkg from '../package.json';

describe('FXCursor V4 Version Consistency Suite', () => {
  it('validates that APP_VERSION matches SemVer format', () => {
    const semverRegex = /^\d+\.\d+\.\d+(-[0-9A-Za-z.-]+)?$/;
    expect(semverRegex.test(APP_VERSION)).toBe(true);
  });

  it('validates that package.json version matches scripts/version.ts', () => {
    expect(pkg.version).toBe(APP_VERSION);
  });
});
