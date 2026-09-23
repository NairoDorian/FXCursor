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

    let mut outcome: RepairOutcome<AppConfig> = match deserialize_with_self_healing(&raw) {
        Ok(o) => o,
        Err(err) => {
            log::error!("[settings] self-healing failed ({err}); using defaults");
            return AppConfig::default();
        }
    };
    // Types are healed above; ranges here (a hand-edited file can hold anything).
    let clamped = outcome.value.sanitize();
    if !clamped.is_empty() {
        log::warn!("[settings] clamped out-of-range value(s): {clamped:?}");
        outcome.repaired_paths.extend(clamped.iter().map(|p| p.to_string()));
        outcome.needs_rewrite = true;
    }

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
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let id = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let pid = std::process::id();
    let tmp = path.with_extension(format!("tmp.{pid}.{id}"));
    std::fs::write(&tmp, json).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    if let Err(err) = std::fs::rename(&tmp, path) {
        let _ = std::fs::remove_file(&tmp);
        return Err(format!("rename {} to {}: {err}", tmp.display(), path.display()));
    }
    Ok(())
}

enum SaveMsg {
    // Boxed: an `AppConfig` is ~0.5 KB and the flush variant is a single sender.
    Save(Box<AppConfig>),
    /// Write any pending snapshot now, then acknowledge.
    Flush(Sender<()>),
}

/// Debounced background writer. Cloneable handle; drop all handles to stop the thread.
#[derive(Clone)]
pub struct AutoSaver {
    tx: Sender<SaveMsg>,
}

impl AutoSaver {
    pub fn spawn(path: PathBuf) -> Self {
        let (tx, rx) = mpsc::channel::<SaveMsg>();
        let write = move |config: &AppConfig| match save(&path, config) {
            Ok(()) => log::debug!("[settings] autosaved {}", path.display()),
            Err(err) => log::warn!("[settings] autosave failed: {err}"),
        };
        std::thread::Builder::new()
            .name("config-autosave".into())
            .spawn(move || {
                let mut pending: Option<AppConfig> = None;
                loop {
                    // Coalesce: while a snapshot is pending, keep swallowing updates until the
                    // stream goes quiet for AUTOSAVE_DEBOUNCE.
                    let msg = if pending.is_some() {
                        match rx.recv_timeout(AUTOSAVE_DEBOUNCE) {
                            Ok(msg) => msg,
                            Err(RecvTimeoutError::Timeout) => {
                                if let Some(config) = pending.take() {
                                    write(&config);
                                }
                                continue;
                            }
                            Err(RecvTimeoutError::Disconnected) => {
                                if let Some(config) = pending.take() {
                                    write(&config);
                                }
                                return;
                            }
                        }
                    } else {
                        match rx.recv() {
                            Ok(msg) => msg,
                            Err(_) => return,
                        }
                    };
                    match msg {
                        SaveMsg::Save(config) => pending = Some(*config),
                        SaveMsg::Flush(ack) => {
                            if let Some(config) = pending.take() {
                                write(&config);
                            }
                            let _ = ack.send(());
                        }
                    }
                }
            })
            .expect("spawn config-autosave thread");
        Self { tx }
    }

    /// Queue a snapshot for writing. Cheap; safe to call on every slider tick.
    pub fn schedule(&self, config: &AppConfig) {
        let _ = self.tx.send(SaveMsg::Save(Box::new(config.clone())));
    }

    /// Writes any snapshot still inside the debounce window and waits (bounded) for it.
    /// Called on exit: `process::exit` would otherwise kill the thread and lose the last edit.
    pub fn flush(&self, timeout: std::time::Duration) -> bool {
        let (ack_tx, ack_rx) = mpsc::channel();
        self.tx.send(SaveMsg::Flush(ack_tx)).is_ok() && ack_rx.recv_timeout(timeout).is_ok()
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
    fn out_of_range_values_are_clamped_on_load() {
        let path = temp_path("clamped.json");
        let mut cfg = AppConfig::default();
        cfg.trail.spring = 9_000.0;
        save(&path, &cfg).unwrap();
        let loaded = load_or_repair(&path);
        assert_eq!(loaded.trail.spring, 500.0);
        let _ = std::fs::remove_file(&path);
        let _ = std::fs::remove_file(path.with_extension("json.bak"));
    }

    #[test]
    fn flush_writes_a_pending_snapshot_immediately() {
        let path = temp_path("flushed.json");
        let _ = std::fs::remove_file(&path);
        let saver = AutoSaver::spawn(path.clone());
        let mut cfg = AppConfig::default();
        cfg.trail.length = 33;
        saver.schedule(&cfg);
        assert!(saver.flush(std::time::Duration::from_secs(2)));
        assert_eq!(load_or_repair(&path).trail.length, 33, "written before the debounce");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn missing_file_yields_defaults() {
        let path = temp_path("does-not-exist.json");
        assert_eq!(load_or_repair(&path), AppConfig::default());
    }
}
