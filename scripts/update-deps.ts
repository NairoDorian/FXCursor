import fs from 'node:fs';
import { spawnSync } from 'node:child_process';
import path from 'node:path';

/**
 * Ultimate All-Inclusive Dependency Updater for FXCursor V4.
 *
 * Capabilities:
 * 1. Dynamic NPM Registry Querying (prerelease dist-tags & @latest)
 * 2. Dynamic Crates.io Registry Querying (newest_version prereleases & max_version)
 * 3. Direct Workspace Dependency Upgrading (package.json & all workspace Cargo.toml files)
 * 4. Transitive Sub-Dependency Upgrading (bun update --latest & cargo update)
 * 5. Full Inventory Audit & Diff Tracking (Cargo.lock & node_modules)
 * 6. Vite Production Build Validation (bun run build)
 * 7. TypeScript Strict Typecheck (bun run typecheck)
 * 8. Native Cargo Workspace Backend Compilation Verification (cargo check --workspace)
 */

const cliArgs = new Set(process.argv.slice(2));
const PRERELEASE_MODE = cliArgs.has('--prerelease') || !cliArgs.has('--stable'); // Default to prerelease mode as requested
const DRY_RUN = cliArgs.has('--dry-run');
const SHOW_HELP = cliArgs.has('--help') || cliArgs.has('-h');

const PRERELEASE_TAGS: readonly string[] = [
  'next',
  'beta',
  'rc',
  'alpha',
  'canary',
  'experimental',
  'insiders',
  'dev',
];

if (SHOW_HELP) {
  console.log(`Usage:
  bun run update-deps                          Prerelease / latest pipeline (default)
  bun run update-deps --stable                 Stable @latest pipeline
  bun run update-deps --dry-run                Report what WOULD upgrade (no changes made)`);
  process.exit(0);
}

interface DependencyStatus {
  name: string;
  ecosystem: 'NPM (Bun)' | 'Cargo (Rust)';
  type: 'runtime' | 'dev' | 'cargo-dep' | 'cargo-build';
  filePath?: string;
  currentVersion: string;
  latestVersion: string;
  needsUpdate: boolean;
  prerelease: boolean;
}

interface SubDepDiff {
  name: string;
  ecosystem: 'NPM (Bun)' | 'Cargo (Rust)';
  before: string;
  after: string;
}

function isPrereleaseVersion(version: string): boolean {
  return /-\d|-[a-z]/i.test(version);
}

function parseVersion(v: string): { core: number[]; pre: string | null } {
  const [coreStr, pre] = v.split('-', 2);
  const core = (coreStr ?? '0').split('.').map((n) => parseInt(n, 10) || 0);
  while (core.length < 3) core.push(0);
  return { core, pre: pre ?? null };
}

function compareVersions(a: string, b: string): number {
  const A = parseVersion(a);
  const B = parseVersion(b);
  for (let i = 0; i < 3; i++) {
    const aCore = A.core[i] ?? 0;
    const bCore = B.core[i] ?? 0;
    if (aCore !== bCore) return aCore > bCore ? 1 : -1;
  }
  if (A.pre === null && B.pre === null) return 0;
  if (A.pre === null) return 1;
  if (B.pre === null) return -1;
  const aId = A.pre.split('.');
  const bId = B.pre.split('.');
  const len = Math.max(aId.length, bId.length);
  for (let i = 0; i < len; i++) {
    const x = aId[i];
    const y = bId[i];
    if (x === undefined) return -1;
    if (y === undefined) return 1;
    const xn = /^\d+$/.test(x) ? Number(x) : null;
    const yn = /^\d+$/.test(y) ? Number(y) : null;
    if (xn !== null && yn !== null) {
      if (xn !== yn) return xn > yn ? 1 : -1;
    } else if (xn !== null) {
      return -1;
    } else if (yn !== null) {
      return 1;
    } else if (x !== y) {
      return x > y ? 1 : -1;
    }
  }
  return 0;
}

