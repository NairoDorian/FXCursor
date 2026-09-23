import { Component, createSignal, onSettled } from 'solid-js';
import type { AppConfig, LayerConfig } from '../../lib/presets';
import { modeMask } from '../../lib/effectMode';
import {
  buildSamples,
  fadeCurve,
  layerSampleStyle,
  MAX_FRAME_DELTA,
  MIN_VISIBLE_ALPHA,
  REFERENCE_FRAME,
  SquishyHead,
  TrailChain,
  type Rgba,
} from '../../lib/trail';

/**
 * 2D-canvas preview of `crates/fxcursor-render`. The trail physics, spline sampling, fade and
 * width/alpha styling come from `src/lib/trail.ts`, which is tested against a Rust reference
 * trace (`test/trail-parity.test.ts`), so the motion here is the overlay's motion. Only the
 * compositing is approximated (Canvas 2D instead of the GPU depth-resolved capsule union).
 * The scene is scaled by `PREVIEW_SCALE` to fit.
 */
interface LivePreviewProps {
  config: () => AppConfig;
}

interface Ripple {
  x: number;
  y: number;
  t: number;
  color: Rgba;
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

/** DOM `MouseEvent.button` (0 left, 1 middle, 2 right) → renderer order (0 left, 1 right, 2 middle). */
const DOM_TO_RENDERER_BUTTON: Record<number, number> = { 0: 0, 1: 2, 2: 1 };

function hslToRgba(hue: number, sat: number, lit: number): Rgba {
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

function rgbaStr(c: Rgba, alphaMul = 1): string {
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
  const chain = new TrailChain();
  const head = new SquishyHead();
  const ripples: Ripple[] = [];
  const particles: Particle[] = [];
  let mouseX = 0;
  let mouseY = 0;
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

  /** `button` in renderer order: 0 = left, 1 = right, anything else = middle/extra. */
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
      // Evenly spaced directions with a little jitter, like `spawn_click` in Rust.
      const count = Math.min(cfg.particles.count_per_click, Math.max(0, 400 - particles.length));
      for (let i = 0; i < count; i++) {
        const angle = (i / Math.max(1, count)) * Math.PI * 2 + (rand() - 0.5) * 0.5;
        const speed = cfg.particles.base_speed * (0.7 + rand() * 0.6);
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
      // Same clamp as `update_mouse` (rAF pauses in hidden tabs; resume without a jump).
      const dt = Math.max(0.001, Math.min((currT - lastT) / 1000, MAX_FRAME_DELTA));
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
      chain.resize(targetLen, mouseX, mouseY);
      chain.advance(mouseX, mouseY, dt, cfg.trail);
      const dtScale = Math.min(Math.max(dt / REFERENCE_FRAME, 0.1), 5);
      if (cfg.head.enabled) head.step(mouseX, mouseY, dtScale, cfg.head);
      else head.reset();

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
      if (enabled && cfg.trail.enabled && mask.trail) {
        const built = buildSamples(chain.nodes, cfg.trail);
        // A collapsed chain is one sample: draw it as a zero-length capsule (the resting dot).
        const samples = built.length === 1 ? [built[0], built[0]] : built;

        const drawLayer = (layer: LayerConfig) => {
          const sat = Math.min(Math.max(cfg.rainbow.saturation, 0), 1);
          const lit = Math.min(Math.max(cfg.rainbow.lightness, 0), 1);
          let startC: Rgba = layer.start_color;
          let endC: Rgba = cfg.trail.enable_gradient ? layer.end_color : layer.start_color;
          if (cfg.rainbow.enabled) {
            startC = hslToRgba(hue, sat, lit);
            startC[3] = layer.start_color[3];
            endC = hslToRgba(hue + 180, sat, lit * 0.6);
            endC[3] = cfg.trail.enable_gradient ? layer.end_color[3] : layer.start_color[3];
          }
          const gradient = cfg.trail.enable_gradient || cfg.rainbow.enabled;
          const styles = samples.map((s) => layerSampleStyle(s, layer, cfg.trail, startC, endC, gradient));
          const blurAt = (p: number) =>
            Math.min(Math.max(layer.start_blur + (layer.end_blur - layer.start_blur) * p, 0), 1);

          // Union of capsules per band, approximating the GPU's max-coverage resolve: each
          // capsule first erases what is under it and then paints itself, so overlapping
          // capsules never blend twice. Head is painted last so it wins where the trail crosses
          // itself. The feather is approximated by a soft outer band and a solid inner core.
          const soft = layer.start_blur > 0.15 || layer.end_blur > 0.15;
          const bands = soft ? ['feather', 'core'] : ['solid'];

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
              if (sa.color[3] < MIN_VISIBLE_ALPHA && sb.color[3] < MIN_VISIBLE_ALPHA) continue;
              const blur = blurAt((a.progress + b.progress) * 0.5);
              const radiusScale = band === 'core' ? 1 - blur * 0.7 : 1;
              const alphaScale = band === 'feather' ? 0.4 : 1;
              const alpha = Math.min(1, (sa.color[3] + sb.color[3]) * 0.5 * alphaScale);
              const r = Math.max(0.35, (sa.radius + sb.radius) * 0.5 * radiusScale);
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
              uctx.strokeStyle = rgbaStr([sa.color[0], sa.color[1], sa.color[2], alpha]);
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

      // Head (eased toward the pointer like the overlay's squishy head)
      if (enabled && cfg.head.enabled && mask.head) {
        ctx.save();
        ctx.translate(head.x, head.y);
        ctx.rotate(head.angle);
        const base = cfg.head.size * 0.5;
        const radX = base * (1 + head.scale);
        const radY = base * Math.max(0.3, 1 - head.scale * 0.5);
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
    spawnClick(DOM_TO_RENDERER_BUTTON[e.button] ?? 2, props.config());
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
        role="img"
        aria-label="Live preview of the cursor trail and effects"
        onMouseMove={handleMouseMove}
        onMouseDown={handleMouseDown}
        onContextMenu={(e) => e.preventDefault()}
        onMouseLeave={() => setAutoMotion(true)}
        style="width: 100%; height: 180px; display: block; cursor: crosshair;"
      />
    </div>
  );
};
