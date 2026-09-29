# Repository Summary (root)

> ⛔ **Never launch the app to test it.** `bun run tauri dev` is the user's job, and a bare
> `target\debug\fxcursor.exe` is far worse: a debug build has **no front end bundled in** (it
> loads from `devUrl` `http://localhost:1420`), so the webview renders an error page and the
> overlay is drawn **opaque** — a stuck non-transparent window across the desktop that has twice
> needed a **Windows reboot**. `scripts/snapshots/*.ps1` spawn `fxcursor.exe` too, so they count
> as launching it. Verify headlessly with `bun run validate` / `rtk cargo test --workspace`.
> Full rule: [`docs/DO_NOT_LAUNCH_THE_APP.md`](docs/DO_NOT_LAUNCH_THE_APP.md).

> The per-file inventory with sizes, line counts and descriptions is generated into **[`ARCHITECTURE.md`](ARCHITECTURE.md)** by `bun run arch`. This file only maps the top-level layout so the root stays accurate. Updated 2026-09-09.

| Path                                   | Purpose                                                                                                    | Status            |
| :------------------------------------- | :--------------------------------------------------------------------------------------------------------- | :---------------- |
| `./` (root)                            | **The application** (Tauri 3 + SolidJS 2 + wgpu 30). Studio UI in `src/`, Rust shell in `src-tauri/`, shared renderer in `crates/fxcursor-render`, shared config/presets in `crates/fxcursor-protocol`, experimental headless renderer in `crates/fxcursor-daemon`. | Active            |
| `docs/`                                | `V4_ARCHITECTURE_SPECIFICATION.md` (target design) and `COMPARATIVE_RESEARCH_AND_BRAINSTORMING.md`         | Reference         |
| `README.md`                            | Repository overview, feature summary, and quick start                                                      | Current           |
| `AGENTS.md`                            | Rules, golden SOP, and commands for coding agents                                                          | Current           |
| `PROGRESS.md`                          | Full status tracker, completed milestones, and roadmap                                                     | Current           |
| `CHANGELOG.md`                         | Release history (0.4.0, 0.5.0, Unreleased)                                                                | Current           |
| `memory.md`                            | Architectural decision log and budgets                                                                     | Current           |
| `repomix.config.json` / `repomix-instruction.md` | Repomix pack config and AI context sheet for repository sources                                            | Current           |