function compareCores(a: string, b: string): number {
  const A = (a.split('-')[0] ?? '0').split('.').map((n) => parseInt(n, 10) || 0);
  const B = (b.split('-')[0] ?? '0').split('.').map((n) => parseInt(n, 10) || 0);
  for (let i = 0; i < 3; i++) {
    const aCore = A[i] ?? 0;
    const bCore = B[i] ?? 0;
    if (aCore !== bCore) return aCore > bCore ? 1 : -1;
  }
  return 0;
}

function cleanVersion(v: string): string {
  if (typeof v !== 'string') return '';
  return v.replace(/^[\^~=v]/, '').trim();
}

async function fetchLatestNpmVersion(
  pkgName: string,
  current: string,
  prerelease = true
): Promise<string | null> {
  try {
    if (!prerelease) {
      const response = await fetch(`https://registry.npmjs.org/${pkgName}/latest`, {
        headers: { Accept: 'application/json' },
      });
      if (response.ok) {
        const data = (await response.json()) as { version?: string };
        return data.version && compareVersions(data.version, current) > 0 ? data.version : null;
      }
      return null;
    }
    const response = await fetch(`https://registry.npmjs.org/${pkgName}`, {
      headers: { Accept: 'application/json' },
    });
    if (!response.ok) return null;
    const data = (await response.json()) as {
      'dist-tags'?: Record<string, string>;
      time?: Record<string, string>;
    };
    const tags = data['dist-tags'] ?? {};
    const times = data.time ?? {};
    const currentTime = times[current] ?? null;

    const isStrictlyNewer = (v: string, publishedAt: string | null): boolean => {
      const coreDiff = compareCores(v, current);
      if (coreDiff !== 0) return coreDiff > 0;
      if (publishedAt !== null && currentTime !== null && publishedAt !== currentTime) {
        return new Date(publishedAt).getTime() > new Date(currentTime).getTime();
      }
      return compareVersions(v, current) > 0;
    };

    let best: { version: string; publishedAt: string | null } | null = null;
    for (const tag of PRERELEASE_TAGS) {
      const candidate = tags[tag];
      const publishedAt = candidate ? (times[candidate] ?? null) : null;
      if (!candidate || !isStrictlyNewer(candidate, publishedAt)) continue;
      if (best === null) {
        best = { version: candidate, publishedAt };
        continue;
      }
      const coreDiff = compareCores(candidate, best.version);
      let takeCandidate: boolean;
      if (coreDiff !== 0) {
        takeCandidate = coreDiff > 0;
      } else if (
        publishedAt !== null &&
        best.publishedAt !== null &&
        publishedAt !== best.publishedAt
      ) {
        takeCandidate = new Date(publishedAt).getTime() > new Date(best.publishedAt).getTime();
      } else {
        takeCandidate = compareVersions(candidate, best.version) > 0;
      }
      if (takeCandidate) best = { version: candidate, publishedAt };
    }

    const latestTag = tags['latest'] ?? null;
    return (
      best?.version ??
      (latestTag && isStrictlyNewer(latestTag, times[latestTag] ?? null) ? latestTag : null)
    );
  } catch {
    return null;
  }
}

async function fetchLatestCrateVersion(
  crateName: string,
  current: string,
  prerelease = true
): Promise<string | null> {
  if (crateName === 'winit') {
    return null; // winit 0.31-beta changed Window into dyn Window trait
  }
  try {
    const response = await fetch(`https://crates.io/api/v1/crates/${crateName}`, {
      headers: { 'User-Agent': 'FXCursorUpdater/4.0' },
    });
    if (response.ok) {
      const data = (await response.json()) as {
        crate?: { max_version?: string; newest_version?: string };
      };
      const crate = data.crate;
      if (!crate) return null;
      if (
        prerelease &&
        crate.newest_version &&
        compareVersions(crate.newest_version, current) > 0
      ) {
        return crate.newest_version;
      }
      return crate.max_version || null;
    }
  } catch {}
  return null;
}

