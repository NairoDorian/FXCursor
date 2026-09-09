//! Persistent configuration store with field-level self-healing.
//!
//! Responsibilities:
//! - Resolve the on-disk location of `config.json` (portable-aware via [`crate::portable`]).
//! - Load the file through `fxcursor_protocol::deserialize_with_self_healing`, so a single
//!   broken field is reset to its default instead of discarding the whole document.
//! - Write the healed document back (with a `.bak` copy of the corrupted original).
//! - Provide a debounced background auto-saver so slider drags do not hammer the disk.

use fxcursor_protocol::{deserialize_with_self_healing, AppConfig, RepairOutcome};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, RecvTimeoutError, Sender};
use std::time::Duration;
use tauri::AppHandle;

pub const CONFIG_FILE_NAME: &str = "config.json";

/// Coalescing window for auto-save. Every `update_config` call resets the timer; the file is
/// written once the stream of updates has been quiet for this long.
const AUTOSAVE_DEBOUNCE: Duration = Duration::from_millis(400);

/// Identifiers the app shipped under before (newest first); the first config directory found
/// is adopted on first launch so nobody loses their settings.
const LEGACY_IDENTIFIERS: &[&str] = &["com.fxcursor.app", "com.cursorfx.studio"];

/// Resolves `<config_dir>/config.json`, creating the directory if needed. On the very first
/// launch after the rename the previous installation's config is copied over.
pub fn config_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = crate::portable::config_dir(app).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    let path = dir.join(CONFIG_FILE_NAME);
    if !path.exists() && !crate::portable::is_active() {
        migrate_legacy_config(&dir, &path);
    }
    Ok(path)
}

/// Copies the first existing `../<legacy identifier>/config.json` next to `path`.
fn migrate_legacy_config(dir: &Path, path: &Path) {
    let Some(parent) = dir.parent() else {
        return;
    };
    let Some(legacy) = LEGACY_IDENTIFIERS
        .iter()
        .map(|id| parent.join(id).join(CONFIG_FILE_NAME))
        .find(|candidate| candidate.exists())
    else {
        return;
    };
    match std::fs::copy(&legacy, path) {
        Ok(_) => log::info!(
            "[settings] adopted configuration from the previous install at {}",
            legacy.display()
        ),
        Err(err) => log::warn!("[settings] could not copy {}: {err}", legacy.display()),
    }
}

/// Loads the configuration from `path`. Never fails: a missing file yields defaults, and a
/// damaged file is healed field-by-field and rewritten.
pub fn load_or_repair(path: &Path) -> AppConfig {
    if !path.exists() {
        log::info!("[settings] no config at {}; using defaults", path.display());
        return AppConfig::default();
    }

    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(err) => {
            log::warn!("[settings] cannot read {} ({err}); using defaults", path.display());
            return AppConfig::default();
        }
    };

    let outcome: RepairOutcome<AppConfig> = match deserialize_with_self_healing(&raw) {
        Ok(o) => o,
        Err(err) => {
            log::error!("[settings] self-healing failed ({err}); using defaults");
            return AppConfig::default();
        }
    };

    if outcome.needs_rewrite {
        log::warn!(
            "[settings] repaired {} field(s) in {}: {:?}",
            outcome.repaired_paths.len(),
            path.display(),
            outcome.repaired_paths
        );
        let backup = path.with_extension("json.bak");
        if let Err(err) = std::fs::copy(path, &backup) {
            log::warn!("[settings] could not write backup {} ({err})", backup.display());
        }
        if let Err(err) = save(path, &outcome.value) {
            log::warn!("[settings] could not persist healed config ({err})");
        }
    } else {
        log::info!("[settings] loaded {}", path.display());
    }

    outcome.value
}

/// Atomically writes `config` to `path` (temp file + rename).
pub fn save(path: &Path, config: &AppConfig) -> Result<(), String> {
    let json = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, path).map_err(|e| format!("rename to {}: {e}", path.display()))?;
    Ok(())
}

/// Debounced background writer. Cloneable handle; drop all handles to stop the thread.
#[derive(Clone)]
pub struct AutoSaver {
    tx: Sender<AppConfig>,
}

impl AutoSaver {
    pub fn spawn(path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<AppConfig>();
        std::thread::Builder::new()
            .name("config-autosave".into())
            .spawn(move || {
                while let Ok(mut latest) = rx.recv() {
                    // Coalesce: keep swallowing updates until the stream goes quiet.
                    loop {
                        match rx.recv_timeout(AUTOSAVE_DEBOUNCE) {
                            Ok(newer) => latest = newer,
                            Err(RecvTimeoutError::Timeout) => break,
                            Err(RecvTimeoutError::Disconnected) => {
                                let _ = save(&path, &latest);
                                return;
                            }
                        }
                    }
                    if let Err(err) = save(&path, &latest) {
                        log::warn!("[settings] autosave failed: {err}");
                    } else {
                        log::debug!("[settings] autosaved {}", path.display());
                    }
                }
            })
            .expect("spawn config-autosave thread");
        Self { tx }
    }

    /// Queue a snapshot for writing. Cheap; safe to call on every slider tick.
    pub fn schedule(&self, config: &AppConfig) {
        let _ = self.tx.send(config.clone());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("fxcursor-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn save_then_load_round_trips() {
        let path = temp_path("roundtrip.json");
        let mut cfg = AppConfig::default();
        cfg.trail.length = 42;
        cfg.satellites.enabled = true;
        save(&path, &cfg).unwrap();
        let loaded = load_or_repair(&path);
        assert_eq!(loaded, cfg);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn corrupted_field_is_healed_and_backed_up() {
        let path = temp_path("healed.json");
        std::fs::write(&path, r#"{ "enabled": false, "trail": { "length": "eighty" } }"#).unwrap();
        let loaded = load_or_repair(&path);
        assert!(!loaded.enabled, "valid fields must survive");
        assert_eq!(loaded.trail.length, AppConfig::default().trail.length);
        assert!(path.with_extension("json.bak").exists());
        // Healed file must now load cleanly.
        let reloaded = load_or_repair(&path);
        assert_eq!(reloaded, loaded);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("json.bak"));
    }

    #[test]
    fn missing_file_yields_defaults() {
        let path = temp_path("does-not-exist.json");
        assert_eq!(load_or_repair(&path), AppConfig::default());
    }
}
