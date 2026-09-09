import fs from 'node:fs';
import path from 'node:path';
import {
  pack,
  searchFiles,
  generateTreeString,
  loadFileConfig,
  mergeConfigs,
  type PackResult,
} from 'repomix';

const fileDescriptions: Record<string, string> = {
  '.gitignore':
    'Git ignore configuration excluding build artifacts, node_modules, logs, and lockfiles.',
  '.prettierrc':
    'Prettier formatting configuration enforcing single quotes and 2-space indentation.',
  '.prettierignore':
    'Prettier ignore configuration excluding dist, target, node_modules, and logs.',
  '.oxlintrc.json':
    'oxlint (TS7-compatible linter) configuration for high-performance static analysis.',
  'package.json': 'Project manifest containing Bun scripts, SolidJS 2.0, and Tauri 2 dependencies.',
  'repomix.config.json': 'Repomix configuration for metadata-only architecture output.',
  'tsconfig.json':
    'TypeScript root configuration with strict type checking and bundler resolution.',
  'vite.config.ts':
    'Vite bundler configuration optimized for SolidJS 2 and Tauri v2 dev server integration.',
  'index.html': 'Main HTML entry point for the SolidJS 2 Studio dashboard.',
  'DOCUMENTATION.md':
    'Master upstream documentation map for Tauri 2, SolidJS 2, Bun, TypeScript 7, and wgpu.',
  'scripts/version.ts': 'Single global source of truth for the application version (APP_VERSION).',
  'scripts/before-commit.ts': '7-gate validation suite and multi-manifest version synchronizer.',
  'scripts/generate-arch.ts': 'Repomix pack() API-driven generator producing ARCHITECTURE.md.',
  'scripts/sync-docs.ts': 'Documentation mirror manager managing shallow git mirrors under .docs/.',
  'scripts/create-icons.ts': 'Cross-platform application icon validator and generator.',
  'scripts/update-deps.ts': 'End-to-end automated dual-ecosystem pre-release upgrade pipeline.',
  'scripts/package-portable.ts':
    'Builds the portable zip (exe + `portable` marker + Data/ + README) from the release build.',
  'src/main.tsx': 'SolidJS 2 application entry point rendering the root component.',
  'src/App.tsx':
    'Application shell: modular tab navigation, header, live ribbon preview, and toast container.',
  'src/lib/presets.ts':
    'Frontend presets registry, default configuration factory, and type definitions.',
  'src/lib/tauri.ts': 'Shared Tauri v2 runtime detection utility exporting isTauri.',
  'src/lib/theme.ts': 'Theme accent customization engine with 5 curated neon color palettes.',
  'src/lib/toast.ts': 'Reactive toast notification event bus and helper methods.',
  'src/lib/console.ts': 'In-memory dev-log event bus shared with DevConsole.',
  'src/components/Common/Slider.tsx':
    'Precision custom range slider with dynamic step calculation.',
  'src/components/Common/Toggle.tsx': 'Accessible switch toggle control.',
  'src/components/Common/ColorPicker.tsx': 'RGBA / Hex color picker with alpha slider.',
  'src/components/Common/SectionCard.tsx': 'Glassmorphic card container with header action slots.',
  'src/components/Common/Toast.tsx': 'Floating animated toast notification container.',
  'src/components/Common/ErrorBoundary.tsx':
    'Top-level SolidJS 2 crash handler with stack trace display.',
  'src/components/Preview/LivePreview.tsx':
    'Canvas mirror of the renderer: same physics, capsule-union ribbon (erase-then-paint), clicks, modes.',
  'src/components/Tabs/LayersTab.tsx':
    '4-layer master controls (Outer Glow, Mid Shadow, Crisp Core, Inner Spine).',
  'src/components/Tabs/TrailTab.tsx':
    'Kinematic physics sliders with independent head and body tension/damping.',
  'src/components/Tabs/HeadTab.tsx':
    'Squishy head SDF controls with velocity elongation and angle normalization.',
  'src/components/Tabs/RipplesTab.tsx':
    'Click shockwave concentric ring controls with per-button coloring.',
  'src/components/Tabs/ParticlesTab.tsx':
    'Kinematic particle bursts with gravity, drag friction, and alpha decay.',
  'src/components/Tabs/SatellitesTab.tsx':
    'Celestial satellite orbitals with counter-rotation dual ring support.',
  'src/components/Tabs/PresetsTab.tsx':
    'Preset selector cards and JSON configuration import/export.',
  'src/components/Tabs/HotkeysTab.tsx': 'Global hotkey bindings and autostart preferences.',
  'src/components/Tabs/DevConsoleTab.tsx':
    'Live diagnostic dev-console log viewer with filter badges.',
  'src/components/Tabs/DeveloperTab.tsx':
    'Developer Hub: IPC latency benchmark, system diagnostics, config path, theme accents.',
  'src/components/Tabs/AboutTab.tsx': 'Architecture overview and diagnostic details.',
  'src-tauri/Cargo.toml': 'Cargo manifest for the Tauri 2 desktop shell and overlay renderer.',
  'src-tauri/tauri.conf.json': 'Tauri 2 configuration defining window dimensions and capabilities.',
  'src-tauri/src/lib.rs':
    'Tauri entry: persisted config state, IPC commands, tray, hotkey/autostart sync, overlay thread.',
  'src-tauri/src/main.rs': 'Main Rust entry point launching the desktop application.',
  'src-tauri/src/panic_log.rs': 'Panic hook logger saving crash dumps to panic.log.',
  'src-tauri/src/portable.rs':
    'Zero-config portable mode detector for USB/isolated directory runs.',
  'src-tauri/src/settings_repair.rs':
    'Persistent config store: self-healing load, atomic save, debounced auto-saver.',
  'src-tauri/src/integrations.rs':
    'OS integrations synced from config: global toggle hotkey and login autostart.',
  'src-tauri/capabilities/default.json':
    'Tauri 2 ACL capability granting core/event/window permissions to main and overlay windows.',
  'src-tauri/src/tracker.rs':
    'device_query mouse poller used on non-Windows platforms (Windows uses input/windows.rs).',
  'src-tauri/src/input/mod.rs':
    'InputHub: event-driven cursor/button/click source with condvar wake for the render thread.',
  'src-tauri/src/input/windows.rs':
    'WH_MOUSE_LL low-level mouse hook thread + GetCursorPos UIPI fallback poll.',
  'src-tauri/src/logger.rs':
    'Log bridge: env_logger + ring buffer + `rust-log` event stream to the Dev Console.',
  'src-tauri/src/capture.rs':
    'Overlay snapshot: offscreen render + readback to PNG (IPC command, --capture CLI flag, Developer Hub).',
  'src/lib/bindings.ts':
    'AUTO-GENERATED by tauri-specta: typed `commands` and config types. Do not edit.',
  'test/config-parity.test.ts':
    'Asserts the TypeScript default config equals the Rust-generated fixture.',
  'test/effect-mode.test.ts':
    'Asserts the TypeScript modeMask() equals the Rust-generated mode_masks.json fixture.',
  'test/fixtures/mode_masks.json':
    'ModeMask::from_mode snapshot for every EffectMode (bun run fixtures); shared parity fixture.',
  'src/lib/effectMode.ts':
    'TypeScript mirror of the renderer ModeMask plus the header effect-mode list.',
  'crates/fxcursor-render/examples/dump_mode_masks.rs':
    'Prints ModeMask::from_mode for every EffectMode as JSON; used by `bun run fixtures`.',
  'crates/fxcursor-render/tests/mode_mask_fixture.rs':
    'Rust side of the effect-mode parity check against test/fixtures/mode_masks.json.',
  'test/fixtures/default_config.json':
    'Rust AppConfig::default() snapshot (bun run fixtures) shared by Rust and Bun parity tests.',
  'crates/fxcursor-protocol/tests/fixture_parity.rs':
    'Rust side of the default-config parity check against test/fixtures.',
  'crates/fxcursor-protocol/examples/dump_default.rs':
    'Prints AppConfig::default() as JSON; used by `bun run fixtures`.',
  'crates/fxcursor-protocol/src/self_healing.rs':
    'Field-level self-healing JSON deserializer (serde_path_to_error) with repair log.',
  'crates/fxcursor-protocol/src/lib.rs':
    'Protocol crate root re-exporting config, presets and self-healing.',
  'crates/fxcursor-daemon/src/gpu/shaders/physics.wgsl':
    'Prototype compute shader (spring chain + particles). NOT dispatched yet; physics runs on CPU.',
  'crates/fxcursor-daemon/src/gpu/shaders/render.wgsl':
    'Daemon copy of the ribbon/SDF render shader (identical to src-tauri shader.wgsl).',
  'crates/fxcursor-daemon/src/input/windows.rs':
    'Win32 GetCursorPos/GetAsyncKeyState poller (Raw Input not yet implemented).',
  'crates/fxcursor-daemon/src/state.rs': 'Daemon config container with self-healing load/save.',
  'crates/fxcursor-daemon/src/tray.rs': 'Daemon tray-icon/muda menu (Open Settings is a stub).',
  'src-tauri/src/webview_hardening.rs':
    'Webview security hardening disabling unauthorized actions in prod.',
  'src-tauri/src/overlay/mod.rs':
    'Multi-monitor virtual desktop bounds query and overlay window setup.',
  'src-tauri/src/overlay/renderer.rs':
    'Hardware-accelerated wgpu 4-layer trail, SDF head, and particle renderer.',
  'src-tauri/src/overlay/shader.wgsl':
    'DirectX 12 / Metal / Vulkan WGSL shaders with pre-multiplied alpha.',
  'crates/fxcursor-protocol/Cargo.toml':
    'Cargo manifest for shared FXCursor IPC protocol and presets.',
  'crates/fxcursor-protocol/src/config.rs':
    'Serializable Rust configuration structs for all visual modules.',
  'crates/fxcursor-protocol/src/presets.rs':
    'Built-in Rust preset definitions and schema unit tests.',
  'crates/fxcursor-render/Cargo.toml':
    'Cargo manifest for the shared wgpu renderer crate used by the Studio and the daemon.',
  'crates/fxcursor-render/src/lib.rs':
    'Shared renderer crate root: re-exports OverlayRenderer, ModeMask and vertex types.',
  'crates/fxcursor-render/src/renderer.rs':
    'Shared renderer: CPU spring-chain physics, 4-layer ribbon mesh, SDF instances, effect-mode mask.',
  'crates/fxcursor-render/src/shaders/render.wgsl':
    'Ribbon + SDF billboard vertex/fragment shader with fwidth anti-aliasing and pre-multiplied alpha.',
  'crates/fxcursor-render/src/shaders/physics.wgsl':
    'Prototype compute shader (spring chain + particles). NOT dispatched yet; physics runs on CPU.',
  'crates/fxcursor-daemon/Cargo.toml': 'Cargo manifest for standalone background daemon service.',
  'crates/fxcursor-daemon/src/main.rs':
    'Experimental standalone winit daemon (Windows-only prototype, not launched by the app).',
  'crates/fxcursor-daemon/src/ipc/mod.rs':
    'Named pipe and Unix domain socket IPC server implementation.',
  'crates/fxcursor-daemon/src/gpu/renderer.rs': 'Daemon-side wgpu hardware overlay renderer.',
};

