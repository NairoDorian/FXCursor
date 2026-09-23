/**
 * TypeScript mirror of the trail math in `crates/fxcursor-render/src/renderer.rs`, used by the
 * Studio live preview. Everything here is pure (no DOM) so `test/trail-parity.test.ts` can replay
 * the Rust reference trace (`test/fixtures/trail_trace.json`, written by `bun run fixtures`) and
 * fail on any drift. Change the physics on one side ⇒ change it on the other and regenerate.
 *
 * Units follow Windhawk: positions in overlay pixels, velocities in px per 1/120 s reference
 * frame, springs `k = setting / 1000`, friction `1 − percent / 100` raised to `dt_scale`.
 */
import type { AppConfig, LayerConfig } from './presets';

type TrailSettings = AppConfig['trail'];
type HeadSettings = AppConfig['head'];
export type Rgba = [number, number, number, number];

/** Below this speed (px per reference frame) a node counts as at rest. */
export const MIN_DIRECTION_SPEED = 0.05;
/** The head counts as arrived when it is this close (px) to the brush. */
export const HEAD_REST_DISTANCE = 0.1;
/** Teleport guard: the largest gap between consecutive nodes. */
export const MAX_NODE_GAP = 512;
/** Windhawk `kReferenceFrameTime`. */
export const REFERENCE_FRAME = 1 / 120;
export const PHYSICS_TICK = 1 / 120;
export const MAX_SUBSTEPS = 16;
export const MAX_FRAME_DELTA = 0.1;
/** Nodes closer than this are merged before splining. */
export const MIN_NODE_SPACING = 0.25;
export const MIN_SAMPLE_SPACING = 3;
export const MAX_SAMPLE_SPACING = 24;
export const MIN_VISIBLE_ALPHA = 0.004;

const clamp = (v: number, lo: number, hi: number) => Math.min(hi, Math.max(lo, v));

export interface TrailNode {
  x: number;
  y: number;
  vx: number;
  vy: number;
  speed: number;
}

/** TD LazyBrush friction factor: 0 → 1 (snap), ≥ 1 → 0 (frozen). */
export function lazyBrushFactor(friction: number): number {
  if (friction <= 0) return 1;
  if (friction >= 1) return 0;
  const u = 1 - friction;
  return 1 - Math.sqrt(1 - u * u);
}

/** Mirrors `TrailChain` (renderer.rs): LazyBrush → head spring-damper → body chain → clamp. */
export class TrailChain {
  nodes: TrailNode[] = [];
  brush = { x: 0, y: 0 };
  private brushValid = false;
  private brushMoving = false;

  /** Grows from the tail (new nodes start on the last node, or on the pointer) or truncates. */
  resize(len: number, x: number, y: number): void {
    if (this.nodes.length > len) this.nodes.length = len;
    while (this.nodes.length < len) {
      const seed = this.nodes[this.nodes.length - 1] ?? { x, y };
      this.nodes.push({ x: seed.x, y: seed.y, vx: 0, vy: 0, speed: 0 });
    }
  }

  reset(): void {
    this.nodes.length = 0;
    this.brushValid = false;
    this.brushMoving = false;
  }

  isMoving(): boolean {
    const head = this.nodes[0];
    const headAway =
      head !== undefined &&
      (Math.abs(head.x - this.brush.x) > HEAD_REST_DISTANCE ||
        Math.abs(head.y - this.brush.y) > HEAD_REST_DISTANCE);
    return this.brushMoving || headAway || this.nodes.some((n) => n.speed > MIN_DIRECTION_SPEED);
  }

  advance(x: number, y: number, dt: number, trail: TrailSettings): void {
    if (this.nodes.length === 0) return;
    if (!this.brushValid) {
      this.brush = { x, y };
      this.brushValid = true;
    }
    if (!trail.lazy_enabled) this.brush = { x, y };
    const substeps = clamp(Math.ceil(dt / PHYSICS_TICK - 1e-3), 1, MAX_SUBSTEPS);
    const sdt = dt / substeps;
    this.brushMoving = false;
    for (let i = 0; i < substeps; i++) this.step(x, y, sdt, trail);
    if (!this.nodes.every((n) => Number.isFinite(n.x) && Number.isFinite(n.y) && Number.isFinite(n.speed))) {
      const len = this.nodes.length;
      this.reset();
      this.resize(len, x, y);
    }
  }

