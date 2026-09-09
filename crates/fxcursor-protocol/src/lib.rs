pub mod config;
pub mod presets;
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
    }
}
