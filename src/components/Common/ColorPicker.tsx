import { Component } from 'solid-js';

interface ColorPickerProps {
  label: string;
  sub?: string;
  color: [number, number, number, number]; // RGBA [0..1, 0..1, 0..1, 0..1]
  onChange: (color: [number, number, number, number]) => void;
}

/** `#rrggbb` for an RGBA in 0–1. Channels are clamped: an imported value outside 0–1 used to
 * produce an invalid hex that the colour input silently ignored. */
function rgbaToHex(rgba: [number, number, number, number]): string {
  const channel = (v: number) =>
    Math.round(Math.min(1, Math.max(0, Number.isFinite(v) ? v : 0)) * 255)
      .toString(16)
      .padStart(2, '0');
  return `#${channel(rgba[0])}${channel(rgba[1])}${channel(rgba[2])}`;
}

function hexToRgba(hex: string, alpha: number): [number, number, number, number] {
  const clean = hex.replace('#', '');
  const r = parseInt(clean.substring(0, 2), 16) / 255 || 0;
  const g = parseInt(clean.substring(2, 4), 16) / 255 || 0;
  const b = parseInt(clean.substring(4, 6), 16) / 255 || 0;
  return [r, g, b, alpha];
}

export const ColorPicker: Component<ColorPickerProps> = (props) => {
  const hex = () => rgbaToHex(props.color);
  const alphaPct = () => Math.round(props.color[3] * 100);

  const handleHexChange = (e: Event & { currentTarget: HTMLInputElement }) => {
    const newRgba = hexToRgba(e.currentTarget.value, props.color[3]);
    props.onChange(newRgba);
  };

  const handleAlphaChange = (e: Event & { currentTarget: HTMLInputElement }) => {
    const a = Number(e.currentTarget.value) / 100;
    props.onChange([props.color[0], props.color[1], props.color[2], a]);
  };

  return (
    <div class="control-row">
      <div class="control-label">
        <span>{props.label}</span>
        {props.sub && <span class="control-sub">{props.sub}</span>}
      </div>
      <div style="display: flex; align-items: center; gap: 10px;">
        <input
          type="color"
          aria-label={`${props.label} colour`}
          value={hex()}
          onChange={handleHexChange}
          style="width: 32px; height: 32px; border-radius: 6px; border: 1px solid rgba(255,255,255,0.2); background: transparent; cursor: pointer; padding: 2px;"
        />
        <div style="display: flex; align-items: center; gap: 6px;">
          <input
            type="range"
            aria-label={`${props.label} opacity`}
            min="0"
            max="100"
            value={alphaPct()}
            onInput={handleAlphaChange}
            class="custom-slider"
            style="width: 70px;"
          />
          <span class="slider-val" style="min-width: 36px;">
            {alphaPct()}%
          </span>
        </div>
      </div>
    </div>
  );
};
