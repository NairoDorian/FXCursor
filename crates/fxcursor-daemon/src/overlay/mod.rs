pub mod windows;

use std::sync::Arc;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowAttributes, WindowLevel};

pub fn create_overlay_window(event_loop: &ActiveEventLoop) -> Arc<Window> {
    #[cfg(windows)]
    let ((origin_x, origin_y), (width, height)) = windows::win32::get_virtual_screen_geometry();
    #[cfg(not(windows))]
    let ((origin_x, origin_y), (width, height)) = ((0, 0), (1920, 1080));

    let attributes = WindowAttributes::default()
        .with_title("FXCursor Overlay")
        .with_decorations(false)
        .with_transparent(true)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_active(false)
        .with_position(PhysicalPosition::new(origin_x, origin_y))
        .with_inner_size(PhysicalSize::new(width, height))
        .with_resizable(false);

    let window = Arc::new(event_loop.create_window(attributes).expect("Failed to create overlay window"));

    #[cfg(windows)]
    windows::win32::apply_click_through_styles(&window);

    window
}
