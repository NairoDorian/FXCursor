//! Guards `test/fixtures/mode_masks.json` (produced by `bun run fixtures`) against drift from
//! `ModeMask::from_mode`. The Bun test `test/effect-mode.test.ts` asserts the TypeScript mirror
//! against the same fixture.

use fxcursor_protocol::EffectMode;
use fxcursor_render::ModeMask;
use serde_json::{json, Value};
use std::path::PathBuf;

const ALL_MODES: [(&str, EffectMode); 5] = [
    ("FourLayerGlow", EffectMode::FourLayerGlow),
    ("Ribbon", EffectMode::Ribbon),
    ("ParticlesOnly", EffectMode::ParticlesOnly),
    ("SatellitesOnly", EffectMode::SatellitesOnly),
    ("Minimal", EffectMode::Minimal),
];

fn mask_to_json(m: &ModeMask) -> Value {
    json!({
        "trail": m.trail,
        "layers": m.layers,
        "head": m.head,
        "ripples": m.ripples,
        "particles": m.particles,
        "satellites": m.satellites,
    })
}

fn fixture() -> Value {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test/fixtures/mode_masks.json");
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("fixture missing at {} ({e}): run `bun run fixtures`", path.display()));
    serde_json::from_str(&raw).expect("fixture is valid JSON")
}

#[test]
fn every_mode_matches_the_shared_fixture() {
    let fixture = fixture();
    for (name, mode) in ALL_MODES {
        let expected = fixture
            .get(name)
            .unwrap_or_else(|| panic!("fixture lacks mode {name}; run `bun run fixtures`"));
        assert_eq!(
            &mask_to_json(&ModeMask::from_mode(mode)),
            expected,
            "ModeMask::from_mode({name}) drifted from test/fixtures/mode_masks.json"
        );
    }
}

#[test]
fn fixture_has_no_unknown_modes() {
    let fixture = fixture();
    let obj = fixture.as_object().expect("fixture is an object");
    assert_eq!(obj.len(), ALL_MODES.len(), "fixture lists a mode the renderer does not know");
}
