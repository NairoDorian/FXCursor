import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { getDefaultConfig } from '../src/lib/presets';

/**
 * `test/fixtures/default_config.json` is produced by the Rust side (`bun run fixtures`, which runs
 * `cargo run -p fxcursor-protocol --example dump_default`). The Rust integration test
 * `crates/fxcursor-protocol/tests/fixture_parity.rs` asserts `AppConfig::default()` equals it, and
 * this test asserts the TypeScript mirror equals it too, so both sides cannot drift silently.
 */
describe('Rust ⇄ TypeScript default config parity', () => {
  const fixturePath = resolve(import.meta.dir, 'fixtures/default_config.json');
  const fixture = JSON.parse(readFileSync(fixturePath, 'utf8'));

  test('getDefaultConfig() matches the Rust default fixture exactly', () => {
    expect(getDefaultConfig()).toEqual(fixture);
  });

  test('fixture has every top-level section the UI relies on', () => {
    for (const key of [
      'enabled',
      'effect_mode',
      'general',
      'trail',
      'head',
      'ripple',
      'particles',
      'satellites',
      'rainbow',
      'fps_counter',
      'gpu_cursor',
    ]) {
      expect(fixture).toHaveProperty(key);
    }
    expect(fixture.trail.layers).toHaveLength(4);
  });
});
