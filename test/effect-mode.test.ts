import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { EFFECT_MODES, MODE_ALL, modeMask } from '../src/lib/effectMode';
import type { EffectMode } from '../src/lib/bindings';

/**
 * `test/fixtures/mode_masks.json` is produced by the Rust renderer (`bun run fixtures` runs
 * `cargo run -p fxcursor-render --example dump_mode_masks`). The Rust integration test
 * `crates/fxcursor-render/tests/mode_mask_fixture.rs` asserts `ModeMask::from_mode` equals it;
 * this test asserts the TypeScript mirror used by the live preview equals it too.
 */
describe('effect mode mask parity with fxcursor-render', () => {
  const fixture: Record<string, ReturnType<typeof modeMask>> = JSON.parse(
    readFileSync(resolve(import.meta.dir, 'fixtures/mode_masks.json'), 'utf8')
  );

  test('fixture covers exactly the modes exposed in the header', () => {
    expect(Object.keys(fixture).toSorted()).toEqual(EFFECT_MODES.map((m) => m.id).toSorted());
  });

  for (const mode of Object.keys(fixture) as EffectMode[]) {
    test(`modeMask('${mode}') matches the Rust renderer`, () => {
      expect(modeMask(mode)).toEqual(fixture[mode]);
    });
  }

  test('full mode enables everything and returns a fresh object', () => {
    const m = modeMask('FourLayerGlow');
    expect(m).toEqual(MODE_ALL);
    expect(m).not.toBe(MODE_ALL);
  });

  test('every mode keeps the head as the cursor anchor', () => {
    for (const { id } of EFFECT_MODES) {
      expect(modeMask(id).head).toBe(true);
    }
  });
});
