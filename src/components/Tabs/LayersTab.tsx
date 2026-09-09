import { Component } from 'solid-js';
import { SectionCard } from '../Common/SectionCard';
import { Toggle } from '../Common/Toggle';
import { Slider } from '../Common/Slider';
import { ColorPicker } from '../Common/ColorPicker';

interface LayerConfig {
  enabled: boolean;
  start_color: [number, number, number, number];
  end_color: [number, number, number, number];
  width_factor: number;
  alpha_factor: number;
  start_blur: number;
  end_blur: number;
}

interface LayersTabProps {
  layers: [LayerConfig, LayerConfig, LayerConfig, LayerConfig];
  onUpdateLayer: (index: number, newLayer: LayerConfig) => void;
}

export const LayersTab: Component<LayersTabProps> = (props) => {
  const updateLayer = (index: number, patch: Partial<LayerConfig>) => {
    const updated = { ...props.layers[index], ...patch };
    props.onUpdateLayer(index, updated);
  };

  return (
    <div style="display: flex; flex-direction: column; gap: 16px;">
      {/* Layer 1: Outer Glow */}
      <SectionCard
        title="Layer 1: Outer Glow (Soft Glow)"
        desc="Feathered exterior border providing contrast across dark backdrops (150% width reference)"
        headerRight={
          <Toggle
            checked={props.layers[0].enabled}
            onChange={(v) => updateLayer(0, { enabled: v })}
          />
        }
      >
        <ColorPicker
          label="Glow Start Color"
          sub="Primary luminous tint at cursor tip"
          color={props.layers[0].start_color}
          onChange={(c) => updateLayer(0, { start_color: c })}
        />
        <ColorPicker
          label="Glow End Color (Gradient)"
          sub="Feathered fade tint at ribbon tail"
          color={props.layers[0].end_color}
          onChange={(c) => updateLayer(0, { end_color: c })}
        />
        <Slider
          label="Width Factor"
          sub="Expansion percentage relative to base cursor size (0% - 400%)"
          min={0}
          max={4.0}
          step={0.05}
          unit="%"
          value={props.layers[0].width_factor}
          onChange={(v) => updateLayer(0, { width_factor: v })}
        />
        <Slider
          label="Base Opacity"
          sub="Maximum layer opacity before velocity and length decay (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[0].alpha_factor}
          onChange={(v) => updateLayer(0, { alpha_factor: v })}
        />
        <Slider
          label="Head Feathering (Start Blur)"
          sub="Edge softness at cursor tip (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[0].start_blur}
          onChange={(v) => updateLayer(0, { start_blur: v })}
        />
        <Slider
          label="Tail Feathering (End Blur)"
          sub="Edge softness at ribbon tail (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[0].end_blur}
          onChange={(v) => updateLayer(0, { end_blur: v })}
        />
      </SectionCard>

      {/* Layer 2: Mid Outline */}
      <SectionCard
        title="Layer 2: Mid Outline (Dark Contrast Shadow)"
        desc="Dark contrast border preventing ribbon washout against pure white surfaces (90% width reference)"
        headerRight={
          <Toggle
            checked={props.layers[1].enabled}
            onChange={(v) => updateLayer(1, { enabled: v })}
          />
        }
      >
        <ColorPicker
          label="Outline Start Color"
          sub="Shadow outline tint at cursor tip"
          color={props.layers[1].start_color}
          onChange={(c) => updateLayer(1, { start_color: c })}
        />
        <ColorPicker
          label="Outline End Color (Gradient)"
          sub="Shadow fade tint at ribbon tail"
          color={props.layers[1].end_color}
          onChange={(c) => updateLayer(1, { end_color: c })}
        />
        <Slider
          label="Width Factor"
          sub="Expansion percentage relative to base cursor size (0% - 300%)"
          min={0}
          max={3.0}
          step={0.05}
          unit="%"
          value={props.layers[1].width_factor}
          onChange={(v) => updateLayer(1, { width_factor: v })}
        />
        <Slider
          label="Base Opacity"
          sub="Maximum layer opacity (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[1].alpha_factor}
          onChange={(v) => updateLayer(1, { alpha_factor: v })}
        />
        <Slider
          label="Head Feathering (Start Blur)"
          sub="Outline softness at cursor tip (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[1].start_blur}
          onChange={(v) => updateLayer(1, { start_blur: v })}
        />
        <Slider
          label="Tail Feathering (End Blur)"
          sub="Outline softness at ribbon tail (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[1].end_blur}
          onChange={(v) => updateLayer(1, { end_blur: v })}
        />
      </SectionCard>

      {/* Layer 3: Core */}
      <SectionCard
        title="Layer 3: Core (Crisp Body)"
        desc="Primary solid luminous interior ribbon (50% width reference)"
        headerRight={
          <Toggle
            checked={props.layers[2].enabled}
            onChange={(v) => updateLayer(2, { enabled: v })}
          />
        }
      >
        <ColorPicker
          label="Core Start Color"
          sub="Solid interior color at cursor tip"
          color={props.layers[2].start_color}
          onChange={(c) => updateLayer(2, { start_color: c })}
        />
        <ColorPicker
          label="Core End Color (Gradient)"
          sub="Solid interior color at ribbon tail"
          color={props.layers[2].end_color}
          onChange={(c) => updateLayer(2, { end_color: c })}
        />
        <Slider
          label="Width Factor"
          sub="Expansion percentage relative to base cursor size (0% - 200%)"
          min={0}
          max={2.0}
          step={0.05}
          unit="%"
          value={props.layers[2].width_factor}
          onChange={(v) => updateLayer(2, { width_factor: v })}
        />
        <Slider
          label="Base Opacity"
          sub="Maximum layer opacity (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[2].alpha_factor}
          onChange={(v) => updateLayer(2, { alpha_factor: v })}
        />
        <Slider
          label="Head Feathering (Start Blur)"
          sub="Core border sharpness at cursor tip (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[2].start_blur}
          onChange={(v) => updateLayer(2, { start_blur: v })}
        />
        <Slider
          label="Tail Feathering (End Blur)"
          sub="Core border sharpness at ribbon tail (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[2].end_blur}
          onChange={(v) => updateLayer(2, { end_blur: v })}
        />
      </SectionCard>

      {/* Layer 4: Inner Spine */}
      <SectionCard
        title="Layer 4: Inner Spine (Thin Precision Centerline)"
        desc="Ultra-thin centerline needle maintaining razor-sharp tracking alignment (15% width reference)"
        headerRight={
          <Toggle
            checked={props.layers[3].enabled}
            onChange={(v) => updateLayer(3, { enabled: v })}
          />
        }
      >
        <ColorPicker
          label="Center Spine Start Color"
          sub="Center needle color at cursor tip"
          color={props.layers[3].start_color}
          onChange={(c) => updateLayer(3, { start_color: c })}
        />
        <ColorPicker
          label="Center Spine End Color (Gradient)"
          sub="Center needle color at ribbon tail"
          color={props.layers[3].end_color}
          onChange={(c) => updateLayer(3, { end_color: c })}
        />
        <Slider
          label="Width Factor"
          sub="Centerline thickness percentage (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[3].width_factor}
          onChange={(v) => updateLayer(3, { width_factor: v })}
        />
        <Slider
          label="Base Opacity"
          sub="Maximum layer opacity (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[3].alpha_factor}
          onChange={(v) => updateLayer(3, { alpha_factor: v })}
        />
        <Slider
          label="Head Feathering (Start Blur)"
          sub="Spine border sharpness at cursor tip (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[3].start_blur}
          onChange={(v) => updateLayer(3, { start_blur: v })}
        />
        <Slider
          label="Tail Feathering (End Blur)"
          sub="Spine border sharpness at ribbon tail (0% - 100%)"
          min={0}
          max={1.0}
          step={0.01}
          unit="%"
          value={props.layers[3].end_blur}
          onChange={(v) => updateLayer(3, { end_blur: v })}
        />
      </SectionCard>
    </div>
  );
};
