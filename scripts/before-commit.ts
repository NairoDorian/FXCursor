import { readFileSync, writeFileSync } from 'node:fs';
import { resolve } from 'node:path';
import { APP_VERSION } from './version';

const rootDir = resolve(import.meta.dir, '..');

interface MirrorTarget {
  name: string;
  path: string;
  readVersion: (content: string) => string | null;
  writeVersion: (content: string, version: string) => string;
}

const targets: MirrorTarget[] = [
  {
    name: 'package.json',
    path: resolve(rootDir, 'package.json'),
    readVersion: (content) => {
      const parsed = JSON.parse(content);
      return parsed.version ?? null;
    },
    writeVersion: (content, version) => {
      const parsed = JSON.parse(content);
      parsed.version = version;
      return JSON.stringify(parsed, null, 2) + '\n';
    },
  },
  {
    name: 'Cargo.toml (Workspace)',
    path: resolve(rootDir, 'Cargo.toml'),
    readVersion: (content) => {
      const match = content.match(/^version\s*=\s*"([^"]+)"/m);
      return match ? match[1] : null;
    },
    writeVersion: (content, version) => {
      return content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
    },
  },
  {
    name: 'src-tauri/Cargo.toml',
    path: resolve(rootDir, 'src-tauri/Cargo.toml'),
    readVersion: (content) => {
      const match = content.match(/^version\s*=\s*"([^"]+)"/m);
      return match ? match[1] : null;
    },
    writeVersion: (content, version) => {
      return content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
    },
  },
  {
    name: 'crates/fxcursor-protocol/Cargo.toml',
    path: resolve(rootDir, 'crates/fxcursor-protocol/Cargo.toml'),
    readVersion: (content) => {
      const match = content.match(/^version\s*=\s*"([^"]+)"/m);
      return match ? match[1] : null;
    },
    writeVersion: (content, version) => {
      return content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
    },
  },
  {
    name: 'crates/fxcursor-daemon/Cargo.toml',
    path: resolve(rootDir, 'crates/fxcursor-daemon/Cargo.toml'),
    readVersion: (content) => {
      const match = content.match(/^version\s*=\s*"([^"]+)"/m);
      return match ? match[1] : null;
    },
    writeVersion: (content, version) => {
      return content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
    },
  },
  {
    name: 'crates/fxcursor-render/Cargo.toml',
    path: resolve(rootDir, 'crates/fxcursor-render/Cargo.toml'),
    readVersion: (content) => {
      const match = content.match(/^version\s*=\s*"([^"]+)"/m);
      return match ? match[1] : null;
    },
    writeVersion: (content, version) => {
      return content.replace(/^version\s*=\s*"[^"]+"/m, `version = "${version}"`);
    },
  },
  {
    name: 'src-tauri/tauri.conf.json',
    path: resolve(rootDir, 'src-tauri/tauri.conf.json'),
    readVersion: (content) => {
      const parsed = JSON.parse(content);
      return parsed.version ?? null;
    },
    writeVersion: (content, version) => {
      const parsed = JSON.parse(content);
      parsed.version = version;
      return JSON.stringify(parsed, null, 2) + '\n';
    },
  },
];

function bumpVersion(current: string, type: 'patch' | 'minor' | 'major'): string {
  const parts = current.split('.').map(Number);
  if (parts.length !== 3 || parts.some(isNaN)) {
    throw new Error(`Invalid SemVer version: ${current}`);
  }
  if (type === 'major') {
    return `${parts[0] + 1}.0.0`;
  } else if (type === 'minor') {
    return `${parts[0]}.${parts[1] + 1}.0`;
  } else {
    return `${parts[0]}.${parts[1]}.${parts[2] + 1}`;
  }
}