  private step(x: number, y: number, sdt: number, trail: TrailSettings): void {
    const dtScale = clamp(sdt / REFERENCE_FRAME, 0.01, 5);

    // 1. LazyBrush dead zone.
    if (trail.lazy_enabled) {
      const dx = x - this.brush.x;
      const dy = y - this.brush.y;
      const dist = Math.hypot(dx, dy);
      const excess = dist - (trail.lazy_radius ?? 30);
      if (dist > 1e-4 && Math.round(excess * 10) / 10 > 0) {
        const f = lazyBrushFactor(trail.lazy_friction ?? 0.4);
        const factor = 1 - Math.pow(1 - f, dtScale);
        const pull = excess * factor;
        this.brush.x += (dx / dist) * pull;
        this.brush.y += (dy / dist) * pull;
        this.brushMoving = true;
      }
    } else {
      this.brush = { x, y };
    }

    const headSpring = trail.head_spring / 1000;
    const headFric = Math.pow(clamp(1 - trail.head_damping / 100, 0.01, 1), dtScale);
    const bodySpring = trail.spring / 1000;
    const bodyFric = Math.pow(clamp(1 - trail.damping / 100, 0.01, 1), dtScale);

    // 2. Head: spring-damper toward the brush.
    const head = this.nodes[0];
    head.vx += (this.brush.x - head.x) * headSpring * dtScale;
    head.vy += (this.brush.y - head.y) * headSpring * dtScale;
    head.vx *= headFric;
    head.vy *= headFric;
    head.x += head.vx * dtScale;
    head.y += head.vy * dtScale;
    head.speed = Math.hypot(head.vx, head.vy);

    // 3. Body chain with 0.3 second-neighbour coupling (forward Gauss-Seidel).
    for (let i = 1; i < this.nodes.length; i++) {
      const cur = this.nodes[i];
      const prev = this.nodes[i - 1];
      if (i > 1) {
        const pp = this.nodes[i - 2];
        cur.vx += (pp.x - cur.x) * bodySpring * 0.3 * dtScale;
        cur.vy += (pp.y - cur.y) * bodySpring * 0.3 * dtScale;
      }
      cur.vx += (prev.x - cur.x) * bodySpring * dtScale;
      cur.vy += (prev.y - cur.y) * bodySpring * dtScale;
      cur.vx *= bodyFric;
      cur.vy *= bodyFric;
      cur.x += cur.vx * dtScale;
      cur.y += cur.vy * dtScale;
      cur.speed = Math.hypot(cur.vx, cur.vy);
    }

    // 4. Inelastic teleport guard.
    for (let i = 1; i < this.nodes.length; i++) {
      const cur = this.nodes[i];
      const prev = this.nodes[i - 1];
      const dx = cur.x - prev.x;
      const dy = cur.y - prev.y;
      const dist = Math.hypot(dx, dy);
      if (dist > MAX_NODE_GAP) {
        const ratio = MAX_NODE_GAP / dist;
        const nx = dx / dist;
        const ny = dy / dist;
        const vn = cur.vx * nx + cur.vy * ny;
        if (vn > 0) {
          cur.vx -= vn * nx;
          cur.vy -= vn * ny;
        }
        cur.x = prev.x + dx * ratio;
        cur.y = prev.y + dy * ratio;
        cur.speed = Math.hypot(cur.vx, cur.vy);
      }
    }
  }
}

export interface Sample {
  x: number;
  y: number;
  speed: number;
  /** 0 at the head, 1 at the tail — by original node index, as in every legacy build. */
  progress: number;
}

type Pt = [number, number];