function parseCargoLock(filePath: string): Record<string, string> {
  const map: Record<string, string> = {};
  if (!fs.existsSync(filePath)) return map;
  const content = fs.readFileSync(filePath, 'utf8');
  const blocks = content.split('[[package]]');
  for (const block of blocks) {
    const nameMatch = block.match(/^\s*name\s*=\s*"([^"]+)"/m);
    const verMatch = block.match(/^\s*version\s*=\s*"([^"]+)"/m);
    if (nameMatch && verMatch && nameMatch[1] !== undefined && verMatch[1] !== undefined) {
      map[nameMatch[1]] = verMatch[1];
    }
  }
  return map;
}

function parseBunInstalledVersions(): Record<string, string> {
  const map: Record<string, string> = {};
  const nmPath = path.resolve('node_modules');
  if (!fs.existsSync(nmPath)) return map;

  function scan(dir: string) {
    try {
      const entries = fs.readdirSync(dir, { withFileTypes: true });
      entries.forEach((e) => {
        if (e.isDirectory()) {
          if (e.name.startsWith('@')) {
            scan(path.join(dir, e.name));
          } else {
            const pkgJsonPath = path.join(dir, e.name, 'package.json');
            if (fs.existsSync(pkgJsonPath)) {
              try {
                const pj = JSON.parse(fs.readFileSync(pkgJsonPath, 'utf8'));
                if (pj.name && pj.version) {
                  map[pj.name] = pj.version;
                }
              } catch {}
            }
          }
        }
      });
    } catch {}
  }
  scan(nmPath);
  return map;
}

function runCmd(
  cmd: string,
  args: string[],
  cwd?: string
): { success: boolean; durationMs: number } {
  const start = Date.now();
  const res = spawnSync(cmd, args, { stdio: 'inherit', shell: true, cwd });
  const durationMs = Date.now() - start;
  return { success: res.status === 0, durationMs };
}

