import { existsSync, mkdirSync } from 'node:fs';
import { resolve } from 'node:path';

const rootDir = resolve(import.meta.dir, '..');
const iconsDir = resolve(rootDir, 'src-tauri/icons');

console.log('🎨 Verifying Cross-Platform Application Icons in', iconsDir);

if (!existsSync(iconsDir)) {
  mkdirSync(iconsDir, { recursive: true });
}

const requiredIcons = [
  '32x32.png',
  '128x128.png',
  '128x128@2x.png',
  'icon.png',
  'icon.ico',
  'icon.icns',
];

for (const icon of requiredIcons) {
  const iconPath = resolve(iconsDir, icon);
  const exists = existsSync(iconPath);
  console.log(`  ${exists ? '✅' : '⚠️'} ${icon}: ${exists ? 'Present' : 'Missing'}`);
}
