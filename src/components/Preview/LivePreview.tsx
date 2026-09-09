import { Component, createSignal, onSettled } from 'solid-js';
import type { AppConfig, LayerConfig } from '../../lib/presets';
import { modeMask } from '../../lib/effectMode';

/**
 * 2D-canvas mirror of `crates/fxcursor-render`. The physics constants, fade curves, width /
 * alpha modulation, rainbow and effect-mode gating follow the Rust renderer so what you see here
 * is what the overlay draws (minus GPU anti-aliasing). Scene is scaled by `PREVIEW_SCALE` to fit.
 */
interface LivePreviewProps {
  config: () => AppConfig;
}

interface Node {
  x: number;
  y: number;
  vx: number;
  vy: number;
  speed: number;
  /** Unit direction of the last significant motion (0,0 until the node has moved). */
  dx: number;
  dy: number;
}

/** The line through a node perpendicular to its direction of motion; successors may not cross it. */
interface Wall {
  x: number;
  y: number;
  ux: number;
  uy: number;
}

/**
 * Keeps `node` from overtaking its predecessor: a step that crossed the wall from behind to
 * ahead is clamped back onto it and the forward part of the velocity is dropped. A node that
 * was already ahead (the pointer just reversed into the trail) is a legitimate hairpin and is
 * left alone. Mirrors `block_overtake` in the Rust renderer.
 */
function blockOvertake(fromX: number, fromY: number, node: Node, wall: Wall) {
  const before = (fromX - wall.x) * wall.ux + (fromY - wall.y) * wall.uy;
  const after = (node.x - wall.x) * wall.ux + (node.y - wall.y) * wall.uy;
  if (before <= 0 && after > 0) {
    const pushBack = after + 1e-4;
    node.x -= wall.ux * pushBack;
    node.y -= wall.uy * pushBack;
    const vAlong = node.vx * wall.ux + node.vy * wall.uy;
    if (vAlong > 0) {
      node.vx -= wall.ux * vAlong;
      node.vy -= wall.uy * vAlong;
    }
  }
}

/** Refreshes the cached speed and, if the node moved, its direction of motion. */
function finishStep(node: Node, fromX: number, fromY: number) {
  node.speed = Math.hypot(node.vx, node.vy);
  const len = Math.hypot(node.x - fromX, node.y - fromY);
  if (len >= 1e-3) {
    node.dx = (node.x - fromX) / len;
    node.dy = (node.y - fromY) / len;
  }
}

/** First-order pursuit toward a target: never overshoots; velocity from the actual displacement. */
function followStep(node: Node, tx: number, ty: number, a: number, wall: Wall | null, dtScale: number) {
  const fromX = node.x;
  const fromY = node.y;
  node.x += (tx - node.x) * a;
  node.y += (ty - node.y) * a;
  if (wall) blockOvertake(fromX, fromY, node, wall);
  node.vx = (node.x - fromX) / dtScale;
  node.vy = (node.y - fromY) / dtScale;
  finishStep(node, fromX, fromY);
}

interface Sample {
  x: number;
  y: number;
  nx: number;
  ny: number;
  speed: number;
  progress: number;
}

interface Ripple {
  x: number;
  y: number;
  t: number;
  color: [number, number, number, number];
}

interface Particle {
  x: number;
  y: number;
  vx: number;
  vy: number;
  life: number;
  maxLife: number;
  size: number;
}

/** Preview pixels per overlay pixel. */
const PREVIEW_SCALE = 0.55;

/**
 * Centripetal Catmull-Rom (α = 0.5, Barry–Goldman form) between p1 and p2 — same as
 * `catmull_rom_centripetal` in the Rust renderer. Never hooks or loops for uneven spacing.
 */
