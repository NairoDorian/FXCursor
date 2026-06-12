import { useState } from "react";
import type { AppConfig } from "../lib/bindings";
import ColorPicker from "./ColorPicker";

interface Props {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export default function CursorHead({ config, onChange }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <section className="mb-3 rounded-xl border border-gray-800 bg-gray-900/50 p-3">
      <button
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-gray-500"
      >
        <span>Squishy Cursor Head</span>
        <span className="text-gray-600">{open ? "-" : "+"}</span>
      </button>

      {open && (
        <div className="mt-3 space-y-1">
          <label className="flex items-center gap-2 text-sm cursor-pointer mb-2">
            <input
              type="checkbox"
              checked={config.head_enabled}
              onChange={(e) =>
                onChange({ ...config, head_enabled: e.target.checked })
              }
              className="h-4 w-4 accent-cyan-500"
            />
            <span>Enable Cursor Head</span>
          </label>

          {config.head_enabled && (
            <>
              <label className="flex items-center gap-2 text-[11px] cursor-pointer mb-2">
                <input
                  type="checkbox"
                  checked={config.head_filled}
                  onChange={(e) =>
                    onChange({ ...config, head_filled: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-cyan-500"
                />
                <span>Filled (vs Outline)</span>
              </label>

              <ColorPicker
                label="Head Color"
                color={config.head_color}
                onChange={(c) => onChange({ ...config, head_color: c })}
              />

              <Slider
                label="Base Size"
                value={config.head_size}
                min={5}
                max={100}
                unit=" px"
                onChange={(v) => onChange({ ...config, head_size: v })}
              />

              {!config.head_filled && (
                <Slider
                  label="Outline Width"
                  value={config.head_outline_width}
                  min={1}
                  max={20}
                  unit=" px"
                  onChange={(v) =>
                    onChange({ ...config, head_outline_width: v })
                  }
                />
              )}

              <Slider
                label="Squish Intensity"
                value={config.head_squish_intensity}
                min={0}
                max={10}
                onChange={(v) =>
                  onChange({ ...config, head_squish_intensity: v })
                }
              />
              <Slider
                label="Squish Smoothing"
                value={config.head_squish_smoothing}
                min={10}
                max={100}
                step={1}
                unit="%"
                onChange={(v) =>
                  onChange({ ...config, head_squish_smoothing: v })
                }
              />
            </>
          )}
        </div>
      )}
    </section>
  );
}

function Slider({
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
    <div className="mb-1.5">
      <div className="flex justify-between text-[11px] text-gray-400 mb-0.5">
        <span>{label}</span>
        <span className="font-mono text-cyan-400">
          {value.toFixed(step < 1 ? 1 : 0)}
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
        className="w-full h-1.5 bg-gray-700 rounded-full appearance-none cursor-pointer accent-cyan-500"
      />
    </div>
  );
}
