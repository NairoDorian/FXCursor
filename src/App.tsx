import { createSignal, createEffect, onCleanup, onSettled, Show, For } from 'solid-js';
import { commands } from './lib/bindings';
import { attachBackendLogs } from './lib/console';
import { listen } from '@tauri-apps/api/event';
import { isTauri } from './lib/tauri';
import type { EffectMode } from './lib/bindings';
import { EFFECT_MODES } from './lib/effectMode';
import {
  AppConfig,
  BUILTIN_PRESETS,
  getDefaultConfig,
  getPresetById,
  LayerConfig,
} from './lib/presets';
import { toast } from './lib/toast';
import { applyThemeAccent } from './lib/theme';

import { LayersTab } from './components/Tabs/LayersTab';
import { TrailTab } from './components/Tabs/TrailTab';
import { HeadTab } from './components/Tabs/HeadTab';
import { RipplesTab } from './components/Tabs/RipplesTab';
import { ParticlesTab } from './components/Tabs/ParticlesTab';
import { SatellitesTab } from './components/Tabs/SatellitesTab';
import { PresetsTab, PresetItem } from './components/Tabs/PresetsTab';
import { HotkeysTab } from './components/Tabs/HotkeysTab';
import { DevConsoleTab } from './components/Tabs/DevConsoleTab';
import { DeveloperTab } from './components/Tabs/DeveloperTab';
import { AboutTab } from './components/Tabs/AboutTab';
import { LivePreview } from './components/Preview/LivePreview';
import { ToastContainer } from './components/Common/Toast';
import { ErrorBoundary } from './components/Common/ErrorBoundary';

type Tab =
  | 'layers'
  | 'trail'
  | 'head'
  | 'ripples'
  | 'particles'
  | 'satellites'
  | 'presets'
  | 'hotkeys'
  | 'console'
  | 'developer'
  | 'about';