function catmullRom(
  p0: [number, number],
  p1: [number, number],
  p2: [number, number],
  p3: [number, number],
  t: number
): [number, number] {
  const knot = (a: [number, number], b: [number, number]) =>
    Math.sqrt(Math.sqrt(Math.max(1e-6, (b[0] - a[0]) ** 2 + (b[1] - a[1]) ** 2)));
  const lerp = (
    a: [number, number],
    b: [number, number],
    wa: number,
    wb: number
  ): [number, number] => [a[0] * wa + b[0] * wb, a[1] * wa + b[1] * wb];
  const t0 = 0;
  const t1 = t0 + knot(p0, p1);
  const t2 = t1 + knot(p1, p2);
  const t3 = t2 + knot(p2, p3);
  const tt = t1 + (t2 - t1) * Math.min(1, Math.max(0, t));
  const a1 = lerp(p0, p1, (t1 - tt) / (t1 - t0), (tt - t0) / (t1 - t0));
  const a2 = lerp(p1, p2, (t2 - tt) / (t2 - t1), (tt - t1) / (t2 - t1));
  const a3 = lerp(p2, p3, (t3 - tt) / (t3 - t2), (tt - t2) / (t3 - t2));
  const b1 = lerp(a1, a2, (t2 - tt) / (t2 - t0), (tt - t0) / (t2 - t0));
  const b2 = lerp(a2, a3, (t3 - tt) / (t3 - t1), (tt - t1) / (t3 - t1));
  return lerp(b1, b2, (t2 - tt) / (t2 - t1), (tt - t1) / (t2 - t1));
}

/** Same curves as `apply_fade_curve` in Rust: 0 linear, 1 ease-out, 2 exponential, 3 sigmoid. */
function fadeCurve(progress: number, mode: number): number {
  switch (mode) {
    case 1:
      return 1 - progress * progress;
    case 2:
      return Math.exp(-progress * 3);
    case 3:
      return 1 / (1 + Math.exp(8 * (progress - 0.5)));
    default:
      return 1 - progress;
  }
}

function hslToRgba(hue: number, sat: number, lit: number): [number, number, number, number] {
  const h = ((hue % 360) + 360) % 360;
  const c = (1 - Math.abs(2 * lit - 1)) * sat;
  const x = c * (1 - Math.abs(((h / 60) % 2) - 1));
  const m = lit - c / 2;
  let r = 0,
    g = 0,
    b = 0;
  if (h < 60) [r, g, b] = [c, x, 0];
  else if (h < 120) [r, g, b] = [x, c, 0];
  else if (h < 180) [r, g, b] = [0, c, x];
  else if (h < 240) [r, g, b] = [0, x, c];
  else if (h < 300) [r, g, b] = [x, 0, c];
  else [r, g, b] = [c, 0, x];
  return [r + m, g + m, b + m, 1];
}

function lerpRgba(
  a: [number, number, number, number],
  b: [number, number, number, number],
  t: number
): [number, number, number, number] {
  const k = Math.max(0, Math.min(1, t));
  return [
    a[0] + (b[0] - a[0]) * k,
    a[1] + (b[1] - a[1]) * k,
    a[2] + (b[2] - a[2]) * k,
    a[3] + (b[3] - a[3]) * k,
  ];
}

function rgbaStr(c: [number, number, number, number], alphaMul = 1): string {
  const r = Math.round(Math.max(0, Math.min(1, c[0])) * 255);
  const g = Math.round(Math.max(0, Math.min(1, c[1])) * 255);
  const b = Math.round(Math.max(0, Math.min(1, c[2])) * 255);
  const a = Math.max(0, Math.min(1, c[3] * alphaMul));
  return `rgba(${r}, ${g}, ${b}, ${a})`;
}

