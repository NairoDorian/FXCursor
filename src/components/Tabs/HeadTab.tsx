import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import { ColorPicker } from '../Common/ColorPicker';

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
}

export const HeadTab: Component<HeadTabProps> = (props) => {
  const update = (patch: Partial<HeadConfig>) => {
    props.onChange({ ...props.head, ...patch });
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Squishy Cursor Head (Organic SDF)"
        desc="Analytical Signed Distance Field ellipse with velocity-directed deformation and angular normalization"
        headerRight={
          <Toggle checked={props.head.enabled} onChange={(v) => update({ enabled: v })} />
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
          onChange={(v) => update({ size: v })}
        />
        <Slider
          label="Squish Intensity (Velocity Stretch)"
          sub="Elongation deformation strength along velocity vector (0.0x - 10.0x)"
          min={0}
          max={10.0}
          step={0.1}
          unit="x"
          value={props.head.squish_intensity}
          onChange={(v) => update({ squish_intensity: v })}
        />
        <Slider
          label="Squish Smoothing"
          sub="Interpolation return speed back to rest circle (5 - 200)"
          min={5}
          max={200}
          step={1}
          value={props.head.squish_smoothing}
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
    </div>
  );
};
