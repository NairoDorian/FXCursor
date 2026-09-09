//! Prints `ModeMask::from_mode` for every `EffectMode` as JSON.
//!
//! `bun run fixtures` writes this to `test/fixtures/mode_masks.json`, which both the Rust test
//! `tests/mode_mask_fixture.rs` and the Bun test `test/effect-mode.test.ts` assert against, so the
//! TypeScript `modeMask()` used by the live preview cannot drift from the renderer.

use fxcursor_protocol::EffectMode;
use fxcursor_render::ModeMask;
use serde_json::{json, Map, Value};

pub const ALL_MODES: [(&str, EffectMode); 5] = [
    ("FourLayerGlow", EffectMode::FourLayerGlow),
    ("Ribbon", EffectMode::Ribbon),
    ("ParticlesOnly", EffectMode::ParticlesOnly),
    ("SatellitesOnly", EffectMode::SatellitesOnly),
    ("Minimal", EffectMode::Minimal),
];

pub fn mask_to_json(m: &ModeMask) -> Value {
    json!({
        "trail": m.trail,
        "layers": m.layers,
        "head": m.head,
        "ripples": m.ripples,
        "particles": m.particles,
        "satellites": m.satellites,
    })
}

pub fn all_masks() -> Value {
    let mut map = Map::new();
    for (name, mode) in ALL_MODES {
        map.insert(name.to_string(), mask_to_json(&ModeMask::from_mode(mode)));
    }
    Value::Object(map)
}

fn main() {
    println!("{}", serde_json::to_string_pretty(&all_masks()).expect("serialize masks"));
}
