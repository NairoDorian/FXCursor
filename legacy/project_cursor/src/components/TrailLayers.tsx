import { useState } from "react";
import type { AppConfig } from "../lib/bindings";
import ColorPicker from "./ColorPicker";

interface Props {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export default function TrailLayers({ config, onChange }: Props) {
  const [open, setOpen] = useState(false);

  const updateLayer = (idx: number, updater: (layer: AppConfig["layers"][0]) => AppConfig["layers"][0]) => {
    const layers = [...config.layers] as AppConfig["layers"];
    layers[idx] = updater({ ...layers[idx] });
    onChange({ ...config, layers });
  };

  return (
    <section className="mb-3 rounded-xl border border-gray-800 bg-gray-900/50 p-3">
      <button
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-gray-500"
      >
        <span>Ribbon Trail Layers</span>
        <span className="text-gray-600">{open ? "-" : "+"}</span>
      </button>

      {open && (
        <div className="mt-3 space-y-2">
          {config.layers.map((layer, i) => (
            <div key={i} className="rounded-lg border border-gray-800 bg-gray-900 p-2">
              <div className="flex items-center gap-3 mb-1.5">
                <label className="flex items-center gap-1.5 text-[11px] cursor-pointer">
                  <input
                    type="checkbox"
                    checked={layer.enabled}
                    onChange={(e) =>
                      updateLayer(i, (l) => ({ ...l, enabled: e.target.checked }))
                    }
                    className="h-3.5 w-3.5 accent-cyan-500"
                  />
                  <span className="text-gray-300 font-medium">Layer {i + 1}</span>
                </label>
              </div>

              {layer.enabled && (
                <div className="space-y-1 pl-2">
                  <ColorPicker
                    label="Start Color"
                    color={layer.start_color}
                    onChange={(c) => updateLayer(i, (l) => ({ ...l, start_color: c }))}
                  />
                  {config.enable_gradient && (
                    <ColorPicker
                      label="End Color"
                      color={layer.end_color}
                      onChange={(c) => updateLayer(i, (l) => ({ ...l, end_color: c }))}
                    />
                  )}

                  <SliderInput
                    label="Width Factor"
                    value={layer.width_factor}
                    min={0.05}
                    max={3}
                    unit="x"
                    onChange={(v) => updateLayer(i, (l) => ({ ...l, width_factor: v }))}
                  />
                  <SliderInput
                    label="Opacity Factor"
                    value={layer.alpha_factor}
                    min={0}
                    max={1}
                    onChange={(v) => updateLayer(i, (l) => ({ ...l, alpha_factor: v }))}
                  />
                  <SliderInput
                    label="Start Blur"
                    value={layer.start_blur}
                    min={0}
                    max={1}
                    onChange={(v) => updateLayer(i, (l) => ({ ...l, start_blur: v }))}
                  />
                  <SliderInput
                    label="End Blur"
                    value={layer.end_blur}
                    min={0}
                    max={1}
                    onChange={(v) => updateLayer(i, (l) => ({ ...l, end_blur: v }))}
                  />
                </div>
              )}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

function SliderInput({
  label,
  value,
  min,
  max,
  step = 0.01,
  unit = "",
  onChange,
}: {
  label: string;
  value: number;
  min: number;
  max: number;
  step?: number;
  unit?: string;
  onChange: (v: number) => void;
}) {
  return (
    <div className="mb-0.5">
      <div className="flex justify-between text-[10px] text-gray-500">
        <span>{label}</span>
        <span className="font-mono text-cyan-400/80">
          {value.toFixed(2)}
          {unit}
        </span>
      </div>
      <input
        type="range"
        min={min}
        max={max}
        step={step}
        value={value}
        onChange={(e) => onChange(parseFloat(e.target.value))}
        className="w-full h-1 bg-gray-700 rounded-full appearance-none cursor-pointer accent-cyan-500"
      />
    </div>
  );
}