export function AppContent() {
  const [activeTab, setActiveTab] = createSignal<Tab>('layers');
  const [config, setConfig] = createSignal<AppConfig>(getDefaultConfig());
  const [currentPresetId, setCurrentPresetId] = createSignal('master_4layer');

  let isInternalSync = false;
  let updateAnimId: number | undefined;
  let lastLocalEditTime = 0;

  const applyConfig = (newCfg: AppConfig) => {
    isInternalSync = true;
    setConfig(newCfg);
    if (newCfg.general?.selected_preset) {
      setCurrentPresetId(newCfg.general.selected_preset);
    }
    queueMicrotask(() => {
      isInternalSync = false;
    });
  };

  // In-window shortcuts (listed in the Hotkeys tab): Ctrl+S save, Ctrl+E toggle, Ctrl+1…9/0 tabs.
  const SHORTCUT_TABS: Tab[] = [
    'layers',
    'trail',
    'head',
    'ripples',
    'particles',
    'satellites',
    'presets',
    'hotkeys',
    'console',
    'developer',
  ];
  onSettled(() => {
    const onKey = (e: KeyboardEvent) => {
      if (!(e.ctrlKey || e.metaKey) || e.altKey) return;
      const key = e.key.toLowerCase();
      if (key === 's') {
        e.preventDefault();
        void handleSaveNow();
      } else if (key === 'e') {
        e.preventDefault();
        setConfig({ ...config(), enabled: !config().enabled });
        toast.info(`Effects ${config().enabled ? 'enabled' : 'disabled'}`);
      } else if (/^[0-9]$/.test(key)) {
        const index = key === '0' ? 9 : Number(key) - 1;
        const tab = SHORTCUT_TABS[index];
        if (tab) {
          e.preventDefault();
          setActiveTab(tab);
        }
      }
    };
    window.addEventListener('keydown', onKey);
    onCleanup(() => window.removeEventListener('keydown', onKey));
  });

  const handleSaveNow = async () => {
    if (!isTauri) {
      toast.info('Preview mode: nothing to save');
      return;
    }
    try {
      const path = await commands.saveConfig();
      toast.success(`Configuration saved to ${path}`);
    } catch (e) {
      toast.error(`Save failed: ${e}`);
    }
  };

  onSettled(() => {
    applyThemeAccent('cyan');

    if (!isTauri) {
      console.log('Running in browser / preview mode');
      return;
    }

    attachBackendLogs();
    (async () => {
      try {
        const remoteCfg = await commands.getConfig();
        if (remoteCfg) {
          applyConfig(remoteCfg);
        }
      } catch (e) {
        console.warn('Initial get_config error:', e);
      }

      try {
        await listen<AppConfig>('config-updated', (event) => {
          // Ignore echo events that arrived from our own active dragging (within 400ms)
          if (event.payload && performance.now() - lastLocalEditTime > 400) {
            applyConfig(event.payload);
          }
        });
      } catch (e) {
        console.warn('Event listener error:', e);
      }
    })();
  });

  // SolidJS 2.0 two-argument createEffect: (computeFn, effectFn)
  createEffect(
    () => config(),
    (cfg) => {
      if (!isTauri || isInternalSync) return;
      lastLocalEditTime = performance.now();
      if (updateAnimId) cancelAnimationFrame(updateAnimId);
      updateAnimId = requestAnimationFrame(() => {
        commands.updateConfig(cfg).catch((err) => {
          console.warn('Backend invoke error:', err);
        });
      });
    }
  );

  const handleUpdateLayer = (idx: number, newLayer: LayerConfig) => {
    const curr = config();
    const newLayers = [...curr.trail.layers] as [
      LayerConfig,
      LayerConfig,
      LayerConfig,
      LayerConfig,
    ];
    newLayers[idx] = newLayer;
    setConfig({
      ...curr,
      trail: {
        ...curr.trail,
        layers: newLayers,
      },
    });
  };

  const handleApplyPreset = async (presetId: string) => {
    if (isTauri) {
      // Rust is the source of truth for presets: the backend applies, persists and echoes.
      try {
        const applied = await commands.applyPreset(presetId);
        applyConfig(applied);
        toast.success(`Preset "${presetId}" applied and saved`);
      } catch (e) {
        toast.error(`Could not apply preset: ${e}`);
      }
      return;
    }
    const p = getPresetById(presetId);
    if (p) {
      setCurrentPresetId(presetId);
      setConfig({ ...p, general: { ...p.general, selected_preset: presetId } });
      toast.success(`Preset "${presetId}" applied (preview mode)`);
    }
  };

  const handleExportJson = () => {
    const json = JSON.stringify(config(), null, 2);
    navigator.clipboard
      .writeText(json)
      .then(() => toast.success('Configuration JSON copied to clipboard!'))
      .catch((e) => toast.error(`Clipboard unavailable: ${e}`));
  };

  const handleImportJson = async (jsonStr: string) => {
    if (isTauri) {
      // The backend runs the document through the self-healing deserializer, so partial or
      // slightly broken files still apply; it reports which fields it had to repair.
      try {
        const outcome = await commands.importConfig(jsonStr);
        applyConfig(outcome.config);
        if (outcome.repaired_paths.length > 0) {
          toast.warning(
            `Imported with ${outcome.repaired_paths.length} repaired field(s): ${outcome.repaired_paths
              .slice(0, 3)
              .join(', ')}${outcome.repaired_paths.length > 3 ? '…' : ''}`
          );
        } else {
          toast.success('Configuration imported and saved');
        }
      } catch (e) {
        toast.error(`Import failed: ${e}`);
      }
      return;
    }
    try {
      const parsed = JSON.parse(jsonStr);
      if (parsed && typeof parsed === 'object') {
        const def = getDefaultConfig();
        // Preview mode: merge one level deep so a partial section does not wipe its siblings.
        const merged: AppConfig = { ...def, ...parsed };
        for (const key of Object.keys(def) as (keyof AppConfig)[]) {
          const d = def[key];
          const p = (parsed as Record<string, unknown>)[key];
          if (d && typeof d === 'object' && p && typeof p === 'object' && !Array.isArray(d)) {
            (merged as unknown as Record<string, unknown>)[key] = { ...d, ...p };
          }
        }
        if (!Array.isArray(merged.trail.layers) || merged.trail.layers.length !== 4) {
          merged.trail.layers = def.trail.layers;
        }
        setConfig(merged);
        toast.success('Configuration imported (preview mode)');
      }
    } catch (e) {
      toast.error(`Invalid JSON format: ${e}`);
    }
  };

  const handleResetDefaults = async () => {
    if (isTauri) {
      try {
        const def = await commands.resetDefaults();
        applyConfig(def);
        return;
      } catch (e) {
        toast.error(`Reset failed: ${e}`);
      }
    }
    setConfig(getDefaultConfig());
    setCurrentPresetId('master_4layer');
  };

  const PRESET_BADGES: Record<string, string> = {
    master_4layer: 'var(--accent-primary)',
    neon_cyberpunk: '#a855f7',
    razor_spine: '#ffffff',
    celestial_orbit: '#eab308',
    particle_firestorm: '#f97316',
    rainbow_aurora: '#ec4899',
  };

  const localPresetList: PresetItem[] = BUILTIN_PRESETS.map((p) => ({
    id: p.id,
    name: p.name,
    desc: p.description,
    badgeColor: PRESET_BADGES[p.id] ?? 'var(--accent-primary)',
    apply: () => handleApplyPreset(p.id),
  }));
  const [presetList, setPresetList] = createSignal<PresetItem[]>(localPresetList);

  /** Effect-relevant part of a config (everything the presets define; not tray/hotkey/telemetry). */
  const effectPart = (c: AppConfig) =>
    JSON.stringify({
      effect_mode: c.effect_mode,
      trail: c.trail,
      head: c.head,
      ripple: c.ripple,
      particles: c.particles,
      satellites: c.satellites,
      rainbow: c.rainbow,
    });
  /** True when the user has edited effect settings since applying the selected preset. */
  const presetModified = () => {
    const preset = getPresetById(currentPresetId());
    return preset ? effectPart(preset) !== effectPart(config()) : false;
  };

  type PresetInfoLike = { id: string; name: string; description: string };
  const toPresetItems = (list: PresetInfoLike[]): PresetItem[] =>
    list.map((p) => ({
      id: p.id,
      name: p.name,
      desc: p.description || (p.id.startsWith('user-') ? 'Your saved preset' : ''),
      badgeColor: PRESET_BADGES[p.id] ?? (p.id.startsWith('user-') ? '#a78bfa' : 'var(--accent-primary)'),
      apply: () => handleApplyPreset(p.id),
    }));

  const refreshPresets = () =>
    commands
      .listPresets()
      .then((list) => setPresetList(toPresetItems(list)))
      .catch((e) => console.warn('list_presets failed, using bundled list:', e));

  onSettled(() => {
    if (!isTauri) return;
    void refreshPresets();
  });

  const handleSaveUserPreset = async (name: string) => {
    if (!isTauri) {
      toast.info('Preview mode: presets are saved by the desktop app');
      return;
    }
    try {
      const saved = await commands.saveUserPreset(name, '');
      applyConfig(saved.config);
      await refreshPresets();
      toast.success(`Preset "${saved.name}" saved`);
    } catch (e) {
      toast.error(`Could not save preset: ${e}`);
    }
  };

  const handleDeleteUserPreset = async (id: string) => {
    if (!isTauri) return;
    try {
      const list = await commands.deleteUserPreset(id);
      setPresetList(toPresetItems(list));
      toast.success('Preset deleted');
    } catch (e) {
      toast.error(`Could not delete preset: ${e}`);
    }
  };

  return (
    <div class="app-container">
      <ToastContainer />

      {/* Header */}
      <header class="app-header">
        <div class="brand-section">
          <div class="brand-icon">
            <svg
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
            >
              <polygon points="3 3 10 21 14 13 22 9 3 3" />
            </svg>
          </div>
          <div>
            <div class="brand-title">FXCursor</div>
            <span class="brand-version">
              D3D11 4-Layer Master Design (Direct3D 12 / Metal / Vulkan)
            </span>
          </div>
        </div>

        <div style="display: flex; align-items: center; gap: 12px;">
          <label
            style="display: flex; align-items: center; gap: 6px; font-size: 11px; color: var(--text-muted);"
            title="Effect mode: which subsystems render. Individual tab toggles still apply within the mode."
          >
            Mode
            <select
              value={config().effect_mode}
              onChange={(e) =>
                setConfig({ ...config(), effect_mode: e.currentTarget.value as EffectMode })
              }
              style="background: #121318; color: var(--text-main); border: 1px solid var(--card-border); border-radius: 6px; padding: 4px 8px; font-size: 12px; outline: none;"
            >
              <For each={EFFECT_MODES}>{(m) => <option value={m.id}>{m.label}</option>}</For>
            </select>
          </label>
          <label class="switch">
            <input
              type="checkbox"
              checked={config().enabled}
              onChange={(e) => setConfig({ ...config(), enabled: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>
      </header>

      {/* Tabs Navigation */}
      <nav class="tab-nav">
        <button
          class={`tab-btn ${activeTab() === 'layers' ? 'active' : ''}`}
          onClick={() => setActiveTab('layers')}
        >
          4-Layer Design
        </button>
        <button
          class={`tab-btn ${activeTab() === 'trail' ? 'active' : ''}`}
          onClick={() => setActiveTab('trail')}
        >
          Trail Physics
        </button>
        <button
          class={`tab-btn ${activeTab() === 'head' ? 'active' : ''}`}
          onClick={() => setActiveTab('head')}
        >
          Squishy Head
        </button>
        <button
          class={`tab-btn ${activeTab() === 'ripples' ? 'active' : ''}`}
          onClick={() => setActiveTab('ripples')}
        >
          Click Ripples
        </button>
        <button
          class={`tab-btn ${activeTab() === 'particles' ? 'active' : ''}`}
          onClick={() => setActiveTab('particles')}
        >
          Particles
        </button>
        <button
          class={`tab-btn ${activeTab() === 'satellites' ? 'active' : ''}`}
          onClick={() => setActiveTab('satellites')}
        >
          Satellites
        </button>
        <button
          class={`tab-btn ${activeTab() === 'presets' ? 'active' : ''}`}
          onClick={() => setActiveTab('presets')}
        >
          Presets
        </button>
        <button
          class={`tab-btn ${activeTab() === 'hotkeys' ? 'active' : ''}`}
          onClick={() => setActiveTab('hotkeys')}
        >
          Hotkeys & Tray
        </button>
        <button
          class={`tab-btn ${activeTab() === 'console' ? 'active' : ''}`}
          onClick={() => setActiveTab('console')}
        >
          Dev Console
        </button>
        <button
          class={`tab-btn ${activeTab() === 'developer' ? 'active' : ''}`}
          onClick={() => setActiveTab('developer')}
        >
          Developer Hub
        </button>
        <button
          class={`tab-btn ${activeTab() === 'about' ? 'active' : ''}`}
          onClick={() => setActiveTab('about')}
        >
          About
        </button>
      </nav>

      {/* Content & Live Preview */}
      <main class="app-content">
        <LivePreview config={config} />

        <Show when={activeTab() === 'layers'}>
          <LayersTab layers={config().trail.layers} onUpdateLayer={handleUpdateLayer} />
        </Show>

        <Show when={activeTab() === 'trail'}>
          <TrailTab
            trail={config().trail}
            onChange={(newTrail) => setConfig({ ...config(), trail: newTrail })}
          />
        </Show>

        <Show when={activeTab() === 'head'}>
          <HeadTab
            head={config().head}
            onChange={(newHead) => setConfig({ ...config(), head: newHead })}
          />
        </Show>

        <Show when={activeTab() === 'ripples'}>
          <RipplesTab
            ripple={config().ripple}
            onChange={(newRipple) => setConfig({ ...config(), ripple: newRipple })}
          />
        </Show>

        <Show when={activeTab() === 'particles'}>
          <ParticlesTab
            particles={config().particles}
            onChange={(newParticles) => setConfig({ ...config(), particles: newParticles })}
          />
        </Show>

        <Show when={activeTab() === 'satellites'}>
          <SatellitesTab
            satellites={config().satellites}
            onChange={(newSatellites) => setConfig({ ...config(), satellites: newSatellites })}
          />
        </Show>

        <Show when={activeTab() === 'presets'}>
          <PresetsTab
            currentPresetId={currentPresetId()}
            modified={presetModified()}
            presets={presetList()}
            onApplyPreset={handleApplyPreset}
            onExportJson={handleExportJson}
            onImportJson={handleImportJson}
            onSaveUserPreset={handleSaveUserPreset}
            onDeleteUserPreset={handleDeleteUserPreset}
          />
        </Show>

        <Show when={activeTab() === 'hotkeys'}>
          <HotkeysTab
            general={config().general}
            onChange={(newGen) => setConfig({ ...config(), general: newGen })}
          />
        </Show>

        <Show when={activeTab() === 'console'}>
          <DevConsoleTab />
        </Show>

        <Show when={activeTab() === 'developer'}>
          <DeveloperTab
            onResetDefaults={handleResetDefaults}
            fpsCounter={config().fps_counter}
            onFpsCounterChange={(next) => setConfig({ ...config(), fps_counter: next })}
          />
        </Show>

        <Show when={activeTab() === 'about'}>
          <AboutTab />
        </Show>
      </main>
    </div>
  );
}

export default function App() {
  return (
    <ErrorBoundary>
      <AppContent />
    </ErrorBoundary>
  );
}
