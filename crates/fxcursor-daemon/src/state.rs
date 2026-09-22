//! Daemon configuration container with thread-safe access and self-healing persistence.

use crate::portable;
use fxcursor_protocol::{AppConfig, deserialize_with_self_healing};
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::Arc;

/// Atomic file write helper: writes to a unique temporary file then renames to destination.
fn write_atomic(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pid = std::process::id();
    let tmp = path.with_extension(format!("tmp.{pid}.{id}"));
    std::fs::write(&tmp, content)?;
    if let Err(err) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(err);
    }
    Ok(())
}

/// Central configuration manager for the FXCursor background micro-daemon.
///
/// Encapsulates the live [`AppConfig`] wrapped in a thread-safe `RwLock` and manages
/// atomic persistence to `config.json` with self-healing recovery.
pub struct StateManager {
    /// Live configuration shared across IPC requests and the render loop.
    pub config: Arc<RwLock<AppConfig>>,
    /// Absolute path to the persisted configuration file.
    config_path: PathBuf,
}

impl StateManager {
    /// Initializes the state manager, loading and self-healing `config.json` if present
    /// or writing defaults if absent.
    pub fn new() -> Self {
        let app_dir = portable::get_app_data_dir();
        let config_path = app_dir.join("config.json");

        let initial_config = if config_path.exists() {
            if let Ok(raw) = std::fs::read_to_string(&config_path) {
                match deserialize_with_self_healing::<AppConfig>(&raw) {
                    Ok(outcome) => {
                        if outcome.needs_rewrite
                            && let Ok(healed_json) = serde_json::to_string_pretty(&outcome.value)
                        {
                            if let Err(e) = write_atomic(&config_path, &healed_json) {
                                log::warn!("[state] Failed to persist healed config: {e}");
                            } else {
                                log::info!(
                                    "[state] Healed settings and saved to disk. Repaired: {:?}",
                                    outcome.repaired_paths
                                );
                            }
                        }
                        outcome.value
                    }
                    Err(e) => {
                        log::warn!("[state] Failed to parse settings ({e}). Using defaults.");
                        AppConfig::default()
                    }
                }
            } else {
                AppConfig::default()
            }
        } else {
            let def = AppConfig::default();
            if let Ok(json) = serde_json::to_string_pretty(&def)
                && let Err(e) = write_atomic(&config_path, &json)
            {
                log::warn!("[state] Failed to initialize default config: {e}");
            }
            def
        };

        Self {
            config: Arc::new(RwLock::new(initial_config)),
            config_path,
        }
    }

    /// Updates the live configuration and commits it atomically to disk.
    pub fn save_config(&self, new_config: AppConfig) {
        *self.config.write() = new_config.clone();
        if let Ok(json) = serde_json::to_string_pretty(&new_config)
            && let Err(e) = write_atomic(&self.config_path, &json)
        {
            log::warn!("[state] Failed to save config to {}: {e}", self.config_path.display());
        }
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