async function updateEverything() {
  console.log('=================================================================');
  console.log(
    `🚀 STARTING FXCURSOR V4 DEPENDENCY UPDATE${PRERELEASE_MODE ? ' (PRERELEASE / LATEST MODE)' : ''}`
  );
  console.log('=================================================================\n');

  const pkgPath = path.resolve('package.json');
  const cargoTomlPaths = [
    path.resolve('Cargo.toml'),
    path.resolve('src-tauri/Cargo.toml'),
    path.resolve('crates/fxcursor-daemon/Cargo.toml'),
    path.resolve('crates/fxcursor-protocol/Cargo.toml'),
  ].filter((p) => fs.existsSync(p));

  const cargoLockPath = path.resolve('Cargo.lock');

  console.log('🔍 Querying Registries (NPM & Crates.io) for Prerelease & Target Versions...');
  const queryStart = Date.now();

  const pkgJson = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  const runtimeDeps = pkgJson.dependencies || {};
  const devDeps = pkgJson.devDependencies || {};

  const allStatuses: DependencyStatus[] = [];
  const fetchPromises: Promise<void>[] = [];

  // 1. Query NPM runtime dependencies
  Object.entries(runtimeDeps).forEach(([name, ver]) => {
    fetchPromises.push(
      (async () => {
        const currClean = cleanVersion(ver as string);
        const latest = await fetchLatestNpmVersion(name, currClean, PRERELEASE_MODE);
        const needs = latest ? currClean !== latest : false;
        allStatuses.push({
          name,
          ecosystem: 'NPM (Bun)',
          type: 'runtime',
          currentVersion: ver as string,
          latestVersion: latest || currClean,
          needsUpdate: needs,
          prerelease: needs && latest !== null && isPrereleaseVersion(latest),
        });
      })()
    );
  });

  // 2. Query NPM devDependencies
  Object.entries(devDeps).forEach(([name, ver]) => {
    fetchPromises.push(
      (async () => {
        const currClean = cleanVersion(ver as string);
        const latest = await fetchLatestNpmVersion(name, currClean, PRERELEASE_MODE);
        const needs = latest ? currClean !== latest : false;
        allStatuses.push({
          name,
          ecosystem: 'NPM (Bun)',
          type: 'dev',
          currentVersion: ver as string,
          latestVersion: latest || currClean,
          needsUpdate: needs,
          prerelease: needs && latest !== null && isPrereleaseVersion(latest),
        });
      })()
    );
  });

  // 3. Query all Cargo.toml files in workspace
  cargoTomlPaths.forEach((tomlPath) => {
    const cargoContent = fs.readFileSync(tomlPath, 'utf8');
    const lines = cargoContent.split(/\r?\n/);
    let currentSection = '';

    lines.forEach((line: string) => {
      const trimmed = line.trim();
      if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
        currentSection = trimmed.slice(1, -1).trim();
        return;
      }

      if (
        [
          'dependencies',
          'build-dependencies',
          'dev-dependencies',
          'workspace.dependencies',
          "target.'cfg(windows)'.dependencies",
        ].includes(currentSection)
      ) {
        if (trimmed.includes('path =') || trimmed.includes('workspace = true')) {
          return; // skip local path/workspace deps
        }
        const matchInline = trimmed.match(/^([a-zA-Z0-9_-]+)\s*=\s*\{[^}]*version\s*=\s*"([^"]+)"/);
        const matchSimple = trimmed.match(/^([a-zA-Z0-9_-]+)\s*=\s*"([^"]+)"/);
        const match = matchInline || matchSimple;
        if (match && match[1] !== undefined && match[2] !== undefined) {
          const name = match[1];
          const ver = match[2];
          fetchPromises.push(
            (async () => {
              const currClean = cleanVersion(ver);
              const latest = await fetchLatestCrateVersion(name, currClean, PRERELEASE_MODE);
              const needs = latest ? currClean !== latest : false;
              allStatuses.push({
                name,
                ecosystem: 'Cargo (Rust)',
                type: currentSection.includes('build') ? 'cargo-build' : 'cargo-dep',
                filePath: tomlPath,
                currentVersion: ver,
                latestVersion: latest || currClean,
                needsUpdate: needs,
                prerelease: needs && latest !== null && isPrereleaseVersion(latest),
              });
            })()
          );
        }
      }
    });
  });

  await Promise.all(fetchPromises);
  const queryDuration = Date.now() - queryStart;
  console.log(`✅ Registry query complete (${queryDuration}ms)\n`);

  if (DRY_RUN) {
    console.log('DRY RUN COMPLETE.');
    process.exit(0);
  }

  const beforeCargoLock = parseCargoLock(cargoLockPath);
  const beforeBunLock = parseBunInstalledVersions();

  // 1. Upgrade NPM runtime deps
  const outdatedRuntime = allStatuses.filter((s) => s.type === 'runtime' && s.needsUpdate);
  if (outdatedRuntime.length > 0) {
    console.log(`📦 Upgrading ${outdatedRuntime.length} NPM Runtime Dependencies...`);
    const targets = outdatedRuntime.map((s) => `${s.name}@${s.latestVersion}`);
    runCmd('bun', ['add', ...targets]);
  }

  // 2. Upgrade NPM dev deps
  const outdatedDev = allStatuses.filter((s) => s.type === 'dev' && s.needsUpdate);
  if (outdatedDev.length > 0) {
    console.log(`🛠️ Upgrading ${outdatedDev.length} NPM DevDependencies...`);
    const targets = outdatedDev.map((s) => `${s.name}@${s.latestVersion}`);
    runCmd('bun', ['add', '-d', ...targets]);
  }

  // 3. Upgrade Cargo.toml files
  const outdatedCargo = allStatuses.filter((s) => s.ecosystem === 'Cargo (Rust)' && s.needsUpdate);
  if (outdatedCargo.length > 0) {
    console.log(`🦀 Updating ${outdatedCargo.length} Cargo.toml entries...`);
    cargoTomlPaths.forEach((tomlPath) => {
      let content = fs.readFileSync(tomlPath, 'utf8');
      outdatedCargo
        .filter((c) => c.filePath === tomlPath)
        .forEach((crate) => {
          const regInline = new RegExp(
            `(${crate.name}\\s*=\\s*\\{[^}]*version\\s*=\\s*")([^"]+)(")`,
            'g'
          );
          const regSimple = new RegExp(`(${crate.name}\\s*=\\s*")([^"]+)(")`, 'g');
          content = content
            .replace(regInline, `$1^${crate.latestVersion}$3`)
            .replace(regSimple, `$1^${crate.latestVersion}$3`);
        });
      fs.writeFileSync(tomlPath, content, 'utf8');
    });
  }

  // 4. Sub-dependencies refresh
  console.log('🔒 Refreshing all transitive sub-dependencies (bun update & cargo update)...');
  runCmd('bun', ['update', '--latest']);
  runCmd('cargo', ['update']);

  // 5. TypeScript validation
  console.log('📐 Checking TypeScript types...');
  const tscRes = runCmd('bun', ['run', 'typecheck']);
  if (!tscRes.success) {
    console.error('❌ TypeScript check failed!');
    process.exit(1);
  }

  // 6. Vite production build
  console.log('⚡ Validating Vite production build...');
  const buildRes = runCmd('bun', ['run', 'build']);
  if (!buildRes.success) {
    console.error('❌ Vite build failed!');
    process.exit(1);
  }

  // 7. Cargo workspace compilation check
  console.log('🦀 Validating Cargo workspace check...');
  const checkRes = runCmd('cargo', ['check', '--workspace']);
  if (!checkRes.success) {
    console.error('❌ Cargo workspace check failed!');
    process.exit(1);
  }

  const afterCargoLock = parseCargoLock(cargoLockPath);
  const afterBunLock = parseBunInstalledVersions();

  const subDepChanges: SubDepDiff[] = [];
  Object.keys(afterBunLock).forEach((name) => {
    const beforeVer = beforeBunLock[name];
    const afterVer = afterBunLock[name];
    if (beforeVer && afterVer && beforeVer !== afterVer) {
      subDepChanges.push({ name, ecosystem: 'NPM (Bun)', before: beforeVer, after: afterVer });
    }
  });
  Object.keys(afterCargoLock).forEach((name) => {
    const beforeVer = beforeCargoLock[name];
    const afterVer = afterCargoLock[name];
    if (beforeVer && afterVer && beforeVer !== afterVer) {
      subDepChanges.push({ name, ecosystem: 'Cargo (Rust)', before: beforeVer, after: afterVer });
    }
  });

  console.log('\n=================================================================');
  console.log('📊 DEPENDENCY STATUS REPORT (FXCURSOR V4)');
  console.log('=================================================================');
  allStatuses.forEach((s) => {
    const namePadded = s.name.padEnd(30, ' ');
    const ecoPadded = s.ecosystem.padEnd(12, ' ');
    const currPadded = s.currentVersion.padEnd(9, ' ');
    const latPadded = s.latestVersion.padEnd(9, ' ');
    const statusText = s.needsUpdate
      ? s.prerelease
        ? '⚠️ Pre-release'
        : '✨ Upgraded'
      : '⚡ Up-to-date';
    console.log(` ${namePadded} | ${ecoPadded} | ${currPadded} | ${latPadded} | ${statusText}`);
  });
  console.log('=================================================================');
  console.log(`🎉 Transitive sub-dependencies upgraded: ${subDepChanges.length}`);
  console.log('✅ All dependencies upgraded to latest / pre-release & verified clean!\n');
}

updateEverything();
