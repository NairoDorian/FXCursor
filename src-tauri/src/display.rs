//! Display facts: virtual-desktop bounds, refresh rate and timer resolution.
//!
//! Windows uses the Win32 metrics directly; other platforms derive the virtual desktop from the
//! monitor list Tauri exposes.

use tauri::AppHandle;

/// Virtual desktop `(x, y, width, height)` in physical pixels. The origin can be negative when
/// a monitor sits left of or above the primary one.
pub fn virtual_bounds(app: &AppHandle) -> (i32, i32, u32, u32) {
    #[cfg(target_os = "windows")]
    {
        let _ = app;
        get_virtual_screen_bounds()
    }
    #[cfg(not(target_os = "windows"))]
    {
        bounds_from_monitors(app).unwrap_or_else(get_virtual_screen_bounds)
    }
}

/// Win32 virtual-screen metrics (Windows) or a conservative fallback elsewhere.
pub fn get_virtual_screen_bounds() -> (i32, i32, u32, u32) {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN,
        };
        let vx = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let vy = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let vw = GetSystemMetrics(SM_CXVIRTUALSCREEN) as u32;
        let vh = GetSystemMetrics(SM_CYVIRTUALSCREEN) as u32;
        (vx, vy, vw.max(800), vh.max(600))
    }
    #[cfg(not(target_os = "windows"))]
    {
        (0, 0, 1920, 1080)
    }
}

/// Union of all monitor rectangles reported by Tauri (physical pixels).
#[allow(dead_code)]
pub fn bounds_from_monitors(app: &AppHandle) -> Option<(i32, i32, u32, u32)> {
    let monitors = app.available_monitors().ok()?;
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;
    for m in &monitors {
        let p = m.position();
        let s = m.size();
        min_x = min_x.min(p.x);
        min_y = min_y.min(p.y);
        max_x = max_x.max(p.x + s.width as i32);
        max_y = max_y.max(p.y + s.height as i32);
    }
    if monitors.is_empty() || max_x <= min_x || max_y <= min_y {
        return None;
    }
    Some((min_x, min_y, (max_x - min_x) as u32, (max_y - min_y) as u32))
}

/// Scale factor of the primary monitor (1.0 when unknown). Used to translate physical sizes into
/// the logical units Tauri's window builder expects.
pub fn primary_scale_factor(app: &AppHandle) -> f64 {
    app.primary_monitor()
        .ok()
        .flatten()
        .map(|m| m.scale_factor())
        .filter(|s| *s > 0.0)
        .unwrap_or(1.0)
}

/// Rectangle of the primary monitor in physical pixels `(x, y, width, height)`: the HUD is
/// anchored to it because a corner of the whole virtual desktop can lie outside every screen.
/// Falls back to the virtual bounds on other platforms.
pub fn primary_rect() -> (f32, f32, f32, f32) {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::UI::WindowsAndMessaging::{GetSystemMetrics, SM_CXSCREEN, SM_CYSCREEN};
        // SAFETY: GetSystemMetrics has no preconditions.
        let (w, h) = unsafe { (GetSystemMetrics(SM_CXSCREEN), GetSystemMetrics(SM_CYSCREEN)) };
        if w > 0 && h > 0 {
            return (0.0, 0.0, w as f32, h as f32);
        }
    }
    let (x, y, w, h) = get_virtual_screen_bounds();
    (x as f32, y as f32, w as f32, h as f32)
}

/// Current refresh rate of the primary display in Hz, when the OS reports one.
pub fn refresh_rate_hz() -> Option<f32> {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::Graphics::Gdi::{EnumDisplaySettingsW, DEVMODEW, ENUM_CURRENT_SETTINGS};
        let mut mode: DEVMODEW = std::mem::zeroed();
        mode.dmSize = std::mem::size_of::<DEVMODEW>() as u16;
        if EnumDisplaySettingsW(std::ptr::null(), ENUM_CURRENT_SETTINGS, &mut mode) != 0 {
            let hz = mode.dmDisplayFrequency;
            // 0 and 1 mean "hardware default" per the Win32 docs.
            if hz > 1 {
                return Some(hz as f32);
            }
        }
        None
    }
    #[cfg(not(target_os = "windows"))]
    {
        None
    }
}

/// Asks the OS for 1 ms timer granularity so frame-pacing sleeps are accurate. No-op elsewhere.
pub fn raise_timer_resolution() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::Media::timeBeginPeriod;
        let _ = timeBeginPeriod(1);
    }
}

/// Restores the default OS timer resolution previously requested with `raise_timer_resolution`.
pub fn restore_timer_resolution() {
    #[cfg(target_os = "windows")]
    unsafe {
        use windows_sys::Win32::Media::timeEndPeriod;
        let _ = timeEndPeriod(1);
    }
}

/// Raises the calling thread (the render loop) to above-normal priority so frame pacing survives
/// a busy desktop. Best effort; nothing to do on other platforms yet.
pub fn raise_render_thread_priority() {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::System::Threading::{
            GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_ABOVE_NORMAL,
        };
        // SAFETY: plain Win32 calls on the current thread with a valid pseudo-handle.
        let ok = unsafe { SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_ABOVE_NORMAL) };
        if ok == 0 {
            log::warn!("[display] could not raise the render thread priority");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn virtual_screen_bounds_are_sane() {
        let (_, _, w, h) = get_virtual_screen_bounds();
        assert!(w >= 800 && h >= 600);
    }

    #[test]
    fn refresh_rate_is_plausible_when_reported() {
        if let Some(hz) = refresh_rate_hz() {
            assert!((24.0..=1000.0).contains(&hz), "reported {hz} Hz");
        }
    }
}