function toPosix(filePath: string): string {
  return filePath.replace(/\\/g, '/');
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  const kb = bytes / 1024;
  if (kb < 1024) return `${kb.toFixed(1)} KB`;
  return `${(kb / 1024).toFixed(1)} MB`;
}

async function generateArchitectureMarkdown() {
  const rootDir = process.cwd();

  const fileConfig = await loadFileConfig(rootDir, null);
  const config = mergeConfigs(rootDir, fileConfig, {});

  config.output.files = false;
  config.output.git.sortByChanges = false;
  config.security.enableSecurityCheck = false;

  const search = await searchFiles(rootDir, config);

  const result: PackResult = await pack([rootDir], config, () => {}, {
    produceOutput: async () => ({ outputForMetrics: '' }),
  });

  const filePaths = [
    ...new Set([
      ...search.filePaths,
      ...result.safeFilePaths,
      ...result.skippedFiles.map((f) => f.path),
    ]),
  ].toSorted((a, b) => toPosix(a).localeCompare(toPosix(b)));

  const emptyDirs = config.output.includeEmptyDirectories ? search.emptyDirPaths : [];
  const tree = generateTreeString(filePaths, emptyDirs);

  const contentByPath = new Map(result.processedFiles.map((f) => [toPosix(f.path), f.content]));
  const tokenCounts = new Map(
    Object.entries(result.fileTokenCounts).map(([k, v]) => [toPosix(k), v])
  );
  const charCounts = new Map(
    Object.entries(result.fileCharCounts).map(([k, v]) => [toPosix(k), v])
  );

  const totalChars = [...charCounts.values()].reduce((sum, v) => sum + v, 0);
  const totalTokens = [...tokenCounts.values()].reduce((sum, v) => sum + v, 0);

  const fileRows: string[] = [];
  for (const filePath of filePaths) {
    const posix = toPosix(filePath);
    const absPath = path.join(rootDir, filePath);
    const size = fs.statSync(absPath, { throwIfNoEntry: false })?.size ?? 0;
    const content = contentByPath.get(posix);
    const lines = content !== undefined ? content.split('\n').length : '—';
    const tokens = tokenCounts.get(posix) ?? '—';
    const chars = charCounts.get(posix) ?? '—';
    const desc = fileDescriptions[posix] ?? 'Source or configuration file for the application.';
    fileRows.push(
      `| \`${posix}\` | ${formatBytes(size)} | ${lines} | ${tokens} | ${chars} | ${desc} |`
    );
  }

  const markdown = `# Project Architecture Overview

This document provides a single-file summary of the **FXCursor V4 Next-Gen** architecture. The directory tree and per-file metadata are generated by the [Repomix](https://repomix.com) \`pack()\` API (\`scripts/generate-arch.ts\`).

> [!NOTE]
> This file contains the complete directory tree and a metadata inventory (size, lines, tokens, characters) of every file in the project. Full code contents are omitted to keep the architecture map concise.

---

## 1. Directory Structure

\`\`\`
${tree}
\`\`\`

---

## 2. File Inventory & Descriptions

Repomix metrics: **${filePaths.length} files · ${formatBytes(totalChars)} · ${totalTokens.toLocaleString()} tokens** (text files; binary assets are listed without content metrics).

| File Path | Size | Lines | Tokens | Chars | Description |
| :--- | :--- | :--- | :--- | :--- | :--- |
${fileRows.join('\n')}

---

## 3. Technology Stack & Data Flow (as implemented)

- **Frontend Layer**: **SolidJS 2.0** + **TypeScript 7** dashboard (11 tabs) with a 100% AMOLED black theme and runtime accent tokens. Talks to Rust exclusively through Tauri \`invoke\` commands and the \`config-updated\` event.
- **Desktop Shell**: **Tauri 2** owns two windows: the Studio window and a transparent, click-through, always-on-top overlay window spanning the Win32 virtual desktop. A dedicated \`gpu-render-thread\` creates a **wgpu 30** surface on the overlay window (DX12/Vulkan on Windows, Metal on macOS, Vulkan on Linux).
- **Render Engine (CPU physics, GPU raster)**: spring-damper chain and Catmull-Rom resampling run on the CPU each frame; each ribbon layer is uploaded as one round capsule per sample pair and the union is resolved on the GPU with a depth-based max-coverage pre-pass (round joins and caps, no folding at hairpins, no double blending); head, ripples, particles and satellites are instanced SDF billboards. When nothing can change on screen the loop stops submitting GPU work (idle skip after 3 settle frames).
- **State & Persistence**: \`Arc<Mutex<AppConfig>>\` shared by IPC, tray, hotkey and renderer. Every change is broadcast, debounce-autosaved to \`config.json\` (OS app-config dir or portable \`Data/\`) and loaded back through the field-level self-healing deserializer.
- **OS Integrations**: system tray, global toggle hotkey (\`tauri-plugin-global-shortcut\`), login autostart (\`tauri-plugin-autostart\`), single-instance guard, minimize-to-tray / start-minimized behaviour.
- **Experimental Daemon**: \`crates/fxcursor-daemon\` is a Windows-only winit prototype of a future headless renderer with a JSON named-pipe/Unix-socket server. Nothing launches or connects to it yet, and \`physics.wgsl\` is not dispatched.
- **Tooling**: Bun scripts for typecheck/lint/test, a 7-gate pre-commit validator, dual-ecosystem dependency updater, this Repomix architecture map, and optional upstream docs mirrors.
`;

  fs.writeFileSync(path.join(rootDir, 'ARCHITECTURE.md'), markdown, 'utf8');
  console.log(`✅ ARCHITECTURE.md generated successfully (${filePaths.length} files indexed)`);
}

generateArchitectureMarkdown().catch((e) => {
  console.error('Error generating architecture map:', e);
  process.exit(1);
});
