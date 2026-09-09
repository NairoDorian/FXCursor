import { defineConfig } from 'vite';
import solid from '@solidjs/vite-plugin';

const host = process.env.TAURI_DEV_HOST;

export default defineConfig({
  plugins: [solid()],
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    watch: {
      ignored: ['**/src-tauri/**', '**/target/**', '**/crates/**'],
    },
  },
  build: {
    target: ['chrome105'],
    outDir: 'dist',
  },
});
