import { describe, expect, test } from 'bun:test';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { getDefaultConfig } from '../src/lib/presets';
import { buildSamples, TrailChain } from '../src/lib/trail';

/**
 * `test/fixtures/trail_trace.json` is recorded from the Rust `TrailChain` + `build_samples`
 * (`bun run fixtures` → `cargo run -p fxcursor-render --example dump_trail_trace`). Replaying the
 * same scripted pointer through `src/lib/trail.ts` must land on the same nodes and samples, so the
 * Studio live preview cannot silently drift from what the overlay draws.
 *
 * Rust integrates in f32, TypeScript in f64: positions are compared to a small tolerance.
 */

interface Checkpoint {
  frame: number;
  brush: [number, number];
  nodes: [number, number, number][];
  samples: [number, number, number][];
  moving: boolean;
}
interface Run {
  name: string;
  fps: number;
  frames: number;
  checkpoints: Checkpoint[];
}
interface Trace {
  config_overrides: Record<string, Record<string, unknown>>;
  runs: Run[];
}

const trace: Trace = JSON.parse(readFileSync(resolve(import.meta.dir, 'fixtures/trail_trace.json'), 'utf8'));

/** Same path as `pointer()` in `examples/dump_trail_trace.rs`. */
function pointer(frame: number): [number, number] {
  if (frame < 60) return [100 + frame * 12, 300 + Math.sin(frame * 0.15) * 60];
  if (frame < 90) {
    const [x0, y0] = pointer(59);
    return [x0 - (frame - 59) * 9, y0 + (frame - 59) * 4];
  }
  return pointer(89);
}

const POS_TOLERANCE = 0.05; // px
const PROGRESS_TOLERANCE = 1e-3;

describe('Rust ⇄ TypeScript trail parity', () => {
  test('the fixture covers the default, 144 Hz, LazyBrush and fixed-step paths', () => {
    expect(trace.runs.map((r) => r.name)).toEqual(['default', 'default_144hz', 'lazy', 'short_fixed_steps']);
  });

  for (const run of trace.runs) {
    test(`${run.name}: nodes, brush and samples match the Rust trace`, () => {
      const cfg = getDefaultConfig();
      const overrideKey = run.name === 'default_144hz' ? 'default' : run.name;
      Object.assign(cfg.trail, trace.config_overrides[overrideKey]);

      const chain = new TrailChain();
      const byFrame = new Map(run.checkpoints.map((c) => [c.frame, c]));
      let worstNode = 0;
      let comparedSampleSets = 0;
      for (let frame = 0; frame < run.frames; frame++) {
        const [x, y] = pointer(frame);
        chain.resize(cfg.trail.length, x, y);
        chain.advance(x, y, 1 / run.fps, cfg.trail);
        const expected = byFrame.get(frame);
        if (!expected) continue;

        expect(chain.nodes.length).toBe(expected.nodes.length);
        expected.nodes.forEach(([ex, ey, es], i) => {
          const n = chain.nodes[i];
          worstNode = Math.max(worstNode, Math.abs(n.x - ex), Math.abs(n.y - ey), Math.abs(n.speed - es));
        });
        expect(Math.abs(chain.brush.x - expected.brush[0])).toBeLessThan(POS_TOLERANCE);
        expect(Math.abs(chain.brush.y - expected.brush[1])).toBeLessThan(POS_TOLERANCE);
        expect(chain.isMoving()).toBe(expected.moving);

        // Sample counts come from `ceil()` of spline lengths; an f32/f64 difference can flip a
        // count right at a boundary, so values are compared whenever the counts agree.
        const samples = buildSamples(chain.nodes, cfg.trail);
        if (samples.length === expected.samples.length) {
          comparedSampleSets++;
          expected.samples.forEach(([sx, sy, sp], i) => {
            expect(Math.abs(samples[i].x - sx)).toBeLessThan(POS_TOLERANCE);
            expect(Math.abs(samples[i].y - sy)).toBeLessThan(POS_TOLERANCE);
            expect(Math.abs(samples[i].progress - sp)).toBeLessThan(PROGRESS_TOLERANCE);
          });
        } else {
          expect(Math.abs(samples.length - expected.samples.length)).toBeLessThanOrEqual(2);
        }
      }
      expect(worstNode).toBeLessThan(POS_TOLERANCE);
      expect(comparedSampleSets).toBeGreaterThanOrEqual(run.checkpoints.length - 1);
    });
  }
});

describe('trail behaviour (TypeScript mirror)', () => {
  test('a collapsed chain yields one sample: the resting dot', () => {
    const cfg = getDefaultConfig();
    const nodes = Array.from({ length: 40 }, () => ({ x: 10, y: 20, speed: 0 }));
    expect(buildSamples(nodes, cfg.trail)).toHaveLength(1);
  });

  test('progress follows the original node index through merges', () => {
    const cfg = getDefaultConfig();
    cfg.trail.adaptive_quality = false;
    cfg.trail.interpolation_steps = 1;
    const nodes = [
      ...Array.from({ length: 5 }, () => ({ x: 0, y: 0, speed: 0 })),
      ...[1, 2, 3, 4, 5].map((i) => ({ x: i * 10, y: 0, speed: 5 })),
    ];
    const at10 = buildSamples(nodes, cfg.trail).find((s) => Math.abs(s.x - 10) < 1e-6);
    expect(at10?.progress).toBeCloseTo(5 / 9, 6);
  });
});
