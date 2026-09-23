#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

pub mod capture;
pub mod cursor;
pub mod display;
pub mod input;
pub mod integrations;
pub mod logger;
pub mod overlay;
pub mod panic_log;
pub mod portable;
pub mod settings_repair;
pub mod user_presets;
#[cfg(not(target_os = "windows"))]
pub mod tracker;

use fxcursor_protocol::{
    deserialize_with_self_healing, get_builtin_presets, AppConfig, PresetInfo, RepairOutcome,
};
use overlay::OverlayState;
use serde::Serialize;
use settings_repair::AutoSaver;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Instant;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};
use tauri_specta::{collect_commands, ErrorHandlingMode};

/// Live configuration shared between IPC commands, the tray, hotkeys and the render thread.
pub type SharedConfig = Arc<Mutex<AppConfig>>;

/// Where the configuration lives on disk plus the debounced writer that keeps it current.
pub struct Persistence {
    pub path: PathBuf,
    pub saver: AutoSaver,
}

/// Facts discovered at runtime (GPU adapter, start time, frame statistics) for diagnostics.
pub struct RuntimeInfo {
    pub started: Instant,
    pub gpu: Mutex<Option<GpuInfo>>,
    pub frame: Mutex<FrameStats>,
    /// Pending overlay snapshots, fulfilled by the render thread (see `capture.rs`).
    pub captures: Mutex<Vec<capture::CaptureRequest>>,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct GpuInfo {
    pub adapter: String,
    pub backend: String,
    pub device_type: String,
}

/// Rolling render-loop statistics, refreshed by the render thread roughly twice a second.
#[derive(Debug, Clone, Default, Serialize, specta::Type)]
pub struct FrameStats {
    /// Presented frames per second over the last window (0 while idle).
    pub fps: f32,
    /// Average CPU time per presented frame in milliseconds (physics + mesh + submit).
    pub frame_ms: f32,
    /// "active" (animating), "settling" (flushing final frames) or "idle" (parked).
    pub state: String,
    /// Ribbon vertices uploaded in the last frame.
    pub ribbon_vertices: u32,
    /// SDF instances (head, ripples, particles, satellites) in the last frame.
    pub instances: u32,
}

pub const CONFIG_UPDATED_EVENT: &str = "config-updated";

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct SystemDiagnostics {
    pub os: String,
    pub arch: String,
    pub portable: bool,
    pub app_version: String,
    pub config_path: String,
    pub screen_virtual_origin: [i32; 2],
    pub screen_virtual_size: [u32; 2],
    pub gpu: Option<GpuInfo>,
    /// "hook" when the OS-level mouse hook is delivering events, otherwise "poll".
    pub input_backend: String,
    pub uptime_secs: f64,
    pub debug_build: bool,
    pub frame: FrameStats,
    /// Result of the last global-hotkey registration, e.g. "registered 'Ctrl+Shift+E'" or an
    /// error message when another application owns the shortcut.
    pub hotkey_status: String,
    /// Display refresh rate the render loop paces to when `general.max_fps` is 0.
    pub display_refresh_hz: f32,
}

#[derive(Debug, Clone, Serialize, specta::Type)]
pub struct ImportOutcome {
    pub config: AppConfig,
    pub repaired_paths: Vec<String>,
}

// ---------------------------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------------------------

/// Broadcasts a new configuration snapshot: notifies webviews, queues an autosave, syncs OS
/// integrations and wakes the render thread. Callers must NOT hold the config mutex.
pub fn commit(app: &AppHandle, snapshot: &AppConfig) {
    let _ = app.emit(CONFIG_UPDATED_EVENT, snapshot);
    if let Some(p) = app.try_state::<Persistence>() {
        p.saver.schedule(snapshot);
    }
    integrations::sync(app, snapshot);
    update_tray_tooltip(app, snapshot);
    if let Some(hub) = app.try_state::<Arc<input::InputHub>>() {
        hub.notify();
    }
}

pub const TRAY_ID: &str = "fxcursor-tray";

/// Tray tooltip showing the live state, e.g. `FXCursor · effects on · Master 4-Layer`.
pub fn tray_tooltip(config: &AppConfig) -> String {
    let preset_id = config.general.selected_preset.as_str();
    let preset_name = get_builtin_presets()
        .into_iter()
        .find(|p| p.id == preset_id)
        .map(|p| p.name)
        .unwrap_or_else(|| preset_id.trim_start_matches(user_presets::USER_PREFIX).replace('-', " "));
    format!(
        "FXCursor · effects {} · {}",
        if config.enabled { "on" } else { "off" },
        preset_name
    )
}

fn update_tray_tooltip(app: &AppHandle, config: &AppConfig) {
    if let Some(tray) = app.tray_by_id(TRAY_ID) {
        let _ = tray.set_tooltip(Some(tray_tooltip(config)));
    }
}

/// Replaces the live configuration and commits it. Returns the stored snapshot.
///
/// Every external source funnels through here (IPC, `--apply`, presets, import, reset), so
/// this is where out-of-range values are clamped before the renderer ever sees them.
fn replace_config(app: &AppHandle, mut new_config: AppConfig) -> Result<AppConfig, String> {
    let clamped = new_config.sanitize();
    if !clamped.is_empty() {
        log::warn!("[config] clamped out-of-range value(s): {clamped:?}");
    }
    let state = app
        .try_state::<SharedConfig>()
        .ok_or("configuration state not initialised")?;
    {
        let mut current = state.lock().map_err(|e| e.to_string())?;
        *current = new_config.clone();
    }
    commit(app, &new_config);
    Ok(new_config)
}

/// Flips `enabled`, commits, and returns the new value. Used by the tray, the hotkey and IPC.
pub fn toggle_enabled(app: &AppHandle) -> Option<bool> {
    let state = app.try_state::<SharedConfig>()?;
    let snapshot = {
        let mut cfg = state.lock().ok()?;
        cfg.enabled = !cfg.enabled;
        cfg.clone()
    };
    commit(app, &snapshot);
    Some(snapshot.enabled)
}

fn show_main_window(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

/// A snapshot request parsed from the command line (`--capture <file> [--capture-size WxH]
/// [--capture-burst N] [--capture-interval ms]`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureArgs {
    pub path: PathBuf,
    pub size: Option<(u32, u32)>,
    pub frames: u32,
    pub interval_ms: u32,
}

/// Default spacing between the frames of a `--capture-burst`.
pub const DEFAULT_BURST_INTERVAL_MS: u32 = 100;

pub fn parse_capture_args(args: &[String], cwd: &str) -> Option<CaptureArgs> {
    let mut path: Option<PathBuf> = None;
    let mut size: Option<(u32, u32)> = None;
    let mut frames = 1;
    let mut interval_ms = DEFAULT_BURST_INTERVAL_MS;
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        match arg.as_str() {
            "--capture" => {
                let p = PathBuf::from(it.next()?);
                path = Some(if p.is_absolute() { p } else { PathBuf::from(cwd).join(p) });
            }
            "--capture-size" => {
                let spec = it.next()?;
                let (w, h) = spec.split_once('x')?;
                size = Some((w.parse().ok()?, h.parse().ok()?));
            }
            "--capture-burst" => frames = it.next()?.parse::<u32>().ok()?.clamp(1, 240),
            "--capture-interval" => interval_ms = it.next()?.parse::<u32>().ok()?.clamp(1, 5000),
            _ => {}
        }
    }
    path.map(|path| CaptureArgs {
        path,
        size,
        frames,
        interval_ms,
    })
}