export const LivePreview: Component<LivePreviewProps> = (props) => {
  let canvasRef!: HTMLCanvasElement;
  const [autoMotion, setAutoMotion] = createSignal(true);
  let animId = 0;

  // Offscreen scratch canvas used to build each ribbon layer as a union of capsules.
  let unionCanvas: HTMLCanvasElement | null = null;
  let unionCtx: CanvasRenderingContext2D | null = null;
  const ensureUnionCanvas = (w: number, h: number) => {
    if (!unionCanvas) {
      unionCanvas = document.createElement('canvas');
      unionCtx = unionCanvas.getContext('2d');
    }
    if (!unionCanvas || !unionCtx) return null;
    if (unionCanvas.width !== w || unionCanvas.height !== h) {
      unionCanvas.width = w;
      unionCanvas.height = h;
    }
    return { canvas: unionCanvas, ctx: unionCtx };
  };

  // Simulation state lives in overlay pixel units; drawing applies PREVIEW_SCALE.
  const nodes: Node[] = [];
  const ripples: Ripple[] = [];
  const particles: Particle[] = [];
  let mouseX = 0;
  let mouseY = 0;
  let squish = { prevX: 0, prevY: 0, scale: 0, target: 0, angle: 0, targetAngle: 0 };
  const headTarget = { x: 0, y: 0 };
  const lastMouse = { x: 0, y: 0 };
  const cursorDir = { x: 0, y: 0 };
  let satAngle = 0;
  let mirrorAngle = 0;
  let hue = 0;
  let time = 0;
  let seed = 12345;
  const rand = () => {
    seed ^= seed << 13;
    seed ^= seed >>> 17;
    seed ^= seed << 5;
    return (seed >>> 0) / 4294967295;
  };

  const spawnClick = (button: number, cfg: AppConfig) => {
    const mask = modeMask(cfg.effect_mode);
    if (!cfg.enabled) return;
    if (cfg.ripple.enabled && mask.ripples) {
      const color =
        button === 0
          ? cfg.ripple.color_left
          : button === 1
            ? cfg.ripple.color_right
            : cfg.ripple.color_middle;
      ripples.push({ x: mouseX, y: mouseY, t: 0, color });
    }
    if (cfg.particles.enabled && mask.particles) {
      for (let i = 0; i < cfg.particles.count_per_click && particles.length < 400; i++) {
        const angle = rand() * Math.PI * 2;
        const speed = cfg.particles.base_speed * (0.5 + rand() * 0.8);
        const maxLife = (cfg.particles.duration_ms / 1000) * (0.6 + rand() * 0.8);
        particles.push({
          x: mouseX,
          y: mouseY,
          vx: Math.cos(angle) * speed,
          vy: Math.sin(angle) * speed,
          life: maxLife,
          maxLife,
          size: cfg.particles.size * (0.7 + rand() * 0.6),
        });
      }
    }
  };

  onSettled(() => {
    if (!canvasRef) return;
    const ctx = canvasRef.getContext('2d');
    if (!ctx) return;

    let lastT = performance.now();
    let nextAutoClick = 1.5;

    const loop = (currT: number) => {
      const dt = Math.max(0.001, Math.min((currT - lastT) / 1000, 0.033));
      lastT = currT;
      time += dt;

      const dpr = typeof window !== 'undefined' ? window.devicePixelRatio || 1 : 1;
      const cssWidth = canvasRef.clientWidth || 380;
      const cssHeight = canvasRef.clientHeight || 180;
      if (
        canvasRef.width !== Math.floor(cssWidth * dpr) ||
        canvasRef.height !== Math.floor(cssHeight * dpr)
      ) {
        canvasRef.width = Math.floor(cssWidth * dpr);
        canvasRef.height = Math.floor(cssHeight * dpr);
      }
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);

      const cfg = props.config();
      const mask = modeMask(cfg.effect_mode);
      // World (overlay-pixel) size of the preview area.
      const W = cssWidth / PREVIEW_SCALE;
      const H = cssHeight / PREVIEW_SCALE;

      // ---- 1. Cursor source -----------------------------------------------------------------
      if (autoMotion()) {
        mouseX = W / 2 + Math.sin(time * 2.2) * W * 0.35;
        mouseY = H / 2 + Math.sin(time * 4.4) * H * 0.32;
        if (time >= nextAutoClick) {
          spawnClick(Math.floor(rand() * 3), cfg);
          nextAutoClick = time + 1.2 + rand() * 1.5;
        }
      }

      // ---- 2. Physics (mirrors OverlayRenderer::update_mouse) --------------------------------
      for (const r of ripples) r.t += dt;
      const rippleDur = cfg.ripple.duration_ms / 1000;
      for (let i = ripples.length - 1; i >= 0; i--)
        if (ripples[i].t >= rippleDur) ripples.splice(i, 1);

      const friction = Math.pow(cfg.particles.friction, dt * 60);
      for (let i = particles.length - 1; i >= 0; i--) {
        const p = particles[i];
        p.life -= dt;
        if (p.life <= 0) {
          particles.splice(i, 1);
          continue;
        }
        p.vy += cfg.particles.gravity * dt;
        p.vx *= friction;
        p.vy *= friction;
        p.x += p.vx * dt;
        p.y += p.vy * dt;
      }

      if (cfg.satellites.enabled && mask.satellites) {
        satAngle = (satAngle + cfg.satellites.speed * dt) % (Math.PI * 2);
        mirrorAngle = (mirrorAngle - cfg.satellites.speed * dt) % (Math.PI * 2);
      }
      if (cfg.rainbow.enabled) hue = (hue + cfg.rainbow.speed * dt * 60) % 360;

      const targetLen = Math.max(4, Math.min(cfg.trail.length, 150));
      while (nodes.length < targetLen) nodes.push({ x: mouseX, y: mouseY, vx: 0, vy: 0, speed: 0, dx: 0, dy: 0 });
      if (nodes.length > targetLen) nodes.length = targetLen;

      const dtScale = Math.min(Math.max(dt / (1 / 60), 0.1), 5.0);
      const bodySpring = cfg.trail.spring / 1000;
      const bodyFric = Math.pow(Math.min(Math.max(1 - cfg.trail.damping / 100, 0.01), 1), dtScale);
      const leadNodes = Math.min(Math.max(cfg.trail.lead_nodes ?? 4, 1), nodes.length);

      // Mirrors `TrailChain::step` in Rust: head + lead nodes are pursuit followers, the rest a
      // spring chain, and no node may overtake its predecessor along the predecessor's motion.
      const headSmoothing = Math.min(Math.max(cfg.trail.head_damping / 100, 0), 0.95);
      const targetBlend = 1 - Math.pow(headSmoothing, dtScale);
      headTarget.x += (mouseX - headTarget.x) * targetBlend;
      headTarget.y += (mouseY - headTarget.y) * targetBlend;
      const follow = Math.min(Math.max(cfg.trail.head_spring / 100, 0.05), 0.98);
      const followA = 1 - Math.pow(1 - follow, dtScale);
      const cursorStep = Math.hypot(mouseX - lastMouse.x, mouseY - lastMouse.y);
      if (cursorStep >= 0.05) {
        cursorDir.x = (mouseX - lastMouse.x) / cursorStep;
        cursorDir.y = (mouseY - lastMouse.y) / cursorStep;
      }
      lastMouse.x = mouseX;
      lastMouse.y = mouseY;
      const cursorWall: Wall | null =
        cursorDir.x !== 0 || cursorDir.y !== 0
          ? { x: mouseX, y: mouseY, ux: cursorDir.x, uy: cursorDir.y }
          : null;
      followStep(nodes[0], headTarget.x, headTarget.y, followA, cursorWall, dtScale);
      for (let i = 1; i < nodes.length; i++) {
        const cur = nodes[i];
        const prev = nodes[i - 1];
        const wall: Wall | null =
          prev.dx !== 0 || prev.dy !== 0 ? { x: prev.x, y: prev.y, ux: prev.dx, uy: prev.dy } : null;
        if (i < leadNodes) {
          followStep(cur, prev.x, prev.y, followA, wall, dtScale);
          continue;
        }
        const fromX = cur.x;
        const fromY = cur.y;
        if (i > 1) {
          const pp = nodes[i - 2];
          cur.vx += (pp.x - cur.x) * bodySpring * 0.3 * dtScale;
          cur.vy += (pp.y - cur.y) * bodySpring * 0.3 * dtScale;
        }
        cur.vx += (prev.x - cur.x) * bodySpring * dtScale;
        cur.vy += (prev.y - cur.y) * bodySpring * dtScale;
        cur.vx *= bodyFric;
        cur.vy *= bodyFric;
        cur.x += cur.vx * dtScale;
        cur.y += cur.vy * dtScale;
        if (wall) blockOvertake(fromX, fromY, cur, wall);
        finishStep(cur, fromX, fromY);
      }

      if (cfg.head.enabled) {
        const dx = mouseX - squish.prevX;
        const dy = mouseY - squish.prevY;
        const velocity = Math.hypot(dx, dy) / dt;
        const intensity = cfg.head.squish_intensity / 100;
        squish.target = (Math.min(velocity * 8, 200) / 15) * intensity;
        const smoothing = Math.min(Math.max(cfg.head.squish_smoothing / 100, 0.01), 1);
        const adaptive = Math.min(Math.max(smoothing * dtScale, 0), 1);
        squish.scale += (squish.target - squish.scale) * adaptive;
        if (velocity > 0.5) squish.targetAngle = Math.atan2(dy, dx);
        let diff = squish.targetAngle - squish.angle;
        while (diff > Math.PI) diff -= Math.PI * 2;
        while (diff < -Math.PI) diff += Math.PI * 2;
        squish.angle += diff * adaptive;
        squish.prevX = mouseX;
        squish.prevY = mouseY;
      }

      // ---- 3. Draw --------------------------------------------------------------------------
      ctx.clearRect(0, 0, cssWidth, cssHeight);
      ctx.strokeStyle = 'rgba(255, 255, 255, 0.03)';
      ctx.lineWidth = 1;
      ctx.beginPath();
      for (let x = 0; x < cssWidth; x += 30) {
        ctx.moveTo(x, 0);
        ctx.lineTo(x, cssHeight);
      }
      for (let y = 0; y < cssHeight; y += 30) {
        ctx.moveTo(0, y);
        ctx.lineTo(cssWidth, y);
      }
      ctx.stroke();

      ctx.save();
      ctx.scale(PREVIEW_SCALE, PREVIEW_SCALE);

      const enabled = cfg.enabled;

      // Ribbon
      if (enabled && cfg.trail.enabled && mask.trail && nodes.length >= 4) {
        const samples: Sample[] = [];
        // Mirror the renderer: the centerline starts at the real cursor so the cap stays attached.
        const chain: Node[] = [
          { x: mouseX, y: mouseY, vx: 0, vy: 0, speed: nodes[0].speed, dx: 0, dy: 0 },
          ...nodes,
        ];
        // Phantom control points mirrored past both ends so the curve runs cursor → last node.
        const mirror = (a: Node, b: Node): Node => ({
          x: 2 * a.x - b.x,
          y: 2 * a.y - b.y,
          vx: 0,
          vy: 0,
          speed: a.speed,
          dx: 0,
          dy: 0,
        });
        const ext: Node[] = [
          mirror(chain[0], chain[1]),
          ...chain,
          mirror(chain[chain.length - 1], chain[chain.length - 2]),
        ];
        const segCount = ext.length - 3;
        const baseSteps = Math.max(1, cfg.trail.interpolation_steps);
        let prevNx = 0;
        let prevNy = 0;
        for (let s = 0; s < segCount; s++) {
          const p0: [number, number] = [ext[s].x, ext[s].y];
          const p1: [number, number] = [ext[s + 1].x, ext[s + 1].y];
          const p2: [number, number] = [ext[s + 2].x, ext[s + 2].y];
          const p3: [number, number] = [ext[s + 3].x, ext[s + 3].y];
          const sp1 = ext[s + 1].speed;
          const sp2 = ext[s + 2].speed;
          const segDist = Math.hypot(p2[0] - p1[0], p2[1] - p1[1]);
          const steps = cfg.trail.adaptive_quality
            ? Math.min(24, Math.max(baseSteps, Math.ceil(segDist / 6)))
            : baseSteps;
          for (let st = 0; st < steps; st++) {
            const t = st / steps;
            const pt = catmullRom(p0, p1, p2, p3, t);
            const nxt = catmullRom(p0, p1, p2, p3, (st + 0.5) / steps);
            const pdx = nxt[0] - pt[0];
            const pdy = nxt[1] - pt[1];
            const plen = Math.hypot(pdx, pdy);
            let nx = plen > 0.0001 ? -pdy / plen : 0;
            let ny = plen > 0.0001 ? pdx / plen : 1;
            if (samples.length > 0 && nx * prevNx + ny * prevNy < 0) {
              nx = -nx;
              ny = -ny;
            }
            prevNx = nx;
            prevNy = ny;
            samples.push({
              x: pt[0],
              y: pt[1],
              nx,
              ny,
              speed: sp1 + (sp2 - sp1) * t,
              progress: (s + t) / segCount,
            });
          }
        }

        const drawLayer = (layer: LayerConfig) => {
          const sat = Math.min(Math.max(cfg.rainbow.saturation, 0), 1);
          const lit = Math.min(Math.max(cfg.rainbow.lightness, 0), 1);
          let startC: [number, number, number, number] = layer.start_color;
          let endC: [number, number, number, number] = cfg.trail.enable_gradient
            ? layer.end_color
            : layer.start_color;
          if (cfg.rainbow.enabled) {
            startC = hslToRgba(hue, sat, lit);
            startC[3] = layer.start_color[3];
            endC = hslToRgba(hue + 180, sat, lit * 0.6);
            endC[3] = cfg.trail.enable_gradient ? layer.end_color[3] : layer.start_color[3];
          }
          const gradient = cfg.trail.enable_gradient || cfg.rainbow.enabled;

          // Per-sample radius / colour / alpha, same formulas as `layer_sample_style` in Rust.
          const styles = samples.map((s) => {
            const fade = fadeCurve(s.progress, cfg.trail.fade_mode);
            const normSpeed = Math.min(s.speed / 20, 1);
            const velWidth = 1 + normSpeed * cfg.trail.velocity_width_mult;
            const velAlpha = 1 + normSpeed * cfg.trail.velocity_alpha_mult;
            const width = Math.max(
              cfg.trail.min_width,
              cfg.trail.cursor_size * layer.width_factor * fade * velWidth
            );
            const c = gradient ? lerpRgba(startC, endC, s.progress) : startC;
            return {
              r: width * 0.5,
              alpha: Math.min(1, c[3] * layer.alpha_factor * fade * velAlpha),
              rgb: [c[0], c[1], c[2]] as [number, number, number],
            };
          });

          // Union-of-capsules per band, mirroring the GPU's max-coverage resolve: each capsule
          // first erases what is under it and then paints itself, so overlapping capsules never
          // blend twice. Head is painted last so it wins where the trail crosses itself.
          const blur = Math.min(Math.max(layer.start_blur, 0), 1);
          const bands: { radiusScale: number; alphaScale: number }[] =
            blur > 0.15
              ? [
                  { radiusScale: 1, alphaScale: 0.4 }, // soft feather
                  { radiusScale: 1 - blur * 0.7, alphaScale: 1 }, // solid core
                ]
              : [{ radiusScale: 1, alphaScale: 1 }];

          const union = ensureUnionCanvas(canvasRef.width, canvasRef.height);
          if (!union) return;
          const uctx = union.ctx;
          uctx.lineCap = 'round';
          uctx.lineJoin = 'round';

          for (const band of bands) {
            uctx.setTransform(1, 0, 0, 1, 0, 0);
            uctx.globalCompositeOperation = 'source-over';
            uctx.clearRect(0, 0, union.canvas.width, union.canvas.height);
            uctx.setTransform(dpr * PREVIEW_SCALE, 0, 0, dpr * PREVIEW_SCALE, 0, 0);

            for (let i = samples.length - 2; i >= 0; i--) {
              const a = samples[i];
              const b = samples[i + 1];
              const sa = styles[i];
              const sb = styles[i + 1];
              const alpha = Math.min(1, (sa.alpha + sb.alpha) * 0.5 * band.alphaScale);
              if (alpha < 0.004) continue;
              const r = Math.max(0.35, (sa.r + sb.r) * 0.5 * band.radiusScale);
              uctx.lineWidth = r * 2;
              uctx.beginPath();
              uctx.moveTo(a.x, a.y);
              uctx.lineTo(b.x, b.y);
              if (alpha < 0.999) {
                uctx.globalCompositeOperation = 'destination-out';
                uctx.strokeStyle = '#000';
                uctx.stroke();
                uctx.globalCompositeOperation = 'source-over';
              }
              uctx.strokeStyle = `rgba(${Math.round(sa.rgb[0] * 255)}, ${Math.round(
                sa.rgb[1] * 255
              )}, ${Math.round(sa.rgb[2] * 255)}, ${alpha})`;
              uctx.stroke();
            }

            ctx.save();
            ctx.setTransform(1, 0, 0, 1, 0, 0);
            ctx.drawImage(union.canvas, 0, 0);
            ctx.restore();
          }
        };

        for (let l = 0; l < 4; l++) {
          const layer = cfg.trail.layers[l];
          if (layer.enabled && mask.layers[l]) drawLayer(layer);
        }
      }

      // Ripples
      if (enabled && mask.ripples) {
        for (const r of ripples) {
          const progress = Math.min(1, r.t / rippleDur);
          const fade = fadeCurve(progress, cfg.trail.fade_mode);
          const ease = 1 - Math.pow(1 - progress, 3);
          const diameter = cfg.ripple.max_diameter * ease;
          const width = cfg.ripple.start_width * fade;
          if (diameter > 0.1 && width >= 0.5) {
            ctx.strokeStyle = rgbaStr(r.color, fade);
            ctx.lineWidth = width;
            ctx.beginPath();
            ctx.arc(r.x, r.y, diameter / 2, 0, Math.PI * 2);
            ctx.stroke();
          }
        }
      }

      // Head
      if (enabled && cfg.head.enabled && mask.head) {
        ctx.save();
        ctx.translate(mouseX, mouseY);
        ctx.rotate(squish.angle);
        const base = cfg.head.size * 0.5;
        const radX = base * (1 + squish.scale);
        const radY = base * Math.max(0.3, 1 - squish.scale * 0.5);
        ctx.beginPath();
        ctx.ellipse(0, 0, radX, radY, 0, 0, Math.PI * 2);
        if (cfg.head.filled) {
          ctx.fillStyle = rgbaStr(cfg.head.color);
          ctx.fill();
        } else {
          ctx.strokeStyle = rgbaStr(cfg.head.color);
          ctx.lineWidth = Math.max(1, cfg.head.thickness);
          ctx.stroke();
        }
        ctx.restore();
      }

      // Particles
      if (enabled && mask.particles) {
        ctx.fillStyle = rgbaStr(cfg.particles.color);
        for (const p of particles) {
          ctx.globalAlpha = Math.max(0, Math.min(1, p.life / p.maxLife));
          ctx.beginPath();
          ctx.arc(p.x, p.y, p.size / 2, 0, Math.PI * 2);
          ctx.fill();
        }
        ctx.globalAlpha = 1;
      }

      // Satellites
      if (enabled && cfg.satellites.enabled && mask.satellites) {
        const orbitR = cfg.satellites.orbit_diameter / 2;
        const satR = cfg.satellites.size / 2;
        if (cfg.satellites.show_orbit_ring) {
          ctx.strokeStyle = rgbaStr(cfg.satellites.color, 0.3);
          ctx.lineWidth = Math.max(0.5, cfg.satellites.orbit_ring_thickness);
          ctx.beginPath();
          ctx.arc(mouseX, mouseY, orbitR, 0, Math.PI * 2);
          ctx.stroke();
        }
        const count = Math.max(1, cfg.satellites.count);
        const step = (Math.PI * 2) / count;
        ctx.fillStyle = rgbaStr(cfg.satellites.color);
        for (let i = 0; i < count; i++) {
          const a = satAngle + i * step;
          ctx.beginPath();
          ctx.arc(
            mouseX + Math.cos(a) * orbitR,
            mouseY + Math.sin(a) * orbitR,
            satR,
            0,
            Math.PI * 2
          );
          ctx.fill();
        }
        if (cfg.satellites.dual_ring) {
          ctx.fillStyle = rgbaStr(cfg.satellites.color, 0.7);
          for (let i = 0; i < count; i++) {
            const a = mirrorAngle + (i + 0.5) * step;
            ctx.beginPath();
            ctx.arc(
              mouseX + Math.cos(a) * orbitR,
              mouseY + Math.sin(a) * orbitR,
              satR * 0.75,
              0,
              Math.PI * 2
            );
            ctx.fill();
          }
        }
      }

      ctx.restore();
      animId = requestAnimationFrame(loop);
    };

    animId = requestAnimationFrame(loop);
    return () => cancelAnimationFrame(animId);
  });

  const toWorld = (e: MouseEvent) => {
    const rect = canvasRef.getBoundingClientRect();
    mouseX = (e.clientX - rect.left) / PREVIEW_SCALE;
    mouseY = (e.clientY - rect.top) / PREVIEW_SCALE;
  };

  const handleMouseMove = (e: MouseEvent) => {
    if (!canvasRef) return;
    toWorld(e);
    if (autoMotion()) setAutoMotion(false);
  };

  const handleMouseDown = (e: MouseEvent) => {
    if (!canvasRef) return;
    e.preventDefault();
    toWorld(e);
    spawnClick(e.button, props.config());
  };

  return (
    <div style="background: #08080a; border: 1px solid var(--card-border); border-radius: var(--radius-lg); overflow: hidden; display: flex; flex-direction: column;">
      <div style="display: flex; align-items: center; justify-content: space-between; padding: 10px 16px; border-bottom: 1px solid rgba(255,255,255,0.06); background: #0c0d11;">
        <div style="font-size: 12px; font-weight: 700; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
          <span style="width: 8px; height: 8px; border-radius: 50%; background: var(--accent-primary); box-shadow: 0 0 8px var(--accent-primary);" />
          Live Preview
          <span style="font-weight: 400; color: var(--text-dim); font-size: 11px;">
            · same physics as the overlay · click to test ripples
          </span>
        </div>
        <button
          class={`tab-btn ${autoMotion() ? 'active' : ''}`}
          style="padding: 3px 10px; font-size: 11px;"
          onClick={() => setAutoMotion(!autoMotion())}
        >
          {autoMotion() ? 'Auto Orbiting' : 'Mouse Follow'}
        </button>
      </div>

      <canvas
        ref={canvasRef}
        onMouseMove={handleMouseMove}
        onMouseDown={handleMouseDown}
        onContextMenu={(e) => e.preventDefault()}
        onMouseLeave={() => setAutoMotion(true)}
        style="width: 100%; height: 180px; display: block; cursor: crosshair;"
      />
    </div>
  );
};
