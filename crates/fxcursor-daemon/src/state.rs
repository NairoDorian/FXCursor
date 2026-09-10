use crate::portable;
use fxcursor_protocol::{AppConfig, deserialize_with_self_healing};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

pub struct StateManager {
    pub config: Arc<RwLock<AppConfig>>,
    config_path: PathBuf,
}

impl StateManager {
    pub fn new() -> Self {
        let app_dir = portable::get_app_data_dir();
        let config_path = app_dir.join("settings.json");

        let initial_config = if config_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&config_path) {
                match deserialize_with_self_healing::<AppConfig>(&raw) {
                    Ok(outcome) => {
                        if outcome.needs_rewrite
                            && let Ok(healed_json) = serde_json::to_string_pretty(&outcome.value)
                        {
                            let _ = std::fs::write(&config_path, healed_json);
                            log::info!(
                                "[state] Healed settings and saved to disk. Repaired: {:?}",
                                outcome.repaired_paths
                            );
                        }
                        outcome.value
                    }
                    Err(e) => {
                        log::warn!("[state] Failed to parse settings ({}). Using defaults.", e);
                        AppConfig::default()
                    }
                }
            } else {
                AppConfig::default()
            }
        } else {
            let def = AppConfig::default();
            if let Ok(json) = serde_json::to_string_pretty(&def) {
                let _ = std::fs::write(&config_path, json);
            }
            def
        };

        Self {
            config: Arc::new(RwLock::new(initial_config)),
            config_path,
        }
    }

    pub fn save_config(&self, new_config: AppConfig) {
        *self.config.write() = new_config.clone();
        if let Ok(json) = serde_json::to_string_pretty(&new_config) {
            let _ = std::fs::write(&self.config_path, json);
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