/// `--apply <json>`: a partial configuration (any subset of `AppConfig`) merged into the live
/// one by a second process, e.g. `fxcursor --apply "{\"fps_counter\":{\"enabled\":true}}"`.
/// Scripts and the snapshot tools use it to flip settings without touching the Studio.
pub fn parse_apply_arg(args: &[String]) -> Option<String> {
    let mut it = args.iter();
    while let Some(arg) = it.next() {
        if arg == "--apply" {
            return it.next().cloned();
        }
        if let Some(json) = arg.strip_prefix("--apply=") {
            return Some(json.to_string());
        }
    }
    None
}

/// Deep-merges JSON object `patch` into `base`; non-object values are replaced wholesale.
pub fn merge_json(base: &mut serde_json::Value, patch: &serde_json::Value) {
    match (base, patch) {
        (serde_json::Value::Object(b), serde_json::Value::Object(p)) => {
            for (key, value) in p {
                match b.get_mut(key) {
                    Some(slot) if slot.is_object() && value.is_object() => merge_json(slot, value),
                    _ => {
                        b.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        (b, p) => *b = p.clone(),
    }
}

/// Merges a JSON patch into the live configuration (through the self-healing deserializer, so
/// wrong types are repaired instead of rejected) and commits the result.
pub fn apply_patch(app: &AppHandle, patch_json: &str) -> Result<AppConfig, String> {
    let patch: serde_json::Value =
        serde_json::from_str(patch_json).map_err(|e| format!("invalid JSON patch: {e}"))?;
    if !patch.is_object() {
        return Err("the patch must be a JSON object".to_string());
    }
    let state = app
        .try_state::<SharedConfig>()
        .ok_or("configuration state not initialised")?;
    let current = state.lock().map_err(|e| e.to_string())?.clone();
    let mut value = serde_json::to_value(&current).map_err(|e| e.to_string())?;
    merge_json(&mut value, &patch);
    let outcome: RepairOutcome<AppConfig> = deserialize_with_self_healing(&value.to_string())?;
    if !outcome.repaired_paths.is_empty() {
        log::warn!("[cli] patch repaired: {}", outcome.repaired_paths.join(", "));
    }
    replace_config(app, outcome.value)
}

// ---------------------------------------------------------------------------------------------
// IPC commands (exported to TypeScript via tauri-specta → src/lib/bindings.ts)
//
// Commands that do real work are `async`: Tauri runs synchronous commands on the main thread,
// and the overlay render thread still needs that thread for its (rare) window queries. A slider
// drag sends `update_config` ~60 times a second, so it must not queue behind disk or registry
// I/O there. Plugins that need the main thread (global shortcut, tray) marshal to it themselves.
// ---------------------------------------------------------------------------------------------

#[tauri::command]
#[specta::specta]
fn ping() -> String {
    "pong".to_string()
}

#[tauri::command]
#[specta::specta]
async fn get_diagnostics(app: AppHandle) -> Result<SystemDiagnostics, String> {
    let (vx, vy, vw, vh) = overlay::get_virtual_screen_bounds();
    let config_path = app
        .try_state::<Persistence>()
        .map(|p| p.path.display().to_string())
        .unwrap_or_default();
    let runtime = app.try_state::<RuntimeInfo>();
    let gpu = runtime
        .as_ref()
        .and_then(|r| r.gpu.lock().ok().and_then(|g| g.clone()));
    let uptime_secs = runtime
        .as_ref()
        .map(|r| r.started.elapsed().as_secs_f64())
        .unwrap_or(0.0);
    let frame = runtime
        .as_ref()
        .and_then(|r| r.frame.lock().ok().map(|f| f.clone()))
        .unwrap_or_default();
    let input_backend = app
        .try_state::<Arc<input::InputHub>>()
        .map(|h| if h.hook_active() { "hook" } else { "poll" })
        .unwrap_or("poll")
        .to_string();
    Ok(SystemDiagnostics {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        portable: portable::is_active(),
        app_version: app.package_info().version.to_string(),
        config_path,
        screen_virtual_origin: [vx, vy],
        screen_virtual_size: [vw, vh],
        gpu,
        input_backend,
        uptime_secs,
        debug_build: cfg!(debug_assertions),
        frame,
        hotkey_status: integrations::hotkey_status(),
        display_refresh_hz: display::refresh_rate_hz().unwrap_or(60.0),
    })
}

#[tauri::command]
#[specta::specta]
fn get_config(state: tauri::State<SharedConfig>) -> Result<AppConfig, String> {
    // First call = the Studio webview booted and can reach the backend (also proves the CSP
    // did not block the IPC bridge).
    static FIRST_CALL: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
    if FIRST_CALL.swap(false, std::sync::atomic::Ordering::Relaxed) {
        log::info!("[ipc] studio connected (first get_config)");
    }
    state.lock().map_err(|e| e.to_string()).map(|c| c.clone())
}

#[tauri::command]
#[specta::specta]
async fn update_config(app: AppHandle, new_config: AppConfig) -> Result<(), String> {
    replace_config(&app, new_config).map(|_| ())
}

#[tauri::command]
#[specta::specta]
fn toggle_overlay(app: AppHandle) -> Result<bool, String> {
    toggle_enabled(&app).ok_or_else(|| "configuration state not initialised".to_string())
}

#[tauri::command]
#[specta::specta]
async fn reset_defaults(app: AppHandle) -> Result<AppConfig, String> {
    replace_config(&app, AppConfig::default())
}

/// Writes the current configuration to disk immediately (bypassing the autosave debounce).
#[tauri::command]
#[specta::specta]
async fn save_config(
    state: tauri::State<'_, SharedConfig>,
    persistence: tauri::State<'_, Persistence>,
) -> Result<String, String> {
    let snapshot = state.lock().map_err(|e| e.to_string())?.clone();
    settings_repair::save(&persistence.path, &snapshot)?;
    Ok(persistence.path.display().to_string())
}

/// Built-in presets (Rust is the single source of truth) followed by the user presets.
fn all_presets(app: &AppHandle) -> Vec<PresetInfo> {
    let mut presets = get_builtin_presets();
    presets.extend(user_presets::list(app));
    presets
}

#[tauri::command]
#[specta::specta]
async fn list_presets(app: AppHandle) -> Vec<PresetInfo> {
    all_presets(&app)
}

#[tauri::command]
#[specta::specta]
async fn apply_preset(app: AppHandle, id: String) -> Result<AppConfig, String> {
    let preset = all_presets(&app)
        .into_iter()
        .find(|p| p.id == id)
        .ok_or_else(|| format!("unknown preset '{id}'"))?;
    let mut config = preset.config;
    config.general.selected_preset = id;
    replace_config(&app, config)
}

/// Stores the live configuration as a named user preset (`<config dir>/presets/user-<slug>.json`)
/// and selects it. Saving under an existing name overwrites that preset.
#[tauri::command]
#[specta::specta]
async fn save_user_preset(app: AppHandle, name: String, description: String) -> Result<PresetInfo, String> {
    let state = app
        .try_state::<SharedConfig>()
        .ok_or("configuration state not initialised")?;
    let snapshot = state.lock().map_err(|e| e.to_string())?.clone();
    let preset = user_presets::save(&app, &name, &description, &snapshot)?;
    replace_config(&app, preset.config.clone())?;
    Ok(preset)
}

/// Deletes a user preset and returns the refreshed preset list. Built-ins are refused.
#[tauri::command]
#[specta::specta]
async fn delete_user_preset(app: AppHandle, id: String) -> Result<Vec<PresetInfo>, String> {
    user_presets::delete(&app, &id)?;
    Ok(all_presets(&app))
}

/// Imports a user-supplied JSON document through the self-healing deserializer, so partial or
/// slightly broken files still apply cleanly. Returns the healed config and what was repaired.
#[tauri::command]
#[specta::specta]
async fn import_config(app: AppHandle, json: String) -> Result<ImportOutcome, String> {
    let outcome: RepairOutcome<AppConfig> = deserialize_with_self_healing(&json)?;
    if outcome
        .repaired_paths
        .iter()
        .any(|p| p == "<entire document corrupted>")
    {
        return Err("not valid JSON".to_string());
    }
    let config = replace_config(&app, outcome.value)?;
    Ok(ImportOutcome {
        config,
        repaired_paths: outcome.repaired_paths,
    })
}

/// Backend log history for the Dev Console (newer records arrive via the `rust-log` event).
#[tauri::command]
#[specta::specta]
fn get_recent_logs() -> Vec<logger::LogRecord> {
    logger::recent()
}

/// Queues an overlay snapshot and blocks until the render thread has written the PNG.
/// `size` = crop in overlay pixels centred on the cursor; `None` captures the whole overlay.
pub fn request_capture(app: &AppHandle, args: CaptureArgs) -> Result<String, String> {
    let runtime = app
        .try_state::<RuntimeInfo>()
        .ok_or("runtime state not initialised")?;
    let (reply, rx) = std::sync::mpsc::channel();
    let request = capture::CaptureRequest::new(
        args.path,
        args.size,
        args.frames,
        std::time::Duration::from_millis(u64::from(args.interval_ms)),
        reply,
    );
    let timeout = request.timeout();
    runtime
        .captures
        .lock()
        .map_err(|e| e.to_string())?
        .push(request);
    if let Some(hub) = app.try_state::<Arc<input::InputHub>>() {
        hub.notify();
    }
    rx.recv_timeout(timeout)
        .map_err(|_| "render thread did not answer in time".to_string())?
}

/// Default snapshot location: `<config dir>/captures/overlay-<unix ms>.png`.
fn default_capture_path(app: &AppHandle) -> Result<PathBuf, String> {
    let dir = portable::config_dir(app)
        .map_err(|e| e.to_string())?
        .join("captures");
    let stamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    Ok(dir.join(format!("overlay-{stamp}.png")))
}

/// Renders the current overlay frame to a PNG (composited over dark grey) and returns its path.
/// `width` / `height` = crop size centred on the cursor; 0 = whole overlay.
#[tauri::command]
#[specta::specta]
async fn capture_overlay(app: AppHandle, width: u32, height: u32) -> Result<String, String> {
    // Async so the wait runs off the main thread: the render thread needs the main thread
    // to answer window queries while it fulfils the request.
    let path = default_capture_path(&app)?;
    let size = if width == 0 || height == 0 { None } else { Some((width, height)) };
    let saved = tauri::async_runtime::spawn_blocking(move || {
        request_capture(
            &app,
            CaptureArgs {
                path,
                size,
                frames: 1,
                interval_ms: 0,
            },
        )
    })
        .await
        .map_err(|e| e.to_string())??;
    log::info!("[capture] overlay snapshot written to {saved}");
    Ok(saved)
}

// ---------------------------------------------------------------------------------------------
// Application entry
// ---------------------------------------------------------------------------------------------

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    panic_log::install();
    portable::init();
    logger::install();

    let config: SharedConfig = Arc::new(Mutex::new(AppConfig::default()));
    let start_hidden_flag = std::env::args().any(|a| a == "--minimized");

    let specta_builder = tauri_specta::Builder::<tauri::Wry>::new()
        .error_handling(ErrorHandlingMode::Throw)
        // `AppConfig::sanitize` replaces NaN/∞ before any config is stored, so export f32 as a
        // plain `number` instead of `number | null`.
        .semantic_types(
            specta_typescript::semantic::Configuration::default().enable_lossless_floats(),
        )
        .commands(collect_commands![
            ping,
            get_diagnostics,
            get_config,
            update_config,
            toggle_overlay,
            reset_defaults,
            save_config,
            list_presets,
            apply_preset,
            import_config,
            get_recent_logs,
            capture_overlay,
            save_user_preset,
            delete_user_preset,
        ]);

    #[cfg(debug_assertions)]
    {
        let header = "// AUTO-GENERATED by tauri-specta from src-tauri/src/lib.rs. Do not edit; run `bun run tauri dev` to refresh.\n";
        if let Err(err) = specta_builder.export(
            specta_typescript::Typescript::default().header(header),
            "../src/lib/bindings.ts",
        ) {
            log::warn!("[specta] failed to export TypeScript bindings: {err}");
        }
    }

    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {
            // `fxcursor --capture out.png [--capture-size WxH]` from a second process
            // asks the running instance for an overlay snapshot instead of showing the window.
            let mut handled = false;
            if let Some(patch) = parse_apply_arg(&args) {
                match apply_patch(app, &patch) {
                    Ok(_) => log::info!("[cli] applied configuration patch {patch}"),
                    Err(err) => log::warn!("[cli] --apply rejected: {err}"),
                }
                handled = true;
            }
            if let Some(request) = parse_capture_args(&args, &cwd) {
                let app = app.clone();
                std::thread::spawn(move || match request_capture(&app, request) {
                    Ok(saved) => log::info!("[capture] CLI snapshot written to {saved}"),
                    Err(err) => log::warn!("[capture] CLI snapshot failed: {err}"),
                });
                handled = true;
            }
            if handled {
                return;
            }
            show_main_window(app);
        }))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--minimized"]),
        ))
        .manage(config.clone())
        .manage(RuntimeInfo {
            started: Instant::now(),
            gpu: Mutex::new(None),
            frame: Mutex::new(FrameStats::default()),
            captures: Mutex::new(Vec::new()),
        })
        .invoke_handler(specta_builder.invoke_handler())
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let minimize_to_tray = window
                    .app_handle()
                    .try_state::<SharedConfig>()
                    .and_then(|s| s.lock().ok().map(|c| c.general.minimize_to_tray))
                    .unwrap_or(true);
                if minimize_to_tray {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    window.app_handle().exit(0);
                }
            }
        })
        .setup(move |app| {
            let handle = app.handle().clone();
            logger::attach(&handle);
            log::info!("FXCursor Studio {} starting", app.package_info().version);

            // 1. Load persisted configuration (self-healing) and start the autosaver.
            let config_path = settings_repair::config_path(&handle)?;
            let loaded = settings_repair::load_or_repair(&config_path);
            if let Ok(mut c) = config.lock() {
                *c = loaded.clone();
            }
            app.manage(Persistence {
                path: config_path.clone(),
                saver: AutoSaver::spawn(config_path),
            });

            // 2. Start-minimized behaviour (webview hardening lives in `src/lib/hardening.ts`).
            if (loaded.general.start_minimized || start_hidden_flag)
                && let Some(main_win) = app.get_webview_window("main")
            {
                let _ = main_win.hide();
            }

            // 3. System tray.
            let show_item = MenuItem::with_id(app, "show", "Show Settings", true, None::<&str>)?;
            let toggle_item =
                MenuItem::with_id(app, "toggle", "Toggle Effects", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let quit_item = MenuItem::with_id(app, "quit", "Quit FXCursor", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&show_item, &toggle_item, &separator, &quit_item])?;

            let mut tray_builder = TrayIconBuilder::with_id(TRAY_ID)
                .menu(&menu)
                .tooltip(tray_tooltip(&loaded))
                .show_menu_on_left_click(false);
            if let Some(icon) = app.default_window_icon() {
                tray_builder = tray_builder.icon(icon.clone());
            }
            let _tray = tray_builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main_window(app),
                    "toggle" => {
                        toggle_enabled(app);
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(w) = app.get_webview_window("main") {
                            if w.is_visible().unwrap_or(false) {
                                let _ = w.hide();
                            } else {
                                show_main_window(app);
                            }
                        }
                    }
                })
                .build(app)?;

            // 4. OS integrations from the loaded config (hotkey + autostart).
            integrations::sync(&handle, &loaded);

            // 5. Transparent overlay spanning the virtual desktop. The builder takes LOGICAL
            //    units, so divide by the primary scale factor here; the render thread then pins
            //    the exact physical rectangle with `set_size` / `set_position`.
            let (vx, vy, vw, vh) = display::virtual_bounds(&handle);
            let scale = display::primary_scale_factor(&handle);
            log::info!(
                "Initializing transparent overlay across virtual bounds: ({vx}, {vy}) {vw}x{vh} physical (scale {scale})"
            );

            WebviewWindowBuilder::new(app, "overlay", WebviewUrl::App("overlay.html".into()))
                .title("FXCursor Transparent Overlay")
                .inner_size(vw as f64 / scale, vh as f64 / scale)
                .position(vx as f64 / scale, vy as f64 / scale)
                .transparent(true)
                .decorations(false)
                .always_on_top(true)
                .skip_taskbar(true)
                .shadow(false)
                .resizable(false)
                .focused(false)
                .build()?;

            // 6. Input hub (low-level mouse hook on Windows) shared with the render thread and
            //    woken by `commit()` so config changes are picked up while idle.
            let input_hub = input::InputHub::start();
            app.manage(input_hub.clone());

            let overlay_config = config.clone();
            std::thread::Builder::new()
                .name("gpu-render-thread".into())
                .spawn(move || {
                    display::raise_render_thread_priority();
                    if let Err(e) = OverlayState::run_render_loop(handle, overlay_config, input_hub) {
                        log::error!("Overlay render loop error: {e}");
                    }
                })?;

            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("error while building fxcursor studio")
        .run(|handle, event| {
            if matches!(
                event,
                tauri::RunEvent::ExitRequested { .. } | tauri::RunEvent::Exit
            ) {
                // `SetSystemCursor` replacements survive process death: put the arrow back
                // and latch it (the render thread may still run a frame before the process
                // ends and must not hide it again). Nothing calls `prevent_exit`, so an exit
                // request always ends the process. The panic hook covers crashes.
                crate::cursor::force_restore();
                // The autosave debounce would otherwise drop the last ~400 ms of edits.
                if let Some(p) = handle.try_state::<Persistence>() {
                    p.saver.flush(std::time::Duration::from_millis(500));
                }
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn json_patches_merge_deeply_and_replace_leaves() {
        let mut base = json!({"a": {"x": 1, "y": 2}, "b": [1, 2], "c": "keep"});
        merge_json(&mut base, &json!({"a": {"y": 20, "z": 30}, "b": [9], "d": true}));
        assert_eq!(base, json!({"a": {"x": 1, "y": 20, "z": 30}, "b": [9], "c": "keep", "d": true}));
    }

    #[test]
    fn cli_flags_are_parsed() {
        let args: Vec<String> = ["fxcursor", "--apply", r#"{"enabled":false}"#, "--capture", "shot.png", "--capture-size", "640x360", "--capture-burst", "12", "--capture-interval", "40"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        assert_eq!(parse_apply_arg(&args).as_deref(), Some(r#"{"enabled":false}"#));
        let capture = parse_capture_args(&args, "C:/work").expect("capture request");
        assert!(capture.path.ends_with("shot.png"));
        assert_eq!(capture.size, Some((640, 360)));
        assert_eq!((capture.frames, capture.interval_ms), (12, 40));
        let plain: Vec<String> = vec!["fxcursor".into(), "--minimized".into()];
        assert!(parse_apply_arg(&plain).is_none());
        assert!(parse_capture_args(&plain, ".").is_none());
        let eq_form: Vec<String> = vec!["fxcursor".into(), "--apply={\"x\":1}".into()];
        assert_eq!(parse_apply_arg(&eq_form).as_deref(), Some("{\"x\":1}"));
    }
}
