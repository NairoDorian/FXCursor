use std::path::PathBuf;
use std::sync::OnceLock;

const MARKER_FILE_NAME: &str = "portable";
const DATA_DIR_NAME: &str = "Data";

static PORTABLE_DATA_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

pub fn init() {
    PORTABLE_DATA_DIR.get_or_init(|| {
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            let marker = exe_dir.join(MARKER_FILE_NAME);
            if marker.exists() {
                let data_dir = exe_dir.join(DATA_DIR_NAME);
                let _ = std::fs::create_dir_all(&data_dir);
                log::info!("[portable] Portable mode active. Data dir: {:?}", data_dir);
                return Some(data_dir);
            }
        }
        None
    });
}

pub fn get_app_data_dir() -> PathBuf {
    if let Some(Some(dir)) = PORTABLE_DATA_DIR.get() {
        return dir.clone();
    }

    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "FXCursor", "FXCursor") {
        let config_dir = proj_dirs.config_dir().to_path_buf();
        let _ = std::fs::create_dir_all(&config_dir);
        return config_dir;
    }

    PathBuf::from(".")
}
