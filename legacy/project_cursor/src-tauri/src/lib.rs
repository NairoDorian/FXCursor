#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

mod config;
mod overlay;
mod tracker;

use config::AppConfig;
use overlay::OverlayState;
use std::sync::{Arc, Mutex};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Arc::new(Mutex::new(AppConfig::load_or_default())))
        .invoke_handler(tauri::generate_handler![
            commands::get_config,
            commands::update_config,
            commands::save_config,
            commands::reset_defaults,
            commands::toggle_overlay,
            commands::get_overlay_status,
        ])
        .setup(|app| {
            use tauri::WebviewWindowBuilder;

            let primary_monitor = app
                .get_webview_window("main")
                .and_then(|w| w.available_monitors().ok())
                .and_then(|monitors| monitors.into_iter().next());

            let mut overlay_builder = WebviewWindowBuilder::new(
                app,
                "overlay",
                tauri::WebviewUrl::App("overlay.html".into()),
            )
            .transparent(true)
            .decorations(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(true)
            .focused(false);

            if let Some(ref monitor) = primary_monitor {
                let size = monitor.size();
                let pos = monitor.position();
                overlay_builder = overlay_builder
                    .inner_size(size.width as f64, size.height as f64)
                    .position(pos.x as f64, pos.y as f64);
            } else {
                overlay_builder = overlay_builder
                    .inner_size(1920.0, 1080.0)
                    .position(0.0, 0.0);
            }

            let _overlay = overlay_builder
                .build()
                .expect("Failed to create overlay window");

            log::info!("Overlay window created successfully");

            let handle = app.handle().clone();
            let config = app.state::<Arc<Mutex<AppConfig>>>().inner().clone();

            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if let Err(e) = OverlayState::run_render_loop(handle, config) {
                    log::error!("Overlay render loop failed: {}", e);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

mod commands {
    use crate::config::AppConfig;
    use std::sync::{Arc, Mutex};
    use tauri::State;

    #[tauri::command]
    pub fn get_config(state: State<Arc<Mutex<AppConfig>>>) -> Result<AppConfig, String> {
        state.lock().map_err(|e| e.to_string()).map(|c| c.clone())
    }

    #[tauri::command]
    pub fn update_config(
        state: State<Arc<Mutex<AppConfig>>>,
        config: AppConfig,
    ) -> Result<(), String> {
        let mut current = state.lock().map_err(|e| e.to_string())?;
        *current = config;
        Ok(())
    }

    #[tauri::command]
    pub fn save_config(state: State<Arc<Mutex<AppConfig>>>) -> Result<(), String> {
        let config = state.lock().map_err(|e| e.to_string())?;
        config.save().map_err(|e| e.to_string())
    }

    #[tauri::command]
    pub fn reset_defaults(state: State<Arc<Mutex<AppConfig>>>) -> Result<AppConfig, String> {
        let mut config = state.lock().map_err(|e| e.to_string())?;
        let defaults = AppConfig::default();
        *config = defaults.clone();
        config.save().map_err(|e| e.to_string())?;
        Ok(defaults)
    }

    #[tauri::command]
    pub fn toggle_overlay(
        state: State<Arc<Mutex<AppConfig>>>,
        enabled: bool,
    ) -> Result<(), String> {
        let mut config = state.lock().map_err(|e| e.to_string())?;
        config.enabled = enabled;
        Ok(())
    }

    #[tauri::command]
    pub fn get_overlay_status(state: State<Arc<Mutex<AppConfig>>>) -> Result<bool, String> {
        state.lock().map_err(|e| e.to_string()).map(|c| c.enabled)
    }
}
