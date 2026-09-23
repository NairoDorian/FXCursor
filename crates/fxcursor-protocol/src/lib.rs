//! Core protocol definitions, configuration schema, curated presets, and self-healing deserialization
//! shared across the FXCursor engine, desktop shell, and tooling.

pub mod config;
pub mod presets;
pub mod sanitize;
pub mod self_healing;

pub use config::*;
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
        // Out of the box only the ribbon trail is active (legacy V3 parity).
        assert!(cfg.trail.enabled);
        assert_eq!(cfg.effect_mode, EffectMode::Ribbon);
        assert!(!cfg.head.enabled);
        assert!(!cfg.ripple.enabled);
        assert!(!cfg.particles.enabled);
        assert!(!cfg.satellites.enabled);
        assert!(!cfg.rainbow.enabled);
    }
}
