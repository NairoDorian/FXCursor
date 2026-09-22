# Repository Summary (root)

> The per-file inventory with sizes, line counts and descriptions is generated into **[`ARCHITECTURE.md`](ARCHITECTURE.md)** by `bun run arch`. This file only maps the top-level layout so the root stays accurate. Updated 2026-09-09.

| Path                                   | Purpose                                                                                                    | Status            |
| :------------------------------------- | :--------------------------------------------------------------------------------------------------------- | :---------------- |
| `./` (root)                            | **V4 application** (Tauri 2 + SolidJS 2 + wgpu 30). Studio UI in `src/`, Rust shell in `src-tauri/`, shared renderer in `crates/fxcursor-render`, shared config/presets in `crates/fxcursor-protocol`, experimental headless renderer in `crates/fxcursor-daemon`. | Active            |
| `docs/`                                | `V4_ARCHITECTURE_SPECIFICATION.md` (target design) and `COMPARATIVE_RESEARCH_AND_BRAINSTORMING.md`         | Reference         |
| `legacy/project_cursor/`               | V3 app (Tauri 2 + React 19 + Tailwind 4 + wgpu 29). Last tracked commit `9f6cc17`.                          | Legacy, frozen    |
| `legacy/original_mods/`                | `D3D_cursor_mod.wh.cpp`, `gdi+_cursor_mod.wh.cpp` — C++ Windhawk mods that defined the 4-layer look         | Reference         |
| `legacy/dev_scripts/`                  | PowerShell/CMD helpers (`build`, `cargo_check`, `cargo_run`, `update_dependencies`, `generate_repomix`) written for `legacy/project_cursor/`; see `build_instructions.md` | Legacy            |
| `README.md`                            | Repository overview, feature summary, and quick start                                                      | Current           |
| `AGENTS.md`                            | Rules, golden SOP, and commands for coding agents                                                          | Current           |
| `PROGRESS.md`                          | Full status tracker, completed milestones, and roadmap                                                     | Current           |
| `CHANGELOG.md`                         | Release history (0.2.0 V3, 0.4.0 V4, 0.5.0 FXCursor, Unreleased)                                           | Current           |
| `memory.md`                            | Architectural decision log and budgets                                                                     | Current           |
| `repomix.config.json` / `repomix-instruction.md` | Repomix pack config and AI context sheet for repository sources                                            | Current           |
