#[cfg(windows)]
pub mod win32 {
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;
    use winit::window::Window;

    pub fn apply_click_through_styles(window: &dyn Window) {
        if let Ok(handle) = window.window_handle()
            && let RawWindowHandle::Win32(win32_handle) = handle.as_raw()
        {
            let hwnd = win32_handle.hwnd.get() as HWND;
            unsafe {
                let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE);
                let new_style =
                    ex_style as u32 | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_NOREDIRECTIONBITMAP;
                SetWindowLongW(hwnd, GWL_EXSTYLE, new_style as i32);

                // Force top-most positioning without activating
                SetWindowPos(
                    hwnd,
                    HWND_TOPMOST,
                    0,
                    0,
                    0,
                    0,
                    SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_FRAMECHANGED,
                );
            }
        }
    }

    pub fn get_virtual_screen_geometry() -> ((i32, i32), (u32, u32)) {
        unsafe {
            let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let width = GetSystemMetrics(SM_CXVIRTUALSCREEN).max(800) as u32;
            let height = GetSystemMetrics(SM_CYVIRTUALSCREEN).max(600) as u32;
            ((x, y), (width, height))
        }
    }
}
