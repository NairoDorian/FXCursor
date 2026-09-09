/**
 * Builds the portable distribution: a zip containing the release executable, an empty `portable`
 * marker (so config/logs stay in `Data/` next to the exe) and a short README.
 *
 * Usage:
 *   bun run package:portable            # expects `bun run tauri build` to have run already
 *   bun run package:portable --build    # runs the release build first
 *
 * Output: target/release/fxcursor-portable-<version>-<os>-<arch>.zip
 */
import { existsSync, mkdirSync, rmSync, writeFileSync, copyFileSync, statSync } from 'node:fs';
import { resolve, join } from 'node:path';
import { APP_VERSION } from './version';

const rootDir = resolve(import.meta.dir, '..');
const releaseDir = join(rootDir, 'target', 'release');
const exeName = process.platform === 'win32' ? 'fxcursor.exe' : 'fxcursor';
const exePath = join(releaseDir, exeName);

const os = process.platform === 'win32' ? 'win' : process.platform === 'darwin' ? 'mac' : 'linux';
const arch = process.arch === 'x64' ? 'x64' : process.arch;
const zipName = `fxcursor-portable-${APP_VERSION}-${os}-${arch}.zip`;
const zipPath = join(releaseDir, zipName);
const stageDir = join(releaseDir, 'portable-stage', `FXCursor-Studio-${APP_VERSION}`);

async function run(cmd: string[], cwd = rootDir): Promise<void> {
  const proc = Bun.spawn(cmd, { cwd, stdout: 'inherit', stderr: 'inherit' });
  const code = await proc.exited;
  if (code !== 0) {
    throw new Error(`${cmd.join(' ')} exited with code ${code}`);
  }
}

async function main() {
  if (process.argv.includes('--build')) {
    console.log('🔨 Running release build first (bun run tauri build)...');
    await run(['bun', 'run', 'tauri', 'build']);
  }

  if (!existsSync(exePath)) {
    console.error(`❌ ${exePath} not found. Run \`bun run tauri build\` (or pass --build).`);
    process.exit(1);
  }

  rmSync(join(releaseDir, 'portable-stage'), { recursive: true, force: true });
  mkdirSync(stageDir, { recursive: true });

  copyFileSync(exePath, join(stageDir, exeName));
  // Empty marker file: the app redirects config/logs to ./Data when it sees this next to the exe.
  writeFileSync(join(stageDir, 'portable'), '');
  mkdirSync(join(stageDir, 'Data'), { recursive: true });
  writeFileSync(
    join(stageDir, 'README.txt'),
    [
      `FXCursor Studio ${APP_VERSION} — portable edition`,
      '',
      'Run fxcursor(.exe). Because the empty file named "portable" sits next to the',
      'executable, all settings and logs are kept in the Data/ folder here instead of your',
      'user profile, so this folder can live on a USB stick.',
      '',
      'First launch: the Studio window opens and a tray icon appears. Left-click the tray icon',
      'to show/hide the Studio; right-click for Toggle Effects / Quit. Default hotkey to toggle',
      'effects: Ctrl+Shift+E (change it in Hotkeys & Tray).',
      '',
      'Windows only for now. Requires a GPU with Direct3D 12 or Vulkan support.',
      '',
      'Source & issues: https://github.com/NairoDorian/FXCursor',
    ].join('\r\n')
  );

  rmSync(zipPath, { force: true });
  const stageParent = join(releaseDir, 'portable-stage');
  if (process.platform === 'win32') {
    await run(
      [
        'powershell',
        '-NoProfile',
        '-Command',
        `Compress-Archive -Path '${join(stageParent, `FXCursor-Studio-${APP_VERSION}`)}' -DestinationPath '${zipPath}' -CompressionLevel Optimal`,
      ],
      rootDir
    );
  } else {
    await run(['zip', '-r', '-q', zipPath, `FXCursor-Studio-${APP_VERSION}`], stageParent);
  }

  const sizeMb = (statSync(zipPath).size / (1024 * 1024)).toFixed(2);
  console.log(`✅ Portable package ready: ${zipPath} (${sizeMb} MB)`);
}

main().catch((e) => {
  console.error('❌ package-portable failed:', e);
  process.exit(1);
});