/** Centripetal Catmull-Rom (α = 0.5, Barry–Goldman form) between p1 and p2. */
export function catmullRomCentripetal(p0: Pt, p1: Pt, p2: Pt, p3: Pt, t: number): Pt {
  const knot = (a: Pt, b: Pt) => Math.sqrt(Math.sqrt(Math.max(1e-6, (b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2)));
  const lerp = (a: Pt, b: Pt, wa: number, wb: number): Pt => [a[0] * wa + b[0] * wb, a[1] * wa + b[1] * wb];
  const t0 = 0;
  const t1 = t0 + knot(p0, p1);
  const t2 = t1 + knot(p1, p2);
  const t3 = t2 + knot(p2, p3);
  const tt = t1 + (t2 - t1) * clamp(t, 0, 1);
  const a1 = lerp(p0, p1, (t1 - tt) / (t1 - t0), (tt - t0) / (t1 - t0));
  const a2 = lerp(p1, p2, (t2 - tt) / (t2 - t1), (tt - t1) / (t2 - t1));
  const a3 = lerp(p2, p3, (t3 - tt) / (t3 - t2), (tt - t2) / (t3 - t2));
  const b1 = lerp(a1, a2, (t2 - tt) / (t2 - t0), (tt - t0) / (t2 - t0));
  const b2 = lerp(a2, a3, (t3 - tt) / (t3 - t1), (tt - t1) / (t3 - t1));
  return lerp(b1, b2, (t2 - tt) / (t2 - t1), (tt - t1) / (t2 - t1));
}

/** Absolute change of direction (radians) at `b` for the polyline a → b → c. */
function turningAngle(a: Pt, b: Pt, c: Pt): number {
  const ux = b[0] - a[0];
  const uy = b[1] - a[1];
  const vx = c[0] - b[0];
  const vy = c[1] - b[1];
  const lu = Math.hypot(ux, uy);
  const lv = Math.hypot(vx, vy);
  if (lu < 1e-4 || lv < 1e-4) return 0;
  return Math.acos(clamp((ux * vx + uy * vy) / (lu * lv), -1, 1));
}

/** Mirrors `build_samples`: merge → phantom endpoints → centripetal spline → index progress. */
export function buildSamples(nodes: readonly { x: number; y: number; speed: number }[], trail: TrailSettings): Sample[] {
  const out: Sample[] = [];
  const total = nodes.length;
  if (total === 0) return out;
  const indexScale = 1 / Math.max(1, total - 1);

  // [x, y, speed, original index]
  const pts: [number, number, number, number][] = [];
  nodes.forEach((n, i) => {
    const last = pts[pts.length - 1];
    if (last) {
      const dx = n.x - last[0];
      const dy = n.y - last[1];
      if (dx * dx + dy * dy < MIN_NODE_SPACING * MIN_NODE_SPACING) return;
    }
    pts.push([n.x, n.y, n.speed, i]);
  });

  const n = pts.length;
  if (n === 1) {
    out.push({ x: pts[0][0], y: pts[0][1], speed: pts[0][2], progress: 0 });
    return out;
  }
  const first = pts[0];
  const second = pts[1];
  const last = pts[n - 1];
  const beforeLast = pts[n - 2];
  const phantomStart: [number, number, number, number] = [2 * first[0] - second[0], 2 * first[1] - second[1], first[2], first[3]];
  const phantomEnd: [number, number, number, number] = [2 * last[0] - beforeLast[0], 2 * last[1] - beforeLast[1], last[2], last[3]];
  const ctrl = (i: number) => (i < 0 ? phantomStart : i >= n ? phantomEnd : pts[i]);
  const baseSteps = clamp(trail.interpolation_steps, 1, 32);

  for (let seg = 0; seg < n - 1; seg++) {
    const c0 = ctrl(seg - 1);
    const c1 = ctrl(seg);
    const c2 = ctrl(seg + 1);
    const c3 = ctrl(seg + 2);
    const p0: Pt = [c0[0], c0[1]];
    const p1: Pt = [c1[0], c1[1]];
    const p2: Pt = [c2[0], c2[1]];
    const p3: Pt = [c3[0], c3[1]];
    const segDist = Math.hypot(p2[0] - p1[0], p2[1] - p1[1]);
    let steps = baseSteps;
    if (trail.adaptive_quality) {
      const turn = turningAngle(p0, p1, p2) + turningAngle(p1, p2, p3);
      const radius = segDist / Math.max(turn, 1e-3);
      const spacing = clamp(Math.sqrt(4 * radius), MIN_SAMPLE_SPACING, MAX_SAMPLE_SPACING);
      steps = clamp(Math.ceil(segDist / spacing), 1, 32);
    }
    for (let step = 0; step < steps; step++) {
      const t = step / steps;
      const [x, y] = catmullRomCentripetal(p0, p1, p2, p3, t);
      out.push({ x, y, speed: c1[2] + (c2[2] - c1[2]) * t, progress: (c1[3] + (c2[3] - c1[3]) * t) * indexScale });
    }
  }
  out.push({ x: last[0], y: last[1], speed: last[2], progress: last[3] * indexScale });
  return out;
}

/** 0 Linear, 1 Ease Out, 2 Exponential, 3 Sigmoid, 4 Smoothstep (`apply_fade_curve`). */
export function fadeCurve(progress: number, mode: number): number {
  const p = clamp(progress, 0, 1);
  switch (mode) {
    case 1:
      return 1 - p * p;
    case 2:
      return Math.exp(-p * 3);
    case 3:
      return 1 / (1 + Math.exp(8 * (p - 0.5)));
    case 4:
      return 1 - p * p * (3 - 2 * p);
    default:
      return 1 - p;
  }
}

export function lerpRgba(a: Rgba, b: Rgba, t: number): Rgba {
  const k = clamp(t, 0, 1);
  return [a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, a[2] + (b[2] - a[2]) * k, a[3] + (b[3] - a[3]) * k];
}

/** Straight RGBA and radius of one sample for one layer (`layer_sample_style`). */
export function layerSampleStyle(
  s: Sample,
  layer: LayerConfig,
  trail: TrailSettings,
  startC: Rgba,
  endC: Rgba,
  gradient: boolean
): { color: Rgba; radius: number } {
  const fade = fadeCurve(s.progress, trail.fade_mode);
  // px per 1/120 s, normalised by 20 like Windhawk / V3 (saturates at 2400 px/s).
  const normSpeed = Math.min(s.speed / 20, 1);
  const velWidth = 1 + normSpeed * trail.velocity_width_mult;
  const velAlpha = 1 + normSpeed * trail.velocity_alpha_mult;
  const width = Math.max(trail.min_width, trail.cursor_size * layer.width_factor * fade * velWidth);
  const c = gradient ? lerpRgba(startC, endC, s.progress) : ([...startC] as Rgba);
  c[3] = clamp(c[3] * layer.alpha_factor * fade * velAlpha, 0, 1);
  return { color: c, radius: width * 0.5 };
}

/** Windhawk `UpdateSquishyCursor` (`SquishyState::step` in Rust). */
export class SquishyHead {
  x = 0;
  y = 0;
  private prevX = 0;
  private prevY = 0;
  scale = 0;
  private targetScale = 0;
  angle = 0;
  private targetAngle = 0;
  private initialized = false;

  step(x: number, y: number, dtScale: number, head: HeadSettings): void {
    if (!this.initialized) {
      Object.assign(this, { x, y, prevX: x, prevY: y, scale: 0, targetScale: 0, angle: 0, targetAngle: 0 });
      this.initialized = true;
    }
    const smoothing = clamp(head.squish_smoothing / 100, 0.01, 1);
    const adaptive = 1 - Math.pow(1 - smoothing, dtScale);
    this.x += (x - this.x) * adaptive;
    this.y += (y - this.y) * adaptive;
    const dx = this.x - this.prevX;
    const dy = this.y - this.prevY;
    const velocity = Math.hypot(dx, dy) / dtScale;
    this.prevX = this.x;
    this.prevY = this.y;
    this.targetScale = (Math.min(velocity * 8, 200) / 15) * (head.squish_intensity / 100);
    this.scale += (this.targetScale - this.scale) * adaptive;
    if (velocity > 0.5) this.targetAngle = Math.atan2(dy, dx);
    const tau = Math.PI * 2;
    const diff = ((((this.targetAngle - this.angle + Math.PI) % tau) + tau) % tau) - Math.PI;
    this.angle += diff * adaptive;
  }

  reset(): void {
    this.initialized = false;
  }
}
