import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Slider } from '../Common/Slider';
import { Toggle } from '../Common/Toggle';
import { ColorPicker } from '../Common/ColorPicker';

interface RippleConfig {
  enabled: boolean;
  max_diameter: number;
  duration_ms: number;
  start_width: number;
  color_left: [number, number, number, number];
  color_right: [number, number, number, number];
  color_middle: [number, number, number, number];
}

interface RipplesTabProps {
  ripple: RippleConfig;
  onChange: (newRipple: RippleConfig) => void;
}

export const RipplesTab: Component<RipplesTabProps> = (props) => {
  const update = (patch: Partial<RippleConfig>) => {
    props.onChange({ ...props.ripple, ...patch });
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      <SectionCard
        title="Click Shockwave Ripples"
        desc="Expanding concentric SDF ring shockwaves triggered on mouse button clicks"
        headerRight={
          <Toggle ariaLabel="Enable click ripples" checked={props.ripple.enabled} onChange={(v) => update({ enabled: v })} />
        }
      >
        <Slider
          label="Max Expansion Diameter"
          sub="Peak shockwave radius before fading out (20 - 400 px)"
          min={20}
          max={400}
          step={1}
          unit="px"
          value={props.ripple.max_diameter}
          onChange={(v) => update({ max_diameter: v })}
        />
        <Slider
          label="Shockwave Duration"
          sub="Lifespan of expanding ring (100 - 2500 ms)"
          min={100}
          max={2500}
          step={25}
          unit="ms"
          value={props.ripple.duration_ms}
          onChange={(v) => update({ duration_ms: v })}
        />
        <Slider
          label="Initial Ring Thickness"
          sub="Starting outline stroke width (1 - 30 px)"
          min={1}
          max={30}
          step={0.5}
          unit="px"
          value={props.ripple.start_width}
          onChange={(v) => update({ start_width: v })}
        />
      </SectionCard>

      <SectionCard
        title="Per-Button Shockwave Color Coding"
        desc="Distinct chromatic shockwaves mapped directly to Left, Right, and Middle mouse button clicks"
      >
        <ColorPicker
          label="Left Click Color"
          sub="Primary button shockwave"
          color={props.ripple.color_left}
          onChange={(c) => update({ color_left: c })}
        />
        <ColorPicker
          label="Right Click Color"
          sub="Secondary context button shockwave"
          color={props.ripple.color_right}
          onChange={(c) => update({ color_right: c })}
        />
        <ColorPicker
          label="Middle Click Color"
          sub="Scroll wheel button shockwave"
          color={props.ripple.color_middle}
          onChange={(c) => update({ color_middle: c })}
        />
      </SectionCard>
    </div>
  );
};
