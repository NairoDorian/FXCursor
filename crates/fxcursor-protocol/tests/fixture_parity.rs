//! Guards the shared fixture `test/fixtures/default_config.json` against drift.
//!
//! The same fixture is asserted from TypeScript (`test/config-parity.test.ts`), so if this test
//! and the Bun test both pass, `AppConfig::default()` and `getDefaultConfig()` agree exactly.
//! Refresh the fixture with `bun run fixtures` after changing defaults.

use fxcursor_protocol::AppConfig;
use std::path::PathBuf;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../test/fixtures/default_config.json")
}

#[test]
fn default_config_matches_shared_fixture() {
    let raw = std::fs::read_to_string(fixture_path())
        .expect("fixture missing: run `bun run fixtures` in ");
    let fixture: serde_json::Value = serde_json::from_str(&raw).expect("fixture is valid JSON");
    // Go through JSON text so f32 fields are printed the same way the fixture was produced
    // (`to_value` would widen 0.39f32 to 0.38999998569488525f64 and never match).
    let text = serde_json::to_string(&AppConfig::default()).expect("serialize default");
    let actual: serde_json::Value = serde_json::from_str(&text).expect("re-parse default");
    assert_eq!(
        actual, fixture,
        "AppConfig::default() drifted from test/fixtures/default_config.json; run `bun run fixtures`"
    );
}

#[test]
fn fixture_round_trips_through_deserialize() {
    let raw = std::fs::read_to_string(fixture_path()).expect("fixture missing");
    let parsed: AppConfig = serde_json::from_str(&raw).expect("fixture deserializes");
    assert_eq!(parsed, AppConfig::default());
}
