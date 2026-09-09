import { existsSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const rootDir = resolve(import.meta.dir, '..');
const docsDir = resolve(rootDir, '.docs');

interface DocMirror {
  name: string;
  repo: string;
  branch: string;
  targetDir: string;
  description: string;
}

const mirrors: DocMirror[] = [
  {
    name: 'Tauri 2 Docs',
    repo: 'https://github.com/tauri-apps/tauri-docs.git',
    branch: 'v2',
    targetDir: resolve(docsDir, 'tauri-docs'),
    description: 'Tauri 2 core, capabilities, IPC protocols, and plugins',
  },
  {
    name: 'SolidJS 2 Docs',
    repo: 'https://github.com/solidjs/solid-docs.git',
    branch: 'v2-rebuild',
    targetDir: resolve(docsDir, 'solid-docs'),
    description: 'SolidJS 2.0 reactivity, signals, and createEffect APIs',
  },
  {
    name: 'Bun Docs',
    repo: 'https://github.com/oven-sh/bun.git',
    branch: 'main',
    targetDir: resolve(docsDir, 'bun-docs'),
    description: 'Bun runtime, package manager, and test runner docs',
  },
  {
    name: 'TypeScript Website Docs',
    repo: 'https://github.com/microsoft/TypeScript-Website.git',
    branch: 'v2',
    targetDir: resolve(docsDir, 'typescript-website'),
    description: 'TypeScript handbook, compiler options, and type system',
  },
  {
    name: 'wgpu Docs',
    repo: 'https://github.com/gfx-rs/wgpu.git',
    branch: 'trunk',
    targetDir: resolve(docsDir, 'wgpu-docs'),
    description: 'WebGPU / DirectX 12 / Metal / Vulkan graphics API docs and examples',
  },
];

async function runGit(args: string[], cwd: string): Promise<boolean> {
  const proc = Bun.spawn(['git', ...args], {
    cwd,
    stdout: 'inherit',
    stderr: 'inherit',
  });
  const code = await proc.exited;
  return code === 0;
}

async function syncDocs() {
  if (!existsSync(docsDir)) {
    mkdirSync(docsDir, { recursive: true });
  }

  console.log('📚 Synchronizing Upstream Documentation Mirrors (.docs/)...\n');

  for (const mirror of mirrors) {
    console.log(`📌 Mirror: ${mirror.name} (${mirror.branch}) — ${mirror.description}`);
    if (existsSync(mirror.targetDir)) {
      console.log(`  🔄 Updating existing mirror...`);
      await runGit(['fetch', 'origin', mirror.branch, '--depth=1'], mirror.targetDir);
      await runGit(['reset', '--hard', `origin/${mirror.branch}`], mirror.targetDir);
    } else {
      console.log(`  📥 Cloning shallow mirror...`);
      await runGit(
        ['clone', '--depth=1', '--branch', mirror.branch, mirror.repo, mirror.targetDir],
        docsDir
      );
    }
  }

  console.log('\n✅ All upstream documentation mirrors synced successfully!');
}

async function checkDocs() {
  console.log('🔍 Checking Documentation Mirror Status...\n');
  for (const mirror of mirrors) {
    const exists = existsSync(mirror.targetDir);
    console.log(
      `  ${exists ? '✅' : '❌'} ${mirror.name}: ${exists ? 'Present on disk' : 'Missing (run bun run docs:sync)'}`
    );
  }
}

async function findInDocs(query: string) {
  if (!query) {
    console.error('❌ Please specify search query: bun run docs:find <query>');
    process.exit(1);
  }
  console.log(`🔎 Searching documentation mirrors for "${query}"...\n`);
  const proc = Bun.spawn(['rg', '-i', '-n', query, docsDir], {
    stdout: 'inherit',
    stderr: 'inherit',
  });
  await proc.exited;
}

const args = process.argv.slice(2);
if (args.includes('--check')) {
  checkDocs();
} else if (args.includes('--find')) {
  const q = args[args.indexOf('--find') + 1];
  findInDocs(q);
} else {
  syncDocs();
}
