import { Component, createSignal, For } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';

export interface PresetItem {
  id: string;
  name: string;
  desc: string;
  badgeColor: string;
  apply: () => void;
}

/** Ids of presets saved by the user (deletable); built-ins use bare ids. */
export const USER_PRESET_PREFIX = 'user-';
export const isUserPreset = (id: string) => id.startsWith(USER_PRESET_PREFIX);

interface PresetsTabProps {
  currentPresetId: string;
  /** True when the live config no longer matches the selected preset's effect settings. */
  modified?: boolean;
  presets: PresetItem[];
  onApplyPreset: (id: string) => void;
  onExportJson: () => void;
  onImportJson: (json: string) => void;
  /** Saves the live configuration as a named user preset. */
  onSaveUserPreset?: (name: string) => void;
  onDeleteUserPreset?: (id: string) => void;
}

export const PresetsTab: Component<PresetsTabProps> = (props) => {
  const [importText, setImportText] = createSignal('');
  const [showImport, setShowImport] = createSignal(false);
  const [newName, setNewName] = createSignal('');

  const handleImportSubmit = () => {
    if (importText().trim()) {
      props.onImportJson(importText());
      setImportText('');
      setShowImport(false);
    }
  };

  const handleSaveUserPreset = () => {
    const name = newName().trim();
    if (!name) return;
    props.onSaveUserPreset?.(name);
    setNewName('');
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Save Current Look"
        desc="Store the live effect settings as your own preset (kept next to config.json, listed below the built-ins)"
      >
        <div style="display: flex; gap: 10px; flex-wrap: wrap; align-items: center;">
          <input
            type="text"
            placeholder="Preset name, e.g. Neon night"
            value={newName()}
            maxlength={48}
            onInput={(e) => setNewName(e.currentTarget.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') handleSaveUserPreset();
            }}
            style="flex: 1; min-width: 220px; background: #08080a; border: 1px solid var(--card-border); border-radius: 8px; color: var(--text-main); font-size: 13px; padding: 8px 12px; outline: none;"
          />
          <button
            class="tab-btn active"
            style="padding: 8px 16px; font-size: 13px;"
            disabled={!newName().trim() || !props.onSaveUserPreset}
            onClick={handleSaveUserPreset}
          >
            Save as Preset
          </button>
        </div>
      </SectionCard>

      <SectionCard
        title="Curated Visual Presets"
        desc="Instant one-click visual themes designed by the FXCursor visual team, followed by your own"
      >
        <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(280px, 1fr)); gap: 12px;">
          <For each={props.presets}>
            {(preset) => (
              <div
                class="settings-card"
                style={`border-color: ${
                  props.currentPresetId === preset.id
                    ? 'var(--accent-primary)'
                    : 'var(--card-border)'
                }; background: ${
                  props.currentPresetId === preset.id ? 'var(--accent-subtle)' : 'var(--card-bg)'
                }; cursor: pointer; transition: all 0.2s ease;`}
                onClick={() => props.onApplyPreset(preset.id)}
              >
                <div style="display: flex; align-items: center; justify-content: space-between; margin-bottom: 6px;">
                  <div style="font-weight: 700; font-size: 14px; color: var(--text-main); display: flex; align-items: center; gap: 8px;">
                    <span
                      style={`width: 10px; height: 10px; border-radius: 50%; background: ${preset.badgeColor}; box-shadow: 0 0 8px ${preset.badgeColor}; display: inline-block;`}
                    />
                    {preset.name}
                  </div>
                  {props.currentPresetId === preset.id && (
                    <span
                      style={`font-size: 10px; font-weight: 700; padding: 2px 8px; border-radius: 4px; ${
                        props.modified
                          ? 'color: #f97316; background: rgba(249, 115, 22, 0.12);'
                          : 'color: var(--accent-cyan); background: var(--accent-badge-bg);'
                      }`}
                      title={
                        props.modified
                          ? 'Effect settings differ from this preset. Apply it again to restore.'
                          : 'Effect settings match this preset.'
                      }
                    >
                      {props.modified ? 'MODIFIED' : 'ACTIVE'}
                    </span>
                  )}
                </div>
                <div style="font-size: 12px; line-height: 1.5; color: var(--text-muted);">
                  {preset.desc}
                </div>
                <div style="display: flex; gap: 8px; margin-top: 10px;">
                  <button
                    class="tab-btn"
                    style="flex: 1; text-align: center; justify-content: center; background: rgba(255,255,255,0.06);"
                    onClick={(e) => {
                      e.stopPropagation();
                      props.onApplyPreset(preset.id);
                    }}
                  >
                    {props.currentPresetId === preset.id ? 'Applied' : 'Apply Preset'}
                  </button>
                  {isUserPreset(preset.id) && props.onDeleteUserPreset && (
                    <button
                      class="tab-btn"
                      title="Delete this preset"
                      style="padding: 4px 12px; background: rgba(239, 68, 68, 0.12); color: #f87171;"
                      onClick={(e) => {
                        e.stopPropagation();
                        props.onDeleteUserPreset?.(preset.id);
                      }}
                    >
                      Delete
                    </button>
                  )}
                </div>
              </div>
            )}
          </For>
        </div>
      </SectionCard>

      <SectionCard
        title="Preset Backup & Sharing"
        desc="Export your current parameter tuning or import custom community themes"
      >
        <div style="display: flex; gap: 12px; flex-wrap: wrap;">
          <button
            class="tab-btn active"
            style="padding: 8px 16px; font-size: 13px;"
            onClick={() => props.onExportJson()}
          >
            Export Current Configuration (JSON)
          </button>
          <button
            class="tab-btn"
            style="padding: 8px 16px; font-size: 13px; background: rgba(255,255,255,0.06);"
            onClick={() => setShowImport(!showImport())}
          >
            {showImport() ? 'Cancel Import' : 'Import Preset JSON...'}
          </button>
        </div>

        {showImport() && (
          <div style="margin-top: 12px; display: flex; flex-direction: column; gap: 8px;">
            <textarea
              rows={4}
              placeholder="Paste JSON configuration payload here..."
              value={importText()}
              onInput={(e) => setImportText(e.currentTarget.value)}
              style="width: 100%; background: #08080a; border: 1px solid var(--card-border); border-radius: 8px; color: var(--text-main); font-family: monospace; font-size: 12px; padding: 10px; outline: none;"
            />
            <button
              class="tab-btn active"
              style="align-self: flex-start; padding: 6px 14px; font-size: 12px;"
              onClick={handleImportSubmit}
            >
              Apply Imported JSON
            </button>
          </div>
        )}
      </SectionCard>
    </div>
  );
};
