//! Custom presets: named snapshots of the effect configuration stored as JSON files in
//! `<config dir>/presets/`. They are listed after the built-in presets and use the same
//! `PresetInfo` shape, so the Studio treats both kinds alike; only user presets (ids starting
//! with [`USER_PREFIX`]) can be deleted.

use fxcursor_protocol::{deserialize_with_self_healing, AppConfig, PresetInfo, RepairOutcome};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tauri::AppHandle;

/// Id prefix that marks a user preset (and its file name).
pub const USER_PREFIX: &str = "user-";
/// Sub-directory of the config directory holding the preset files.
pub const PRESETS_DIR: &str = "presets";

#[derive(Debug, Default, Serialize, Deserialize)]
struct PresetFile {
    name: String,
    #[serde(default)]
    description: String,
    config: AppConfig,
}

/// `<config dir>/presets`, created on demand.
pub fn presets_dir(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = crate::portable::config_dir(app)
        .map_err(|e| e.to_string())?
        .join(PRESETS_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| format!("cannot create {}: {e}", dir.display()))?;
    Ok(dir)
}

/// Turns a display name into a stable file/preset id: `user-<lowercase-ascii-slug>`.
pub fn preset_id(name: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = true;
    for c in name.trim().chars() {
        if c.is_ascii_alphanumeric() {
            slug.push(c.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }
    let slug = slug.trim_end_matches('-');
    let slug = if slug.is_empty() { "preset" } else { slug };
    format!("{USER_PREFIX}{}", &slug[..slug.len().min(48)])
}

pub fn is_user_preset(id: &str) -> bool {
    id.starts_with(USER_PREFIX)
}

fn file_for(dir: &Path, id: &str) -> PathBuf {
    dir.join(format!("{id}.json"))
}

/// All user presets, sorted by name. Unreadable files are skipped with a warning; slightly
/// damaged ones are healed like the main configuration.
pub fn list(app: &AppHandle) -> Vec<PresetInfo> {
    let Ok(dir) = presets_dir(app) else {
        return Vec::new();
    };
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };
    let mut presets: Vec<PresetInfo> = entries
        .flatten()
        .filter(|e| e.path().extension().is_some_and(|x| x == "json"))
        .filter_map(|e| {
            let path = e.path();
            let id = path.file_stem()?.to_string_lossy().into_owned();
            if !is_user_preset(&id) {
                return None;
            }
            let raw = std::fs::read_to_string(&path).ok()?;
            match deserialize_with_self_healing::<PresetFile>(&raw) {
                Ok(RepairOutcome { value, repaired_paths, .. }) => {
                    if !repaired_paths.is_empty() {
                        log::warn!(
                            "[presets] {} healed ({} field(s))",
                            path.display(),
                            repaired_paths.len()
                        );
                    }
                    Some(PresetInfo {
                        id,
                        name: value.name,
                        description: value.description,
                        config: value.config,
                    })
                }
                Err(err) => {
                    log::warn!("[presets] skipping {}: {err}", path.display());
                    None
                }
            }
        })
        .collect();
    presets.sort_by_key(|p| p.name.to_lowercase());
    presets
}

/// Saves `config` under `name` (overwriting a preset with the same id) and returns it.
pub fn save(
    app: &AppHandle,
    name: &str,
    description: &str,
    config: &AppConfig,
) -> Result<PresetInfo, String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("preset name is empty".to_string());
    }
    let id = preset_id(name);
    let dir = presets_dir(app)?;
    let mut stored = config.clone();
    stored.general.selected_preset = id.clone();
    let file = PresetFile {
        name: name.to_string(),
        description: description.trim().to_string(),
        config: stored.clone(),
    };
    let json = serde_json::to_string_pretty(&file).map_err(|e| e.to_string())?;
    let path = file_for(&dir, &id);
    std::fs::write(&path, json).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
    log::info!("[presets] saved '{name}' as {}", path.display());
    Ok(PresetInfo {
        id,
        name: name.to_string(),
        description: description.trim().to_string(),
        config: stored,
    })
}

/// Deletes a user preset file. Built-in ids are refused.
pub fn delete(app: &AppHandle, id: &str) -> Result<(), String> {
    if !is_user_preset(id) {
        return Err(format!("'{id}' is a built-in preset and cannot be deleted"));
    }
    let path = file_for(&presets_dir(app)?, id);
    std::fs::remove_file(&path).map_err(|e| format!("cannot delete {}: {e}", path.display()))?;
    log::info!("[presets] deleted {}", path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_stable_slugs_with_the_user_prefix() {
        assert_eq!(preset_id("My Neon Look!"), "user-my-neon-look");
        assert_eq!(preset_id("  spaced   out  "), "user-spaced-out");
        assert_eq!(preset_id("***"), "user-preset");
        assert_eq!(preset_id("Ünïcode Näme"), "user-n-code-n-me");
        assert!(is_user_preset(&preset_id("x")));
        assert!(!is_user_preset("master_4layer"));
    }

    #[test]
    fn preset_files_round_trip_through_the_healing_deserializer() {
        let file = PresetFile {
            name: "Round trip".into(),
            description: "desc".into(),
            config: AppConfig::default(),
        };
        let json = serde_json::to_string(&file).unwrap();
        let back: RepairOutcome<PresetFile> = deserialize_with_self_healing(&json).unwrap();
        assert!(back.repaired_paths.is_empty());
        assert_eq!(back.value.name, "Round trip");
        assert_eq!(back.value.config, AppConfig::default());
        // A preset written by an older version (missing fields) still loads.
        let partial = r#"{"name":"Old","config":{"trail":{"length":12}}}"#;
        let healed: RepairOutcome<PresetFile> = deserialize_with_self_healing(partial).unwrap();
        assert_eq!(healed.value.config.trail.length, 12);
        assert!(!healed.repaired_paths.is_empty());
    }
}
