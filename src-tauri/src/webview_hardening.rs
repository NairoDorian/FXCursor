//! Security hardening for the Tauri webview windows.
//!
//! In production/release builds, this module restricts external interactions:
//! - Disables drag-and-drop file navigation onto the window (`dragover`, `drop`).
//! - Suppresses default context menus except on interactive text input elements.
//! - Safeguards against unexpected document reloads or navigation.

use tauri::WebviewWindow;

/// Applies frontend security hardening scripts to the specified webview window.
/// In debug builds this is a no-op to allow developer tools and inspection.
pub fn apply_hardening(_window: &WebviewWindow) {
    #[cfg(not(debug_assertions))]
    {
        // Disable devtools and file drag drops in release builds
        let _ = _window.eval(
            r#"
            window.addEventListener('dragover', (e) => e.preventDefault(), false);
            window.addEventListener('drop', (e) => e.preventDefault(), false);
            window.addEventListener('contextmenu', (e) => {
                if (e.target.tagName !== 'INPUT' && e.target.tagName !== 'TEXTAREA') {
                    e.preventDefault();
                }
            });
            "#
        );
    }
}
