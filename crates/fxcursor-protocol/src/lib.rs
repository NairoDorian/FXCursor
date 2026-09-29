//! Core protocol definitions, configuration schema, curated presets, and self-healing deserialization
//! shared across the FXCursor engine, desktop shell, and tooling.

pub mod config;
pub mod migrate;
pub mod presets;
pub mod sanitize;
pub mod self_healing;

pub use config::*;
pub use migrate::{deserialize_config, migrate, migrate_value, CURRENT_SCHEMA_VERSION};
pub use presets::*;
pub use self_healing::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_creates() {
        let cfg = AppConfig::default();
        assert!(cfg.enabled);
        assert_eq!(cfg.trail.length, 80);
        // Out of the box only the ribbon trail is active.
        assert!(cfg.trail.enabled);
        assert_eq!(cfg.effect_mode, EffectMode::Ribbon);
        assert!(!cfg.head.enabled);
        assert!(!cfg.ripple.enabled);
        assert!(!cfg.particles.enabled);
        assert!(!cfg.satellites.enabled);
        assert!(!cfg.rainbow.enabled);
    }

    #[test]
    fn springs_and_frictions_are_plain_fractions() {
        // No hidden ÷1000 and no percentages: the slider value is the stored value.
        let cfg = AppConfig::default();
        for value in [
            cfg.trail.spring,
            cfg.trail.damping,
            cfg.trail.head_spring,
            cfg.trail.head_damping,
            cfg.head.squish_intensity,
            cfg.head.squish_smoothing,
        ] {
            assert!((0.0..=1.0).contains(&value), "{value} is out of the natural range");
        }
    }
}
