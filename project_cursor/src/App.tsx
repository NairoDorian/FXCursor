import { useState, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import type { AppConfig } from "./lib/bindings";
import { defaultConfig } from "./lib/config";
import GeneralSettings from "./components/GeneralSettings";
import TrailPhysics from "./components/TrailPhysics";
import TrailLayers from "./components/TrailLayers";
import CursorHead from "./components/CursorHead";
import RipplesParticles from "./components/RipplesParticles";
import Satellites from "./components/Satellites";

export default function App() {
  const [config, setConfig] = useState<AppConfig>(defaultConfig);
  const [loading, setLoading] = useState(true);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    invoke<AppConfig>("get_config")
      .then(setConfig)
      .catch(console.error)
      .finally(() => setLoading(false));
  }, []);

  const updateConfig = useCallback(
    async (newConfig: AppConfig) => {
      setConfig(newConfig);
      try {
        await invoke("update_config", { config: newConfig });
      } catch (e) {
        console.error("Failed to update config:", e);
      }
    },
    [],
  );

  const handleSave = useCallback(async () => {
    try {
      await invoke("save_config");
      setSaved(true);
      setTimeout(() => setSaved(false), 2000);
    } catch (e) {
      console.error("Failed to save:", e);
    }
  }, []);

  const handleReset = useCallback(async () => {
    try {
      const fresh = await invoke<AppConfig>("reset_defaults");
      setConfig(fresh);
    } catch (e) {
      console.error("Failed to reset:", e);
    }
  }, []);

  if (loading) {
    return (
      <div className="flex h-screen items-center justify-center bg-gray-950">
        <div className="flex flex-col items-center gap-3">
          <div className="h-8 w-8 animate-spin rounded-full border-2 border-cyan-400 border-t-transparent" />
          <span className="text-sm text-gray-400">Loading configuration...</span>
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-screen flex-col bg-gray-950 text-gray-100 select-none">
      <header className="flex shrink-0 items-center justify-between border-b border-gray-800 px-4 py-3">
        <div className="flex items-center gap-2">
          <span className="text-lg font-bold tracking-tight text-cyan-400">
            CURSOR FX
          </span>
          <span className="rounded-full bg-cyan-400/10 px-2 py-0.5 text-[10px] font-medium text-cyan-400">
            ULTIMATE
          </span>
        </div>
        <div className="text-[11px] text-gray-500">WebGPU Multi-Layer FX Engine</div>
      </header>

      <main className="flex-1 overflow-y-auto px-3 py-2">
        <GeneralSettings config={config} onChange={updateConfig} />
        <TrailPhysics config={config} onChange={updateConfig} />
        <TrailLayers config={config} onChange={updateConfig} />
        <CursorHead config={config} onChange={updateConfig} />
        <RipplesParticles config={config} onChange={updateConfig} />
        <Satellites config={config} onChange={updateConfig} />
      </main>

      <footer className="shrink-0 border-t border-gray-800 px-4 py-3">
        <div className="flex gap-2">
          <button
            onClick={handleSave}
            className="flex-1 rounded-lg bg-cyan-500 px-4 py-2 text-sm font-semibold text-black transition hover:bg-cyan-400 active:scale-[0.98]"
          >
            {saved ? "SAVED!" : "SAVE SETTINGS"}
          </button>
          <button
            onClick={handleReset}
            className="rounded-lg border border-gray-700 px-4 py-2 text-sm font-medium text-gray-300 transition hover:border-gray-600 hover:bg-gray-800 active:scale-[0.98]"
          >
            RESET DEFAULTS
          </button>
        </div>
      </footer>
    </div>
  );
}
