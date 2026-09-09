//! Prints `AppConfig::default()` as pretty JSON.
//!
//! Used by `bun run fixtures` to refresh `test/fixtures/default_config.json`, the shared fixture
//! that keeps the Rust defaults and the TypeScript `getDefaultConfig()` mirror in lock-step.

fn main() {
    let cfg = fxcursor_protocol::AppConfig::default();
    println!("{}", serde_json::to_string_pretty(&cfg).expect("serialize default config"));
}
