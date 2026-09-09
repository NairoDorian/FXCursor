//! Portable mode detection and path resolution.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use tauri::{AppHandle, Manager};

const MARKER_FILE_NAME: &str = "portable";
const DATA_DIR_NAME: &str = "Data";

static PORTABLE_DATA_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

fn resolve_data_dir(exe_dir: &Path) -> Option<PathBuf> {
    if !exe_dir.join(MARKER_FILE_NAME).exists() {
        return None;
    }

    let data_dir = exe_dir.join(DATA_DIR_NAME);
    match std::fs::create_dir_all(&data_dir) {
        Ok(()) => Some(data_dir),
        Err(err) => {
            eprintln!(
                "[portable] '{}' marker found but {} is not writable ({err}); falling back to OS directories.",
                MARKER_FILE_NAME,
                data_dir.display()
            );
            None
        }
    }
}

pub fn init() {
    PORTABLE_DATA_DIR.get_or_init(|| {
        let exe_path = std::env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        let resolved = resolve_data_dir(exe_dir);

        if let Some(dir) = &resolved {
            println!("[portable] Portable mode active — data directory: {}", dir.display());
        }

        resolved
    });
}

pub fn data_dir() -> Option<&'static PathBuf> {
    PORTABLE_DATA_DIR.get().and_then(Option::as_ref)
}

pub fn is_active() -> bool {
    data_dir().is_some()
}

pub fn config_dir(app: &AppHandle) -> Result<PathBuf, tauri::Error> {
    match data_dir() {
        Some(dir) => Ok(dir.clone()),
        None => app.path().app_config_dir(),
    }
}
