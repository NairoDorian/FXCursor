import { Component, createEffect, createSignal } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { commands, type GeneralConfig } from '../../lib/bindings';
import { isTauri } from '../../lib/tauri';

interface HotkeysTabProps {
  general: GeneralConfig;
  onChange: (newGeneral: GeneralConfig) => void;
}

export const HotkeysTab: Component<HotkeysTabProps> = (props) => {
  const update = <K extends keyof GeneralConfig>(field: K, val: GeneralConfig[K]) => {
    props.onChange({ ...props.general, [field]: val });
  };

  // Registration outcome of the shortcut (from the backend), refreshed shortly after each edit
  // so conflicts with other applications are visible right here.
  const [hotkeyStatus, setHotkeyStatus] = createSignal<string>('');
  const [refreshHz, setRefreshHz] = createSignal<number | null>(null);
  const refreshStatus = async () => {
    if (!isTauri) return;
    try {
      const d = await commands.getDiagnostics();
      setHotkeyStatus(d.hotkey_status);
      setRefreshHz(d.display_refresh_hz);
    } catch {
      // diagnostics unavailable; keep the last value
    }
  };
  createEffect(
    () => props.general.global_hotkey,
    () => {
      const id = setTimeout(() => void refreshStatus(), 600);
      // SolidJS 2: the effect returns its cleanup (runs before the next run / on dispose).
      return () => clearTimeout(id);
    }
  );
  const statusIsError = () => hotkeyStatus().startsWith('error');

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Global Keyboard Shortcuts"
        desc="System-wide hotkeys active across all applications and full-screen games"
      >
        <div class="control-row">
          <div class="control-label">
            <span>Toggle Effects Shortcut</span>
            <span class="control-sub">
              Registered with the OS on change. Format: modifiers + key, e.g. Ctrl+Shift+E, Alt+F9,
              CmdOrCtrl+Shift+X. Leave empty to disable.
            </span>
          </div>
          <div style="display: flex; align-items: center; gap: 8px;">
            <input
              type="text"
              value={props.general.global_hotkey}
              onInput={(e) => update('global_hotkey', e.currentTarget.value)}
              style="background: #08080a; border: 1px solid var(--card-border); border-radius: 6px; color: var(--accent-primary); font-family: monospace; font-weight: 700; font-size: 13px; padding: 6px 12px; width: 140px; text-align: center; outline: none;"
            />
          </div>
        </div>
      </SectionCard>

      <div
        style={`margin-top: -8px; font-size: 11px; font-family: monospace; color: ${
          statusIsError() ? '#ef4444' : 'var(--text-dim)'
        };`}
      >
        {isTauri
          ? hotkeyStatus()
            ? `Shortcut status: ${hotkeyStatus()}`
            : 'Shortcut status: …'
          : 'Shortcut registration is only available in the desktop app.'}
      </div>

      <SectionCard
        title="Performance"
        desc="Frame pacing for the overlay render loop. 0 follows the display refresh rate; higher caps trade CPU/GPU for lower latency."
      >
        <div class="control-row">
          <div class="control-label">
            <span>Frame rate limit</span>
            <span class="control-sub">
              {props.general.max_fps === 0
                ? `Display refresh rate${refreshHz() ? ` (${Math.round(refreshHz()!)} Hz)` : ''}`
                : `${props.general.max_fps} fps`}
            </span>
          </div>
          <input
            type="range"
            min="0"
            max="360"
            step="10"
            value={props.general.max_fps}
            onInput={(e) => update('max_fps', Number(e.currentTarget.value))}
            style="width: 220px;"
          />
        </div>
      </SectionCard>

      <SectionCard
        title="Application Lifecycle & System Resident Settings"
        desc="Startup, tray behavior, and background resident execution"
      >
        <div class="control-row">
          <div class="control-label">
            <span>Start on System Boot</span>
            <span class="control-sub">Launch FXCursor automatically when you log into Windows</span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              checked={props.general.autostart}
              onChange={(e) => update('autostart', e.currentTarget.checked)}
            />
            <span class="slider-round" />
          </label>
        </div>

        <div class="control-row">
          <div class="control-label">
            <span>Minimize to System Tray</span>
            <span class="control-sub">
              Closing settings window keeps effects running silently in tray
            </span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              checked={props.general.minimize_to_tray}
              onChange={(e) => update('minimize_to_tray', e.currentTarget.checked)}
            />
            <span class="slider-round" />
          </label>
        </div>

        <div class="control-row">
          <div class="control-label">
            <span>Start Minimized</span>
            <span class="control-sub">
              Launch directly to the system tray without opening settings
            </span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              checked={props.general.start_minimized}
              onChange={(e) => update('start_minimized', e.currentTarget.checked)}
            />
            <span class="slider-round" />
          </label>
        </div>
      </SectionCard>

      <SectionCard
        title="Studio Shortcuts"
        desc="Keyboard shortcuts that work while this window is focused (the global hotkey above works everywhere)"
      >
        <div style="display: grid; grid-template-columns: max-content 1fr; gap: 8px 18px; font-size: 12px; align-items: center;">
          {STUDIO_SHORTCUTS.map(([keys, what]) => (
            <>
              <kbd
                style="font-family: monospace; font-size: 11px; padding: 3px 8px; border-radius: 6px; background: rgba(255,255,255,0.08); border: 1px solid var(--card-border); color: var(--text-main);"
              >
                {keys}
              </kbd>
              <span style="color: var(--text-muted);">{what}</span>
            </>
          ))}
        </div>
      </SectionCard>
    </div>
  );
};

/** Shortcuts handled by `App.tsx` (keep in sync with its keydown handler). */
export const STUDIO_SHORTCUTS: [string, string][] = [
  ['Ctrl + S', 'Save the configuration to disk now'],
  ['Ctrl + E', 'Toggle the effects on / off'],
  ['Ctrl + 1 … 9', 'Switch tabs: Layers, Trail, Head, Ripples, Particles, Satellites, Presets, Hotkeys, Console'],
  ['Ctrl + 0', 'Developer Hub'],
  ['Ctrl + /', 'About FXCursor & Architecture Reference'],
];
