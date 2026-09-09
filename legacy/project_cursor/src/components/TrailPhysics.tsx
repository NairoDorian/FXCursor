import { useState } from "react";
import type { AppConfig } from "../lib/bindings";

interface Props {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
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
          {typeof value === "number" ? value.toFixed(step < 1 ? 1 : 0) : value}
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

export default function TrailPhysics({ config, onChange }: Props) {
  const [open, setOpen] = useState(false);

  return (
    <section className="mb-3 rounded-xl border border-gray-800 bg-gray-900/50 p-3">
      <button
        onClick={() => setOpen(!open)}
        className="flex w-full items-center justify-between text-xs font-semibold uppercase tracking-wider text-gray-500"
      >
        <span>Trail Physics Settings</span>
        <span className="text-gray-600">{open ? "-" : "+"}</span>
      </button>

      {open && (
        <div className="mt-3 space-y-1">
          <Slider
            label="Trail Length"
            value={config.trail_length}
            min={5}
            max={100}
            step={1}
            unit=" nodes"
            onChange={(v) => onChange({ ...config, trail_length: Math.round(v) })}
          />
          <Slider
            label="Base Size"
            value={config.trail_width}
            min={5}
            max={150}
            unit=" px"
            onChange={(v) => onChange({ ...config, trail_width: v })}
          />
          <Slider
            label="Min Tail Size"
            value={config.min_trail_width}
            min={1}
            max={20}
            unit=" px"
            onChange={(v) => onChange({ ...config, min_trail_width: v })}
          />

          <hr className="my-2 border-gray-800" />

          <Slider
            label="Head Spring"
            value={config.head_spring}
            min={1}
            max={500}
            step={1}
            unit=" strength"
            onChange={(v) => onChange({ ...config, head_spring: v })}
          />
          <Slider
            label="Head Friction"
            value={config.head_friction}
            min={0}
            max={99}
            step={1}
            unit="%"
            onChange={(v) => onChange({ ...config, head_friction: v })}
          />
          <Slider
            label="Body Spring"
            value={config.body_spring}
            min={1}
            max={500}
            step={1}
            unit=" strength"
            onChange={(v) => onChange({ ...config, body_spring: v })}
          />
          <Slider
            label="Body Friction"
            value={config.body_friction}
            min={0}
            max={99}
            step={1}
            unit="%"
            onChange={(v) => onChange({ ...config, body_friction: v })}
          />

          <hr className="my-2 border-gray-800" />

          <Slider
            label="Position Skip"
            value={config.position_skip}
            min={0}
            max={10}
            step={1}
            unit=" updates"
            onChange={(v) =>
              onChange({ ...config, position_skip: Math.round(v) })
            }
          />
          <Slider
            label="Spline Smoothness"
            value={config.interpolation_steps}
            min={1}
            max={20}
            step={1}
            unit=" steps"
            onChange={(v) =>
              onChange({ ...config, interpolation_steps: Math.round(v) })
            }
          />

          <div className="mt-2 flex flex-wrap gap-1">
            {[
              { value: 0, label: "Linear" },
              { value: 1, label: "Ease-Out" },
              { value: 2, label: "Expo" },
              { value: 3, label: "Sigmoid" },
            ].map(({ value, label }) => (
              <button
                key={value}
                onClick={() => onChange({ ...config, fade_mode: value })}
                className={`rounded px-2 py-1 text-[10px] font-medium transition ${
                  config.fade_mode === value
                    ? "bg-cyan-500/20 text-cyan-300"
                    : "bg-gray-800 text-gray-500 hover:bg-gray-700"
                }`}
              >
                {label}
              </button>
            ))}
          </div>

          <div className="mt-2 flex flex-wrap items-center gap-3">
            <label className="flex items-center gap-2 text-[11px] cursor-pointer">
              <input
                type="checkbox"
                checked={config.rainbow_mode}
                onChange={(e) =>
                  onChange({ ...config, rainbow_mode: e.target.checked })
                }
                className="h-3.5 w-3.5 accent-cyan-500"
              />
              <span>Rainbow Hue Cycle</span>
            </label>
            {config.rainbow_mode && (
              <Slider
                label="Speed"
                value={config.rainbow_speed}
                min={0.5}
                max={10}
                onChange={(v) => onChange({ ...config, rainbow_speed: v })}
              />
            )}

            <label className="flex items-center gap-2 text-[11px] cursor-pointer">
              <input
                type="checkbox"
                checked={config.adaptive_quality}
                onChange={(e) =>
                  onChange({ ...config, adaptive_quality: e.target.checked })
                }
                className="h-3.5 w-3.5 accent-cyan-500"
              />
              <span>Adaptive Quality</span>
            </label>

            <label className="flex items-center gap-2 text-[11px] cursor-pointer">
              <input
                type="checkbox"
                checked={config.enable_gradient}
                onChange={(e) =>
                  onChange({ ...config, enable_gradient: e.target.checked })
                }
                className="h-3.5 w-3.5 accent-cyan-500"
              />
              <span>Enable Gradient</span>
            </label>
          </div>

          <Slider
            label="Velocity Width Boost"
            value={config.velocity_width_multiplier}
            min={0}
            max={5}
            onChange={(v) =>
              onChange({ ...config, velocity_width_multiplier: v })
            }
          />
          <Slider
            label="Velocity Opacity Boost"
            value={config.velocity_alpha_multiplier}
            min={0}
            max={5}
            onChange={(v) =>
              onChange({ ...config, velocity_alpha_multiplier: v })
            }
          />
        </div>
      )}
    </section>
  );
}
