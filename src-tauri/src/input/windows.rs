//! Windows event source: `WH_MOUSE_LL` low-level mouse hook on a dedicated message-loop thread,
//! plus a `GetCursorPos` fallback poll used by the render loop.
//!
//! Low-level hooks are muted while an elevated (UAC) window has the foreground; the fallback poll
//! keeps the trail following the cursor in that case, at the cost of edge-only click detection.

use super::InputHub;
use std::sync::{Arc, OnceLock};
use windows_sys::Win32::Foundation::{LPARAM, LRESULT, POINT, WPARAM};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CallNextHookEx, DispatchMessageW, GetCursorPos, GetMessageW, SetWindowsHookExW,
    TranslateMessage, HC_ACTION, MSG, MSLLHOOKSTRUCT, WH_MOUSE_LL, WM_LBUTTONDOWN, WM_LBUTTONUP,
    WM_MBUTTONDOWN, WM_MBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP, WM_XBUTTONDOWN,
    WM_XBUTTONUP,
};

static HUB: OnceLock<Arc<InputHub>> = OnceLock::new();

/// Installs the hook on its own thread. Idempotent: a second call is ignored.
pub fn spawn_hook_thread(hub: Arc<InputHub>) {
    if HUB.set(hub.clone()).is_err() {
        return;
    }
    let result = std::thread::Builder::new()
        .name("mouse-hook-thread".into())
        .spawn(move || unsafe {
            let hinstance = GetModuleHandleW(std::ptr::null());
            let hook = SetWindowsHookExW(WH_MOUSE_LL, Some(mouse_hook_proc), hinstance, 0);
            if hook.is_null() {
                log::warn!("[input] SetWindowsHookEx(WH_MOUSE_LL) failed; falling back to polling");
                return;
            }
            hub.set_hook_active(true);
            log::info!("[input] low-level mouse hook installed");

            let mut msg: MSG = std::mem::zeroed();
            while GetMessageW(&mut msg, std::ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
            hub.set_hook_active(false);
        });
    if let Err(err) = result {
        log::warn!("[input] could not spawn mouse-hook-thread: {err}");
    }
}

unsafe extern "system" fn mouse_hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    if code == HC_ACTION as i32 {
        if let Some(hub) = HUB.get() {
            let info = &*(lparam as *const MSLLHOOKSTRUCT);
            let (x, y) = (info.pt.x as f32, info.pt.y as f32);
            match wparam as u32 {
                WM_MOUSEMOVE => hub.push_move(x, y),
                WM_LBUTTONDOWN => hub.push_button(0, true, x, y),
                WM_LBUTTONUP => hub.push_button(0, false, x, y),
                WM_RBUTTONDOWN => hub.push_button(1, true, x, y),
                WM_RBUTTONUP => hub.push_button(1, false, x, y),
                WM_MBUTTONDOWN => hub.push_button(2, true, x, y),
                WM_MBUTTONUP => hub.push_button(2, false, x, y),
                WM_XBUTTONDOWN | WM_XBUTTONUP => {
                    // High word of mouseData: 1 = XBUTTON1, 2 = XBUTTON2.
                    let which = (info.mouseData >> 16) & 0xFFFF;
                    let button = if which == 2 { 4 } else { 3 };
                    hub.push_button(button, wparam as u32 == WM_XBUTTONDOWN, x, y);
                }
                _ => {}
            }
        }
    }
    CallNextHookEx(std::ptr::null_mut(), code, wparam, lparam)
}

/// Fallback poll of the cursor position (works even when hooks are muted by UIPI).
pub fn poll_cursor(hub: &InputHub) {
    unsafe {
        let mut pt = POINT { x: 0, y: 0 };
        if GetCursorPos(&mut pt) != 0 {
            hub.push_move(pt.x as f32, pt.y as f32);
        }
    }
}
