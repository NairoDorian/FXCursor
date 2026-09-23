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
 * 4. Transitive Sub-Dependency Refresh within the pinned ranges (bun update & cargo update)
 * 5. Full Inventory Audit & Diff Tracking (Cargo.lock & node_modules)
 * 6. Vite Production Build Validation (bun run build)
 * 7. TypeScript Strict Typecheck (bun run typecheck)
 * 8. Cargo workspace check + unit tests, Bun tests
 *
 * Never downgrades: a registry answer that is not strictly newer than the current pin is
 * ignored. Any failing step aborts with a non-zero exit code.
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
      // `max_version` is the newest *stable*: only an upgrade if it beats the current pin
      // (returning it unconditionally rewrote e.g. `=2.0.0-rc.25` down to 1.x).
      return crate.max_version && compareVersions(crate.max_version, current) > 0
        ? crate.max_version
        : null;
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

/** Runs a pipeline step and aborts the whole update when it fails. */
function mustRun(label: string, cmd: string, args: string[]): void {
  console.log(label);
  if (!runCmd(cmd, args).success) {
    console.error(`❌ ${cmd} ${args.join(' ')} failed; aborting (files may already be modified).`);
    process.exit(1);
  }
}

/** True when `latest` is a real upgrade over the pinned `current` (never a downgrade). */
function isUpgrade(latest: string | null, current: string): latest is string {
  return latest !== null && compareVersions(latest, current) > 0;
}

/** Root + every workspace member manifest (`crates/*`, `src-tauri`). */
function workspaceManifests(): string[] {
  const found = [path.resolve('Cargo.toml'), path.resolve('src-tauri/Cargo.toml')];
  const cratesDir = path.resolve('crates');
  if (fs.existsSync(cratesDir)) {
    for (const entry of fs.readdirSync(cratesDir, { withFileTypes: true })) {
      if (entry.isDirectory()) found.push(path.join(cratesDir, entry.name, 'Cargo.toml'));
    }
  }
  return found.filter((p) => fs.existsSync(p));
}

function escapeRegExp(text: string): string {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

function printReport(statuses: DependencyStatus[], applied: boolean): void {
  console.log('\n=================================================================');
  console.log(`📊 DEPENDENCY STATUS REPORT (FXCURSOR V4)${applied ? '' : ' — DRY RUN, nothing changed'}`);
  console.log('=================================================================');
  for (const s of statuses.toSorted((a, b) => a.name.localeCompare(b.name))) {
    const statusText = s.needsUpdate
      ? `${applied ? '✨ Upgraded' : '⬆️ Would upgrade'}${s.prerelease ? ' (pre-release)' : ''}`
      : '⚡ Up-to-date';
    console.log(
      ` ${s.name.padEnd(30, ' ')} | ${s.ecosystem.padEnd(12, ' ')} | ${s.currentVersion.padEnd(9, ' ')} | ${s.latestVersion.padEnd(9, ' ')} | ${statusText}`
    );
  }
  console.log('=================================================================');
}

async function updateEverything() {
  console.log('=================================================================');
  console.log(
    `🚀 STARTING FXCURSOR V4 DEPENDENCY UPDATE${PRERELEASE_MODE ? ' (PRERELEASE / LATEST MODE)' : ''}`
  );
  console.log('=================================================================\n');

  const pkgPath = path.resolve('package.json');
  // Discovered, not listed: a hard-coded list once missed crates/fxcursor-render, leaving its
  // wgpu behind src-tauri's (two wgpu majors in one build).
  const cargoTomlPaths = workspaceManifests();

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
        const needs = isUpgrade(latest, currClean);
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
        const needs = isUpgrade(latest, currClean);
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

      // Every dependency table, including platform ones such as
      // [target.'cfg(not(windows))'.dependencies] or [target.'cfg(target_os = "macos")'.dependencies].
      if (/(^|\.)(dependencies|build-dependencies|dev-dependencies)$/.test(currentSection)) {
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
              const needs = isUpgrade(latest, currClean);
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
    printReport(allStatuses, false);
    process.exit(0);
  }

  const beforeCargoLock = parseCargoLock(cargoLockPath);
  const beforeBunLock = parseBunInstalledVersions();

  // 1. Upgrade NPM runtime deps
  const outdatedRuntime = allStatuses.filter((s) => s.type === 'runtime' && s.needsUpdate);
  if (outdatedRuntime.length > 0) {
    console.log(`📦 Upgrading ${outdatedRuntime.length} NPM Runtime Dependencies...`);
    const targets = outdatedRuntime.map((s) => `${s.name}@${s.latestVersion}`);
    mustRun('   bun add', 'bun', ['add', ...targets]);
  }

  // 2. Upgrade NPM dev deps
  const outdatedDev = allStatuses.filter((s) => s.type === 'dev' && s.needsUpdate);
  if (outdatedDev.length > 0) {
    console.log(`🛠️ Upgrading ${outdatedDev.length} NPM DevDependencies...`);
    const targets = outdatedDev.map((s) => `${s.name}@${s.latestVersion}`);
    mustRun('   bun add -d', 'bun', ['add', '-d', ...targets]);
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
          // Anchored to the start of the line (`specta` must not also rewrite `tauri-specta`)
          // and keeping the original operator (`=` exact pins stay exact).
          const name = escapeRegExp(crate.name);
          const regInline = new RegExp(
            `^(\\s*${name}\\s*=\\s*\\{[^}\\n]*version\\s*=\\s*"[\\^~=<>]*)([^"]+)(")`,
            'gm'
          );
          const regSimple = new RegExp(`^(\\s*${name}\\s*=\\s*"[\\^~=<>]*)([^"]+)(")`, 'gm');
          content = content
            .replace(regInline, `$1${crate.latestVersion}$3`)
            .replace(regSimple, `$1${crate.latestVersion}$3`);
        });
      fs.writeFileSync(tomlPath, content, 'utf8');
    });
  }

  // 4. Sub-dependencies refresh
  // `bun update` (no --latest) stays inside the ranges just written: `--latest` would move
  // every package to its `latest` dist-tag, i.e. back from a pre-release (solid-js 2 rc → 1.x).
  mustRun('🔒 Refreshing transitive sub-dependencies (bun update)...', 'bun', ['update']);
  mustRun('🔒 Refreshing Cargo.lock (cargo update)...', 'cargo', ['update']);

  // 5–8. Validation
  mustRun('📐 Checking TypeScript types...', 'bun', ['run', 'typecheck']);
  mustRun('⚡ Validating Vite production build...', 'bun', ['run', 'build']);
  mustRun('🦀 Validating Cargo workspace check...', 'cargo', ['check', '--workspace']);
  mustRun('🧪 Running Cargo workspace tests...', 'cargo', ['test', '--workspace']);
  mustRun('🧪 Running Bun tests...', 'bun', ['test']);

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

  printReport(allStatuses, true);
  console.log(`🎉 Transitive sub-dependencies upgraded: ${subDepChanges.length}`);
  console.log('✅ All dependencies upgraded to latest / pre-release & verified clean!\n');
}

updateEverything().catch((err) => {
  console.error('❌ update-deps crashed:', err);
  process.exit(1);
});
