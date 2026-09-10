import { Component, createSignal, createEffect, onSettled, For, Show } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import {
  commands,
  type FpsCounterConfig,
  type FrameStats,
  type SystemDiagnostics,
} from '../../lib/bindings';
import { isTauri } from '../../lib/tauri';
import { toast } from '../../lib/toast';
import { THEME_ACCENTS, applyThemeAccent } from '../../lib/theme';

interface DeveloperTabProps {
  onResetDefaults: () => void;
  /** `config.fps_counter`: enables live frame telemetry polling at `refresh_rate_ms`. */
  fpsCounter: FpsCounterConfig;
  onFpsCounterChange: (next: FpsCounterConfig) => void;
}

const cardStyle =
  'background: #121318; padding: 10px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.05);';
const labelStyle = 'color: #888; font-size: 11px; display: block;';
const valueStyle = 'color: #ffffff; font-size: 13px; font-weight: 600;';
const monoStyle = `${valueStyle} font-family: monospace;`;

function formatUptime(secs: number): string {
  const s = Math.floor(secs);
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const r = s % 60;
  return h > 0 ? `${h}h ${m}m ${r}s` : m > 0 ? `${m}m ${r}s` : `${r}s`;
}

export const DeveloperTab: Component<DeveloperTabProps> = (props) => {
  const [diag, setDiag] = createSignal<SystemDiagnostics | null>(null);
  const [latencyMs, setLatencyMs] = createSignal<number | null>(null);
  const [benchmarking, setBenchmarking] = createSignal(false);
  const [saving, setSaving] = createSignal(false);
  const [activeTheme, setActiveTheme] = createSignal('cyan');

  const refreshDiagnostics = async () => {
    if (!isTauri) return;
    try {
      setDiag(await commands.getDiagnostics());
    } catch (e) {
      console.warn('Could not load diagnostics:', e);
    }
  };

  onSettled(() => {
    void refreshDiagnostics();
  });

  // Live frame telemetry: poll the backend while `fps_counter.enabled` at its refresh rate.
  const [frame, setFrame] = createSignal<FrameStats | null>(null);
  const [telemetryOn, setTelemetryOn] = createSignal(false);
  const pollEvery = () => Math.max(100, props.fpsCounter.refresh_rate_ms);
  createEffect(
    () => ({ on: props.fpsCounter.enabled && isTauri, every: pollEvery() }),
    ({ on, every }) => {
      setTelemetryOn(on);
      if (!on) return;
      const tick = async () => {
        try {
          const d = await commands.getDiagnostics();
          setFrame(d.frame);
        } catch {
          // transient IPC error: keep the last sample
        }
      };
      void tick();
      const id = setInterval(() => void tick(), every);
      // SolidJS 2: the effect returns its cleanup (runs before the next run / on dispose).
      return () => clearInterval(id);
    }
  );

  const stateColor = () => {
    const s = frame()?.state;
    return s === 'active' ? '#10b981' : s === 'settling' ? '#f97316' : 'var(--text-muted)';
  };

  const runBenchmark = async () => {
    if (!isTauri) {
      toast.info('Browser preview mode: no IPC channel to benchmark');
      return;
    }
    setBenchmarking(true);
    const iterations = 50;
    const start = performance.now();
    for (let i = 0; i < iterations; i++) {
      await commands.ping();
    }
    const elapsed = performance.now() - start;
    const avg = Math.round((elapsed / iterations) * 100) / 100;
    setLatencyMs(avg);
    setBenchmarking(false);
    toast.success(`IPC benchmark: ${avg} ms average round trip over ${iterations} calls`);
  };

  const saveNow = async () => {
    if (!isTauri) {
      toast.info('Browser preview mode: nothing to save');
      return;
    }
    setSaving(true);
    try {
      const path = await commands.saveConfig();
      toast.success(`Configuration written to ${path}`);
    } catch (e) {
      toast.error(`Save failed: ${e}`);
    } finally {
      setSaving(false);
    }
  };

  const [capturing, setCapturing] = createSignal(false);
  const [lastCapture, setLastCapture] = createSignal<string | null>(null);
  const captureSnapshot = async (whole: boolean) => {
    if (!isTauri) {
      toast.info('Browser preview mode: no overlay to capture');
      return;
    }
    setCapturing(true);
    try {
      // 0×0 = whole overlay; otherwise a crop centred on the cursor.
      const path = await (whole
        ? commands.captureOverlay(0, 0)
        : commands.captureOverlay(1280, 720));
      setLastCapture(path);
      toast.success(`Overlay snapshot saved: ${path}`);
    } catch (e) {
      toast.error(`Snapshot failed: ${e}`);
    } finally {
      setCapturing(false);
    }
  };

  const handleSelectTheme = (themeId: string) => {
    setActiveTheme(themeId);
    applyThemeAccent(themeId);
    toast.info(`Applied theme accent: ${themeId}`);
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      {/* IPC Telemetry & Benchmarking */}
      <SectionCard
        title="IPC Benchmark"
        desc="Measure Tauri 2 command round-trip latency between the Studio webview and the Rust backend"
        headerRight={
          <button
            class="tab-btn active"
            style="padding: 4px 12px; font-size: 11px;"
            onClick={runBenchmark}
            disabled={benchmarking()}
          >
            {benchmarking() ? 'Benchmarking...' : 'Run IPC Benchmark'}
          </button>
        }
      >
        <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 12px;">
          <div style="background: #121318; padding: 12px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.06);">
            <div style="color: #888; font-size: 11px; margin-bottom: 4px;">
              Average IPC Roundtrip Latency
            </div>
            <div style="color: var(--accent-primary); font-size: 20px; font-weight: 700; font-family: monospace;">
              {latencyMs() !== null ? `${latencyMs()} ms` : 'Not tested'}
            </div>
          </div>
          <div style="background: #121318; padding: 12px; border-radius: 8px; border: 1px solid rgba(255,255,255,0.06);">
            <div style="color: #888; font-size: 11px; margin-bottom: 4px;">Transport</div>
            <div style="color: #10b981; font-size: 20px; font-weight: 700;">
              {isTauri ? 'Tauri IPC (typed bindings)' : 'Browser preview'}
            </div>
          </div>
        </div>
      </SectionCard>

      {/* Live frame telemetry */}
      <SectionCard
        title="Render Loop Telemetry"
        desc="Frames presented per second, CPU cost per frame and geometry counts reported by the gpu-render-thread"
        headerRight={
          <div style="display: flex; align-items: center; gap: 10px;">
            <span style="font-size: 11px; color: var(--text-dim);">
              {telemetryOn() ? `every ${pollEvery()} ms` : isTauri ? 'off' : 'desktop app only'}
            </span>
            <label class="switch" title="Toggle live telemetry (config.fps_counter.enabled)">
              <input
                type="checkbox"
                checked={props.fpsCounter.enabled}
                disabled={!isTauri}
                onChange={(e) =>
                  props.onFpsCounterChange({
                    ...props.fpsCounter,
                    enabled: e.currentTarget.checked,
                  })
                }
              />
              <span class="slider-round" />
            </label>
          </div>
        }
      >
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(150px, 1fr)); gap: 10px;">
          <div style={cardStyle}>
            <span style={labelStyle}>State</span>
            <span style={`${valueStyle} color: ${stateColor()};`}>{frame()?.state ?? '—'}</span>
          </div>
          <div style={cardStyle}>
            <span style={labelStyle}>Frames / s</span>
            <span style={monoStyle}>{frame() ? frame()!.fps.toFixed(0) : '—'}</span>
          </div>
          <div style={cardStyle}>
            <span style={labelStyle}>CPU per frame</span>
            <span style={monoStyle}>{frame() ? `${frame()!.frame_ms.toFixed(2)} ms` : '—'}</span>
          </div>
          <div style={cardStyle}>
            <span style={labelStyle}>Ribbon vertices</span>
            <span style={monoStyle}>{frame()?.ribbon_vertices ?? '—'}</span>
          </div>
          <div style={cardStyle}>
            <span style={labelStyle}>SDF instances</span>
            <span style={monoStyle}>{frame()?.instances ?? '—'}</span>
          </div>
          <div style={cardStyle}>
            <span style={labelStyle}>Refresh rate</span>
            <input
              type="range"
              min="100"
              max="2000"
              step="100"
              value={props.fpsCounter.refresh_rate_ms}
              disabled={!isTauri}
              onInput={(e) =>
                props.onFpsCounterChange({
                  ...props.fpsCounter,
                  refresh_rate_ms: Number(e.currentTarget.value),
                })
              }
              style="width: 100%;"
            />
          </div>
        </div>
      </SectionCard>

      {/* System Diagnostics */}
      <SectionCard
        title="Runtime Diagnostics"
        desc="GPU adapter, input backend, virtual desktop bounds and configuration storage reported by the backend"
        headerRight={
          <button
            class="tab-btn"
            style="padding: 4px 12px; font-size: 11px;"
            onClick={() => void refreshDiagnostics()}
            disabled={!isTauri}
          >
            Refresh
          </button>
        }
      >
        <Show
          when={diag()}
          fallback={
            <div style="color: var(--text-dim); font-size: 12px;">
              {isTauri
                ? 'Loading diagnostics…'
                : 'Diagnostics are only available inside the desktop app.'}
            </div>
          }
        >
          {(d) => (
            <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 10px;">
              <div style={cardStyle}>
                <span style={labelStyle}>OS & Target Arch</span>
                <span style={valueStyle}>
                  {d().os} ({d().arch}) {d().debug_build ? '· debug' : '· release'}
                </span>
              </div>
              <div style={cardStyle}>
                <span style={labelStyle}>App Version / Uptime</span>
                <span style={monoStyle}>
                  v{d().app_version} · {formatUptime(d().uptime_secs)}
                </span>
              </div>
              <div style={cardStyle}>
                <span style={labelStyle}>GPU Adapter</span>
                <span style={valueStyle}>{d().gpu?.adapter ?? 'initialising…'}</span>
              </div>
              <div style={cardStyle}>
                <span style={labelStyle}>Graphics Backend</span>
                <span style={monoStyle}>
                  {d().gpu ? `${d().gpu?.backend} · ${d().gpu?.device_type}` : '—'}
                </span>
              </div>
              <div style={cardStyle}>
                <span style={labelStyle}>Input Backend</span>
                <span style={valueStyle}>
                  {d().input_backend === 'hook'
                    ? 'OS low-level mouse hook (event driven)'
                    : 'Cursor polling (fallback)'}
                </span>
              </div>
              <div style={cardStyle}>
                <span style={labelStyle}>Virtual Desktop</span>
                <span style={monoStyle}>
                  {d().screen_virtual_size[0]} × {d().screen_virtual_size[1]} @ (
                  {d().screen_virtual_origin[0]}, {d().screen_virtual_origin[1]})
                </span>
              </div>
              <div style={cardStyle}>
                <span style={labelStyle}>Portable Mode</span>
                <span style={valueStyle}>
                  {d().portable ? 'Active (Data/)' : 'Standard (AppData)'}
                </span>
              </div>
              <div style={`${cardStyle} grid-column: 1 / -1;`}>
                <span style={labelStyle}>Configuration File (auto-saved, self-healing)</span>
                <span
                  style={`${monoStyle} font-size: 12px; word-break: break-all; user-select: text;`}
                  title={d().config_path}
                >
                  {d().config_path || '—'}
                </span>
              </div>
            </div>
          )}
        </Show>
      </SectionCard>

      {/* Overlay snapshot */}
      <SectionCard
        title="Overlay Snapshot"
        desc="Render the current overlay frame to a PNG (GDI screen capture cannot see the GPU overlay). Move the cursor first, then click."
      >
        <div style="display: flex; gap: 12px; flex-wrap: wrap; align-items: center;">
          <button
            class="tab-btn active"
            style="padding: 8px 16px;"
            onClick={() => void captureSnapshot(false)}
            disabled={capturing() || !isTauri}
          >
            {capturing() ? 'Capturing…' : 'Capture around cursor (1280×720)'}
          </button>
          <button
            class="tab-btn"
            style="padding: 8px 16px;"
            onClick={() => void captureSnapshot(true)}
            disabled={capturing() || !isTauri}
          >
            Capture whole overlay
          </button>
          <Show when={lastCapture()}>
            <span
              style="font-family: monospace; font-size: 11px; color: var(--text-muted); word-break: break-all; user-select: text;"
              title={lastCapture() ?? ''}
            >
              {lastCapture()}
            </span>
          </Show>
        </div>
        <div style="margin-top: 8px; font-size: 11px; color: var(--text-dim);">
          Also available from a terminal while the app runs:{' '}
          <code>fxcursor.exe --capture out.png --capture-size 1280x720</code>
        </div>
      </SectionCard>

      {/* Studio UI Theme Customizer */}
      <SectionCard
        title="Studio Neon Accent Themes"
        desc="Customize the AMOLED black interface accent glow"
      >
        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
          <For each={THEME_ACCENTS}>
            {(t) => (
              <button
                class={`tab-btn ${activeTheme() === t.id ? 'active' : ''}`}
                style="display: flex; align-items: center; gap: 8px; padding: 6px 14px;"
                onClick={() => handleSelectTheme(t.id)}
              >
                <div
                  style={`width: 10px; height: 10px; border-radius: 50%; background: ${t.primary};`}
                />
                <span>{t.name}</span>
              </button>
            )}
          </For>
        </div>
      </SectionCard>

      {/* Toast & Notification Testing */}
      <SectionCard
        title="Toast Notification Testing"
        desc="Trigger floating toast notifications across all severity levels"
      >
        <div style="display: flex; gap: 8px; flex-wrap: wrap;">
          <button
            class="tab-btn"
            style="padding: 6px 12px; color: var(--accent-primary); border-color: var(--accent-glow);"
            onClick={() => toast.info('Info: render loop running')}
          >
            Trigger Info Toast
          </button>
          <button
            class="tab-btn"
            style="padding: 6px 12px; color: #10b981; border-color: rgba(16, 185, 129, 0.3);"
            onClick={() => toast.success('Success: preset configuration applied')}
          >
            Trigger Success Toast
          </button>
          <button
            class="tab-btn"
            style="padding: 6px 12px; color: #f97316; border-color: rgba(249, 115, 22, 0.3);"
            onClick={() => toast.warning('Warning: high trail length may impact low-end GPUs')}
          >
            Trigger Warning Toast
          </button>
          <button
            class="tab-btn"
            style="padding: 6px 12px; color: #ef4444; border-color: rgba(239, 68, 68, 0.3);"
            onClick={() => toast.error('Error: shader pipeline recompile fault')}
          >
            Trigger Error Toast
          </button>
        </div>
      </SectionCard>

      {/* Persistence & Factory Reset */}
      <SectionCard
        title="Persistence & Factory Reset"
        desc="Settings auto-save after every change; force a write now or restore pristine factory defaults"
      >
        <div style="display: flex; gap: 12px; flex-wrap: wrap;">
          <button
            class="tab-btn active"
            style="padding: 8px 16px;"
            onClick={() => void saveNow()}
            disabled={saving()}
          >
            {saving() ? 'Saving…' : 'Save Configuration Now'}
          </button>
          <button
            class="tab-btn"
            style="padding: 8px 16px; background: rgba(239, 68, 68, 0.1); border-color: rgba(239, 68, 68, 0.4); color: #ef4444;"
            onClick={() => {
              if (confirm('Are you sure you want to restore all settings to factory defaults?')) {
                props.onResetDefaults();
                toast.success('Configuration restored to factory defaults');
              }
            }}
          >
            Restore Factory Defaults
          </button>
        </div>
      </SectionCard>
    </div>
  );
};
