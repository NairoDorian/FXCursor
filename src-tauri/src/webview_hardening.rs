//! Security hardening for the Tauri webview.

use tauri::WebviewWindow;

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
