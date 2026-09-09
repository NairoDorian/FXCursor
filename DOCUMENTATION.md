# Upstream Documentation & Reference Architecture

This repository adopts the **Documentation-First Standard** from the Minimalistic App reference architecture. Upstream documentation for every technology stack layer can be mirrored locally under `.docs/` for offline browsing, AI coding agents, and fast cross-mirror search.

---

## 📚 Document Mirror Manifest

Run `bun run docs:sync` to clone or refresh the local documentation mirrors:

| Stack Layer       | Local Mirror Directory               | Pinned Branch | Source Repository                                     |
| :---------------- | :----------------------------------- | :------------ | :---------------------------------------------------- |
| **Tauri 2**       | `.docs/tauri-docs/src/content/docs/` | `v2`          | `https://github.com/tauri-apps/tauri-docs.git`        |
| **SolidJS 2**     | `.docs/solid-docs/src/routes/`       | `v2-rebuild`  | `https://github.com/solidjs/solid-docs.git`           |
| **Bun.js**        | `.docs/bun-docs/content/docs/`       | `main`        | `https://github.com/oven-sh/bun.git`                  |
| **TypeScript 7**  | `.docs/typescript-website/packages/` | `v2`          | `https://github.com/microsoft/TypeScript-Website.git` |
| **wgpu (WebGPU)** | `.docs/wgpu-docs/`                   | `trunk`       | `https://github.com/gfx-rs/wgpu.git`                  |

---

## 🔍 Commands

```bash
# Clone or fast-forward all upstream documentation mirrors (~200MB, gitignored)
bun run docs:sync

# Check status of local documentation mirrors
bun run docs:check

# Search across all documentation mirrors simultaneously
bun run docs:find "capabilities"
```
