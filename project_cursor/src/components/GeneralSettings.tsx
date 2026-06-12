import type { AppConfig } from "../lib/bindings";

interface Props {
  config: AppConfig;
  onChange: (config: AppConfig) => void;
}

export default function GeneralSettings({ config, onChange }: Props) {
  return (
    <section className="mb-3 rounded-xl border border-gray-800 bg-gray-900/50 p-3">
      <h3 className="mb-3 text-xs font-semibold uppercase tracking-wider text-gray-500">
        General Setup
      </h3>

      <div className="flex flex-wrap items-center gap-3">
        <label className="flex items-center gap-2 text-sm cursor-pointer">
          <input
            type="checkbox"
            checked={config.enabled}
            onChange={(e) => onChange({ ...config, enabled: e.target.checked })}
            className="h-4 w-4 accent-cyan-500"
          />
          <span>Enable Cursor FX</span>
        </label>

        <label className="flex items-center gap-2 text-sm cursor-pointer">
          <input
            type="checkbox"
            checked={config.click_response}
            onChange={(e) =>
              onChange({ ...config, click_response: e.target.checked })
            }
            className="h-4 w-4 accent-cyan-500"
          />
          <span>Click Ripple</span>
        </label>
      </div>

      <div className="mt-3 flex gap-1">
        {[
          { value: 0, label: "Ribbon Trail" },
          { value: 1, label: "SDF Ripple Only" },
          { value: 2, label: "Glow Aura" },
        ].map(({ value, label }) => (
          <button
            key={value}
            onClick={() => onChange({ ...config, effect_type: value })}
            className={`rounded-md px-3 py-1.5 text-xs font-medium transition ${
              config.effect_type === value
                ? "bg-cyan-500/20 text-cyan-300 ring-1 ring-cyan-500/50"
                : "bg-gray-800 text-gray-400 hover:bg-gray-700"
            }`}
          >
            {label}
          </button>
        ))}
      </div>
    </section>
  );
}
