import { useState } from "react";
import type { AppConfig } from "../lib/bindings";
import ColorPicker from "./ColorPicker";

interface Props {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export default function RipplesParticles({ config, onChange }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <section className="mb-3 rounded-xl border border-gray-800 bg-gray-900/50 p-3">
      <button
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-gray-500"
      >
        <span>Click Ripples & Particles</span>
        <span className="text-gray-600">{open ? "-" : "+"}</span>
      </button>

      {open && (
        <div className="mt-3 space-y-2">
          <div className="space-y-1">
            <h4 className="text-[11px] font-medium text-gray-400">SDF Click Ripples</h4>
            <ColorPicker label="Left Click" color={config.ripple_left_color} onChange={(c) => onChange({ ...config, ripple_left_color: c })} />
            <ColorPicker label="Right Click" color={config.ripple_right_color} onChange={(c) => onChange({ ...config, ripple_right_color: c })} />
            <ColorPicker label="Middle Click" color={config.ripple_middle_color} onChange={(c) => onChange({ ...config, ripple_middle_color: c })} />

            <Slider label="Ripple Size" value={config.ripple_radius} min={20} max={400} unit=" px" onChange={(v) => onChange({ ...config, ripple_radius: v })} />
            <Slider label="Ripple Duration" value={config.ripple_duration} min={0.1} max={3} unit=" s" onChange={(v) => onChange({ ...config, ripple_duration: v })} />
            <Slider label="Start Width" value={config.ripple_start_width} min={0.5} max={50} unit=" px" onChange={(v) => onChange({ ...config, ripple_start_width: v })} />
          </div>

          <hr className="my-2 border-gray-800" />

          <div>
            <label className="flex items-center gap-2 text-sm cursor-pointer mb-2">
              <input
                type="checkbox"
                checked={config.particle_enabled}
                onChange={(e) =>
                  onChange({ ...config, particle_enabled: e.target.checked })
                }
                className="h-4 w-4 accent-cyan-500"
              />
              <span>Enable Particles</span>
            </label>

            {config.particle_enabled && (
              <div className="space-y-1 pl-2">
                <Slider label="Count" value={config.particle_count} min={4} max={64} step={1} onChange={(v) => onChange({ ...config, particle_count: Math.round(v) })} />
                <Slider label="Ejection Speed" value={config.particle_speed} min={50} max={1000} step={1} onChange={(v) => onChange({ ...config, particle_speed: v })} />
                <Slider label="Lifetime" value={config.particle_lifetime} min={0.1} max={3} unit=" s" onChange={(v) => onChange({ ...config, particle_lifetime: v })} />
                <Slider label="Size" value={config.particle_size} min={1} max={20} unit=" px" onChange={(v) => onChange({ ...config, particle_size: v })} />
                <Slider label="Friction" value={config.particle_friction} min={50} max={99} step={1} unit="%" onChange={(v) => onChange({ ...config, particle_friction: v })} />
                <Slider label="Gravity" value={config.particle_gravity} min={-500} max={1000} step={1} onChange={(v) => onChange({ ...config, particle_gravity: v })} />
              </div>
            )}
          </div>
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
