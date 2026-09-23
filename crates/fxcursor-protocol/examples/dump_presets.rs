//! Prints the built-in presets as JSON, each config exactly as `apply_preset` stores it
//! (`general.selected_preset` = the preset id).
//!
//! `bun run fixtures` writes this to `src/lib/generated/builtin_presets.json`, which the Studio
//! uses in browser preview mode. Rust stays the single source of truth; the JSON used to be a
//! hand-maintained TypeScript copy that had drifted in 5 of 6 presets (false "MODIFIED" badges).

fn main() {
    let presets: Vec<_> = fxcursor_protocol::get_builtin_presets()
        .into_iter()
        .map(|mut p| {
            p.config.general.selected_preset = p.id.clone();
            p
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&presets).expect("serialize presets"));
}