async function runCommand(cmd: string[], label: string): Promise<boolean> {
  const start = performance.now();
  console.log(`\n⏳ [Gate] ${label}...`);
  const proc = Bun.spawn(cmd, {
    cwd: rootDir,
    stdout: 'inherit',
    stderr: 'inherit',
  });
  const exitCode = await proc.exited;
  const elapsed = ((performance.now() - start) / 1000).toFixed(2);
  if (exitCode === 0) {
    console.log(`✅ [Pass] ${label} (${elapsed}s)`);
    return true;
  } else {
    console.error(`❌ [Fail] ${label} (exit code ${exitCode})`);
    return false;
  }
}

async function main() {
  const args = process.argv.slice(2);
  const isCheck = args.includes('--check');
  const isFull = args.includes('--full');
  const bumpIdx = args.indexOf('--bump');

  let targetVersion = APP_VERSION;

  if (bumpIdx !== -1) {
    const bumpType = args[bumpIdx + 1] as 'patch' | 'minor' | 'major';
    if (!['patch', 'minor', 'major'].includes(bumpType)) {
      console.error('❌ Specify --bump patch, --bump minor, or --bump major');
      process.exit(1);
    }
    targetVersion = bumpVersion(APP_VERSION, bumpType);
    console.log(`🚀 Bumping version: ${APP_VERSION} → ${targetVersion}`);
    const versionFile = resolve(rootDir, 'scripts/version.ts');
    writeFileSync(
      versionFile,
      `/**\n * Single source of truth for the application version.\n * Run \`bun run before-commit\` to sync this version across manifests.\n */\nexport const APP_VERSION = '${targetVersion}';\n`
    );
  }

  console.log(`\n📌 Target APP_VERSION: ${targetVersion}`);
  let hasDrift = false;

  for (const target of targets) {
    try {
      const content = readFileSync(target.path, 'utf8');
      const current = target.readVersion(content);
      const inheritsWorkspace = current === null && /^version\.workspace\s*=\s*true/m.test(content);
      if (current === targetVersion) {
        console.log(`  ✅ ${target.name}: in sync (${current})`);
      } else if (inheritsWorkspace) {
        console.log(`  ✅ ${target.name}: inherits workspace version (${targetVersion})`);
      } else {
        if (isCheck) {
          console.error(
            `  ❌ ${target.name}: DRIFT detected (found ${current}, expected ${targetVersion})`
          );
          hasDrift = true;
        } else {
          const updated = target.writeVersion(content, targetVersion);
          writeFileSync(target.path, updated, 'utf8');
          console.log(`  🔧 ${target.name}: updated (${current} → ${targetVersion})`);
        }
      }
    } catch (e) {
      console.warn(`  ⚠️ Could not read/write ${target.name}: ${e}`);
    }
  }

  if (isCheck && hasDrift) {
    console.error('\n❌ Version drift detected! Run `bun run before-commit` to fix.');
    process.exit(1);
  }

  if (isFull) {
    console.log('\n🚀 Executing 7-Gate Pre-Commit Validation Suite...\n');
    const gates: Array<{ cmd: string[]; label: string }> = [
      { cmd: ['bun', 'run', 'typecheck'], label: '1. TypeScript 7 Typecheck (tsc -b)' },
      { cmd: ['bun', 'run', 'lint'], label: '2. Oxlint Static Code Analysis' },
      { cmd: ['bun', 'test'], label: '3. Bun Unit Tests' },
      { cmd: ['bun', 'run', 'build'], label: '4. Production Vite Bundle Build' },
      { cmd: ['cargo', 'check', '--workspace'], label: '5. Cargo Workspace Compilation Check' },
      { cmd: ['cargo', 'test', '--workspace'], label: '6. Cargo Workspace Unit Tests' },
      { cmd: ['bun', 'run', 'arch'], label: '7. Architecture Map Generation (Repomix)' },
    ];

    for (const gate of gates) {
      const ok = await runCommand(gate.cmd, gate.label);
      if (!ok) {
        console.error(`\n❌ Validation suite failed at step: ${gate.label}`);
        process.exit(1);
      }
    }

    console.log('\n🎉 ALL 7 VALIDATION GATES PASSED CLEANLY! Ready for commit/release.');
  }
}

main().catch((e) => {
  console.error('Fatal error in before-commit:', e);
  process.exit(1);
});
