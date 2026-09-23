//! OS integrations driven by `GeneralConfig`: the global toggle hotkey and login autostart.
//!
//! Both are idempotent "sync to desired state" operations so they can be called after every
//! configuration change without re-registering when nothing relevant changed.

use fxcursor_protocol::AppConfig;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_global_shortcut::{GlobalShortcutExt as _, Shortcut, ShortcutState};

/// The hotkey string currently registered with the OS (`None` = nothing registered).
static REGISTERED_HOTKEY: Mutex<Option<String>> = Mutex::new(None);
/// Human-readable outcome of the last registration attempt, surfaced in diagnostics.
static HOTKEY_STATUS: Mutex<String> = Mutex::new(String::new());

fn set_status(status: String) {
    if let Ok(mut s) = HOTKEY_STATUS.lock() {
        *s = status;
    }
}

/// Last hotkey registration outcome ("registered 'Ctrl+Shift+E'", "disabled", or an error).
pub fn hotkey_status() -> String {
    HOTKEY_STATUS.lock().map(|s| s.clone()).unwrap_or_default()
}

/// Brings the global hotkey and autostart entry in line with `config.general`.
pub fn sync(app: &AppHandle, config: &AppConfig) {
    if let Err(err) = sync_hotkey(app, &config.general.global_hotkey) {
        log::warn!("[hotkey] {err}");
        set_status(format!("error: {err}"));
    }
    sync_autostart(app, config.general.autostart);
}

/// Registers `desired` as the system-wide toggle shortcut, replacing any previous binding.
/// An empty string unregisters. Returns a human-readable error when the string cannot be parsed
/// or the OS refuses the binding (for example when another app already owns it).
pub fn sync_hotkey(app: &AppHandle, desired: &str) -> Result<(), String> {
    let desired = desired.trim().to_string();
    let mut registered = REGISTERED_HOTKEY.lock().unwrap_or_else(|p| p.into_inner());
    if registered.as_deref() == Some(desired.as_str()) {
        return Ok(());
    }

    let gs = app.global_shortcut();
    if let Err(err) = gs.unregister_all() {
        log::warn!("[hotkey] unregister_all failed: {err}");
    }
    *registered = None;

    if desired.is_empty() {
        log::info!("[hotkey] global toggle hotkey disabled");
        set_status("disabled (no shortcut set)".to_string());
        return Ok(());
    }

    let shortcut: Shortcut = desired
        .parse()
        .map_err(|e| format!("cannot parse hotkey '{desired}': {e}"))?;

    gs.on_shortcut(shortcut, |app, _shortcut, event| {
        if event.state == ShortcutState::Pressed
            && let Some(enabled) = crate::toggle_enabled(app)
        {
            log::info!("[hotkey] effects toggled via global shortcut -> enabled={enabled}");
        }
    })
    .map_err(|e| format!("cannot register hotkey '{desired}': {e}"))?;

    log::info!("[hotkey] registered global toggle hotkey '{desired}'");
    set_status(format!("registered '{desired}'"));
    *registered = Some(desired);
    Ok(())
}

/// Last autostart state this process confirmed or set; `None` until the first sync.
static AUTOSTART_SYNCED: Mutex<Option<bool>> = Mutex::new(None);

/// Enables or disables the OS login item so it matches `wanted`.
///
/// `commit()` runs on every configuration change (every slider tick), so the OS is only
/// queried when `wanted` differs from the last state this process confirmed.
pub fn sync_autostart(app: &AppHandle, wanted: bool) {
    let mut synced = AUTOSTART_SYNCED.lock().unwrap_or_else(|p| p.into_inner());
    if *synced == Some(wanted) {
        return;
    }
    let launcher = app.autolaunch();
    let current = launcher.is_enabled().unwrap_or(false);
    if current == wanted {
        *synced = Some(wanted);
        return;
    }
    let result = if wanted {
        launcher.enable()
    } else {
        launcher.disable()
    };
    match result {
        Ok(()) => {
            log::info!("[autostart] login autostart set to {wanted}");
            *synced = Some(wanted);
        }
        Err(err) => log::warn!("[autostart] could not set autostart={wanted}: {err}"),
    }
}
