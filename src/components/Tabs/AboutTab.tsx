import { Component, createSignal, onSettled } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { commands, type SystemDiagnostics } from '../../lib/bindings';
import { isTauri } from '../../lib/tauri';
import { APP_VERSION } from '../../../scripts/version';

const tile =
  'background: rgba(255,255,255,0.03); border: 1px solid rgba(255,255,255,0.06); border-radius: 8px; padding: 12px;';
const tileLabel =
  'font-size: 11px; font-weight: 700; color: var(--text-dim); text-transform: uppercase;';
const tileValue = 'font-size: 13px; font-weight: 600; margin-top: 2px;';

export const AboutTab: Component = () => {
  const [diag, setDiag] = createSignal<SystemDiagnostics | null>(null);

  onSettled(() => {
    if (!isTauri) return;
    commands
      .getDiagnostics()
      .then(setDiag)
      .catch((e) => console.warn('Could not load diagnostics:', e));
  });

  const version = () => diag()?.app_version ?? APP_VERSION;
  const gpu = () => {
    const g = diag()?.gpu;
    return g
      ? `${g.adapter} · ${g.backend}`
      : isTauri
        ? 'initialising…'
        : 'wgpu 30 (DX12 / Vulkan / Metal)';
  };
  const input = () =>
    !diag()
      ? isTauri
        ? '…'
        : 'n/a in browser preview'
      : diag()?.input_backend === 'hook'
        ? 'OS low-level mouse hook, event-driven wake'
        : 'Cursor polling fallback';

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title={`FXCursor Studio v${version()}`}
        desc="Hardware-accelerated cursor effects for the desktop: 4-layer ribbon trail, squishy head, click ripples, particles and satellites on a transparent click-through overlay"
      >
        <p style="font-size: 13px; line-height: 1.6; color: var(--text-muted);">
          The trail reproduces the <strong>4-layer master design</strong> (Outer Glow 150 %, Mid
          Shadow 90 %, Crisp Core 50 %, Inner Spine 15 %) with round capsule joins that stay smooth
          through hairpins (union resolved on the GPU, no double blending). Physics (spring-damper
          chain, Catmull-Rom sampling) runs on the CPU each frame; rasterisation runs on the GPU
          through <strong>wgpu 30</strong> with pre-multiplied alpha. When nothing on screen can
          change, GPU work stops entirely.
        </p>

        <div style="margin-top: 8px; display: grid; grid-template-columns: repeat(auto-fit, minmax(240px, 1fr)); gap: 10px;">
          <div style={tile}>
            <div style={tileLabel}>Graphics Adapter</div>
            <div style={`${tileValue} color: var(--accent-primary);`}>{gpu()}</div>
          </div>
          <div style={tile}>
            <div style={tileLabel}>Input Backend</div>
            <div style={`${tileValue} color: #10b981;`}>{input()}</div>
          </div>
          <div style={tile}>
            <div style={tileLabel}>User Interface</div>
            <div style={`${tileValue} color: var(--accent-blue);`}>
              SolidJS 2.0 · TypeScript 7 · Vite 8 · Bun 1.4
            </div>
          </div>
          <div style={tile}>
            <div style={tileLabel}>Desktop Shell</div>
            <div style={`${tileValue} color: #a855f7;`}>
              Tauri 2 · single process · {diag()?.os ?? 'desktop'} {diag()?.arch ?? ''}
            </div>
          </div>
        </div>
      </SectionCard>

      <SectionCard
        title="Resilience & Integration"
        desc="What the backend does for you behind the scenes"
      >
        <div style="display: flex; flex-direction: column; gap: 8px; font-size: 12px; color: var(--text-muted); line-height: 1.6;">
          <div>
            • <strong>Auto-save with field-level self-healing:</strong> every change is written to{' '}
            <code style="user-select: text;">{diag()?.config_path || 'config.json'}</code>; a
            corrupted file is repaired field by field (backup kept as <code>.json.bak</code>).
          </div>
          <div>
            • <strong>Multi-monitor virtual desktop:</strong> the overlay spans{' '}
            {diag()
              ? `${diag()?.screen_virtual_size[0]} × ${diag()?.screen_virtual_size[1]} from (${diag()?.screen_virtual_origin[0]}, ${diag()?.screen_virtual_origin[1]})`
              : 'all monitors'}
            , including negative coordinates.
          </div>
          <div>
            • <strong>Never-missed clicks:</strong> button presses are captured by the OS hook and
            queued, so a click shorter than a frame still triggers ripples and particles.
          </div>
          <div>
            • <strong>Tray, hotkey, autostart:</strong> global toggle shortcut, login autostart,
            single-instance guard, minimize-to-tray and start-minimized are driven by the Hotkeys
            &amp; Tray tab and persisted with everything else.
          </div>
          <div>
            • <strong>Portable mode:</strong> place an empty file named <code>portable</code> next
            to the executable to keep all data in <code>Data/</code>.
          </div>
        </div>
      </SectionCard>

      <SectionCard
        title="License"
        desc="MIT License — free for personal, academic and commercial use"
      >
        <div style="font-size: 12px; color: var(--text-dim);">
          Source: github.com/NairoDorian/FXCursor
        </div>
      </SectionCard>
    </div>
  );
};
