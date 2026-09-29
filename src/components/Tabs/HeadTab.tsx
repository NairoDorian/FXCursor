import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import { ColorPicker } from '../Common/ColorPicker';
import type { GpuCursorConfig } from '../../lib/bindings';
import { getDefaultConfig } from '../../lib/presets';
import { isModified, resetFields } from '../../lib/reset';

const DEFAULTS = getDefaultConfig();
const HEAD_DEFAULTS = DEFAULTS.head;
const GPU_DEFAULTS = DEFAULTS.gpu_cursor as GpuCursorConfig;
const HEAD_KEYS = ['size', 'squish_intensity', 'squish_smoothing', 'thickness'] as const;
const GPU_KEYS = [
  'hide_system_cursor',
  'rotate_with_movement',
  'rotation_smoothing',
  'click_scale_percent',
  'click_scale_duration_ms',
] as const;

interface HeadConfig {
  enabled: boolean;
  size: number;
  squish_intensity: number;
  squish_smoothing: number;
  color: [number, number, number, number];
  filled: boolean;
  thickness: number;
}

interface HeadTabProps {
  head: HeadConfig;
  onChange: (newHead: HeadConfig) => void;
  /** `config.gpu_cursor`: GPU-rendered system-cursor bypass. */
  gpuCursor: GpuCursorConfig;
  onGpuCursorChange: (next: GpuCursorConfig) => void;
}

export const HeadTab: Component<HeadTabProps> = (props) => {
  const update = (patch: Partial<HeadConfig>) => {
    props.onChange({ ...props.head, ...patch });
  };
  const updateGpu = (patch: Partial<GpuCursorConfig>) => {
    props.onGpuCursorChange({ ...props.gpuCursor, ...patch });
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Squishy Cursor Head (Organic SDF)"
        desc="Analytical Signed Distance Field ellipse with velocity-directed deformation and angular normalization"
        modified={isModified(props.head, HEAD_DEFAULTS, HEAD_KEYS)}
        onReset={() => props.onChange(resetFields(props.head, HEAD_DEFAULTS, HEAD_KEYS))}
        headerRight={
          <Toggle ariaLabel="Enable squishy head" checked={props.head.enabled} onChange={(v) => update({ enabled: v })} />
        }
      >
        <ColorPicker
          label="Cursor Head Color"
          sub="Tip tint color with alpha transparency"
          color={props.head.color}
          onChange={(c) => update({ color: c })}
        />
        <Slider
          label="Base Diameter"
          sub="Resting circular head diameter (4 - 80 px)"
          min={4}
          max={80}
          step={1}
          unit="px"
          value={props.head.size}
          defaultValue={HEAD_DEFAULTS.size}
          onChange={(v) => update({ size: v })}
        />
        <Slider
          label="Squish Intensity (Velocity Stretch)"
          sub="Elongation deformation strength along the velocity vector (0 = rigid, 1 = extreme)"
          min={0}
          max={1}
          step={0.005}
          value={props.head.squish_intensity}
          defaultValue={HEAD_DEFAULTS.squish_intensity}
          onChange={(v) => update({ squish_intensity: v })}
        />
        <Slider
          label="Squish Smoothing"
          sub="Fraction of the remaining gap the blob closes per 1/120 s (0 = frozen, 1 = locked to the pointer)"
          min={0}
          max={1}
          step={0.005}
          value={props.head.squish_smoothing}
          defaultValue={HEAD_DEFAULTS.squish_smoothing}
          onChange={(v) => update({ squish_smoothing: v })}
        />

        <div class="control-row">
          <div class="control-label">
            <span>Filled Body</span>
            <span class="control-sub">Solid filled circle vs hollow outline ring</span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              aria-label="Filled Body"
              checked={props.head.filled}
              onChange={(e) => {
                const filled = e.currentTarget.checked;
                update({
                  filled,
                  thickness: filled ? -1.0 : props.head.thickness > 0 ? props.head.thickness : 2.0,
                });
              }}
            />
            <span class="slider-round" />
          </label>
        </div>

        {!props.head.filled && (
          <Slider
            label="Outline Ring Thickness"
            sub="Width of hollow ring border (0.5 - 12 px)"
            min={0.5}
            max={12}
            step={0.5}
            unit="px"
            value={props.head.thickness > 0 ? props.head.thickness : 2.0}
            onChange={(v) => update({ thickness: v })}
          />
        )}
      </SectionCard>

      <SectionCard
        title="GPU Cursor Bypass"
        desc="Extracts the active system cursor shape and redraws it on the overlay with movement rotation and a click bounce; can hide the real cursor while active"
        modified={isModified(props.gpuCursor, GPU_DEFAULTS, GPU_KEYS)}
        onReset={() => props.onGpuCursorChange(resetFields(props.gpuCursor, GPU_DEFAULTS, GPU_KEYS))}
        headerRight={
          <Toggle
            ariaLabel="Enable GPU cursor bypass"
            checked={props.gpuCursor.enabled}
            onChange={(v) => updateGpu({ enabled: v })}
          />
        }
      >
        <div class="control-row">
          <div class="control-label">
            <span>Hide System Cursor</span>
            <span class="control-sub">
              Replaces the OS arrow with an invisible one while the bypass runs (restored on
              disable, exit and crash)
            </span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              aria-label="Hide System Cursor"
              checked={props.gpuCursor.hide_system_cursor}
              onChange={(e) => updateGpu({ hide_system_cursor: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>

        <div class="control-row">
          <div class="control-label">
            <span>Rotate With Movement</span>
            <span class="control-sub">
              Point the arrow along the direction of travel (non-arrow shapes stay upright)
            </span>
          </div>
          <label class="switch">
            <input
              type="checkbox"
              aria-label="Rotate With Movement"
              checked={props.gpuCursor.rotate_with_movement}
              onChange={(e) => updateGpu({ rotate_with_movement: e.currentTarget.checked })}
            />
            <span class="slider-round" />
          </label>
        </div>

        <Slider
          label="Rotation Smoothing"
          sub="Direction-averaging window: higher tracks with less lag (1 - 30 frames)"
          min={1}
          max={30}
          step={1}
          unit="fr"
          value={props.gpuCursor.rotation_smoothing}
          defaultValue={GPU_DEFAULTS.rotation_smoothing}
          onChange={(v) => updateGpu({ rotation_smoothing: v })}
        />
        <Slider
          label="Click Bounce Scale"
          sub="Peak size during the click bump (100 = no bounce)"
          min={100}
          max={300}
          step={5}
          format={(v) => `${Math.round(v)}%`}
          value={props.gpuCursor.click_scale_percent}
          defaultValue={GPU_DEFAULTS.click_scale_percent}
          onChange={(v) => updateGpu({ click_scale_percent: v })}
        />
        <Slider
          label="Click Bounce Duration"
          sub="Length of the click bump animation"
          min={50}
          max={500}
          step={10}
          unit="ms"
          value={props.gpuCursor.click_scale_duration_ms}
          defaultValue={GPU_DEFAULTS.click_scale_duration_ms}
          onChange={(v) => updateGpu({ click_scale_duration_ms: v })}
        />
      </SectionCard>
    </div>
  );
};
