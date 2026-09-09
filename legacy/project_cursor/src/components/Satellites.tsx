import { useState } from "react";
import type { AppConfig } from "../lib/bindings";
import ColorPicker from "./ColorPicker";

interface Props {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export default function Satellites({ config, onChange }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <section className="mb-3 rounded-xl border border-gray-800 bg-gray-900/50 p-3">
      <button
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-gray-500"
      >
        <span>Mirrored Orbiting Satellites</span>
        <span className="text-gray-600">{open ? "-" : "+"}</span>
      </button>

      {open && (
        <div className="mt-3 space-y-1">
          <label className="flex items-center gap-2 text-sm cursor-pointer mb-2">
            <input
              type="checkbox"
              checked={config.satellite_enabled}
              onChange={(e) =>
                onChange({ ...config, satellite_enabled: e.target.checked })
              }
              className="h-4 w-4 accent-cyan-500"
            />
            <span>Enable Satellites</span>
          </label>

          {config.satellite_enabled && (
            <>
              <label className="flex items-center gap-2 text-[11px] cursor-pointer mb-2">
                <input
                  type="checkbox"
                  checked={config.satellite_filled}
                  onChange={(e) =>
                    onChange({ ...config, satellite_filled: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-cyan-500"
                />
                <span>Filled (vs Outline)</span>
              </label>

              <ColorPicker
                label="Satellite Color"
                color={config.satellite_color}
                onChange={(c) => onChange({ ...config, satellite_color: c })}
              />

              <Slider label="Count" value={config.satellite_count} min={1} max={12} step={1} onChange={(v) => onChange({ ...config, satellite_count: Math.round(v) })} />
              <Slider label="Orbit Diameter" value={config.satellite_orbit_diameter} min={10} max={400} unit=" px" onChange={(v) => onChange({ ...config, satellite_orbit_diameter: v })} />
              <Slider label="Satellite Size" value={config.satellite_size} min={2} max={50} unit=" px" onChange={(v) => onChange({ ...config, satellite_size: v })} />

              {!config.satellite_filled && (
                <Slider label="Outline Width" value={config.satellite_outline_width} min={1} max={10} unit=" px" onChange={(v) => onChange({ ...config, satellite_outline_width: v })} />
              )}

              <Slider label="Orbit Speed" value={config.satellite_speed} min={-20} max={20} step={0.1} unit=" deg/frame" onChange={(v) => onChange({ ...config, satellite_speed: v })} />

              <hr className="my-2 border-gray-800" />

              <label className="flex items-center gap-2 text-[11px] cursor-pointer mb-1">
                <input
                  type="checkbox"
                  checked={config.satellite_enable_dual_ring}
                  onChange={(e) =>
                    onChange({ ...config, satellite_enable_dual_ring: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-cyan-500"
                />
                <span>Mirrored Dual Ring</span>
              </label>

              {config.satellite_enable_dual_ring && (
                <Slider label="Mirrored Speed" value={config.satellite_dual_speed} min={-20} max={20} step={0.1} unit=" deg/frame" onChange={(v) => onChange({ ...config, satellite_dual_speed: v })} />
              )}

              <hr className="my-2 border-gray-800" />

              <label className="flex items-center gap-2 text-[11px] cursor-pointer mb-1">
                <input
                  type="checkbox"
                  checked={config.satellite_show_orbit_ring}
                  onChange={(e) =>
                    onChange({ ...config, satellite_show_orbit_ring: e.target.checked })
                  }
                  className="h-3.5 w-3.5 accent-cyan-500"
                />
                <span>Show Orbit Ring</span>
              </label>

              {config.satellite_show_orbit_ring && (
                <>
                  <ColorPicker
                    label="Ring Color"
                    color={config.satellite_ring_color}
                    onChange={(c) => onChange({ ...config, satellite_ring_color: c })}
                  />
                  <Slider label="Ring Width" value={config.satellite_ring_width} min={1} max={10} unit=" px" onChange={(v) => onChange({ ...config, satellite_ring_width: v })} />
                </>
              )}
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
