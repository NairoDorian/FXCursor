//! Input hub: a single shared, event-driven source of cursor position, button state and
//! discrete click events for the render thread.
//!
//! * Windows: a `WH_MOUSE_LL` low-level hook on a dedicated message thread pushes every move and
//!   button transition into the hub (see [`windows`]). Clicks shorter than a frame are therefore
//!   never lost. A `GetCursorPos` poll remains as a fallback for the UIPI case (hooks are muted
//!   while an elevated window has focus).
//! * Other platforms: `device_query` polling with edge detection done here.
//!
//! The render loop calls [`InputHub::wait_for_activity`] while idle, so the thread parks in the
//! kernel instead of spinning.

use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

#[cfg(target_os = "windows")]
pub mod windows;

/// Number of tracked buttons: left, right, middle, x1, x2.
pub const BUTTON_COUNT: usize = 5;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ClickEvent {
    /// 0 = left, 1 = right, 2 = middle, 3 = x1, 4 = x2
    pub button: usize,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Default)]
struct Inner {
    x: f32,
    y: f32,
    buttons: [bool; BUTTON_COUNT],
    clicks: Vec<ClickEvent>,
    /// Incremented on every change so waiters can detect activity.
    generation: u64,
    /// True once the OS hook is delivering events (Windows only).
    hook_active: bool,
}

/// Snapshot handed to the renderer once per frame.
#[derive(Debug, Clone)]
pub struct InputFrame {
    pub x: f32,
    pub y: f32,
    pub buttons: [bool; BUTTON_COUNT],
    pub clicks: Vec<ClickEvent>,
}

pub struct InputHub {
    inner: Mutex<Inner>,
    cv: Condvar,
}

impl Default for InputHub {
    fn default() -> Self {
        Self::new()
    }
}

impl InputHub {
    pub fn new() -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            cv: Condvar::new(),
        }
    }

    /// Creates the hub and starts the platform event source.
    pub fn start() -> Arc<Self> {
        let hub = Arc::new(Self::new());
        #[cfg(target_os = "windows")]
        windows::spawn_hook_thread(hub.clone());
        hub
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Feed an absolute cursor position (from a hook or a poll).
    pub fn push_move(&self, x: f32, y: f32) {
        let mut g = self.lock();
        if (g.x, g.y) != (x, y) {
            g.x = x;
            g.y = y;
            g.generation += 1;
            drop(g);
            self.cv.notify_all();
        }
    }

    /// Feed a button transition. A press is also recorded as a discrete click event.
    pub fn push_button(&self, button: usize, pressed: bool, x: f32, y: f32) {
        if button >= BUTTON_COUNT {
            return;
        }
        let mut g = self.lock();
        g.x = x;
        g.y = y;
        if g.buttons[button] != pressed {
            g.buttons[button] = pressed;
            if pressed {
                g.clicks.push(ClickEvent { button, x, y });
            }
        }
        g.generation += 1;
        drop(g);
        self.cv.notify_all();
    }

    /// Feed a full polled button state; presses that appear since the last call become clicks.
    pub fn push_polled_buttons(&self, buttons: &[bool]) {
        let mut g = self.lock();
        let (x, y) = (g.x, g.y);
        let mut changed = false;
        for (i, &pressed) in buttons.iter().take(BUTTON_COUNT).enumerate() {
            if g.buttons[i] != pressed {
                g.buttons[i] = pressed;
                if pressed {
                    g.clicks.push(ClickEvent { button: i, x, y });
                }
                changed = true;
            }
        }
        if changed {
            g.generation += 1;
            drop(g);
            self.cv.notify_all();
        }
    }

    pub fn set_hook_active(&self, active: bool) {
        let mut g = self.lock();
        g.hook_active = active;
    }

    pub fn hook_active(&self) -> bool {
        self.lock().hook_active
    }

    /// Wake any waiter without changing input (used when the configuration changes).
    pub fn notify(&self) {
        {
            let mut g = self.lock();
            g.generation += 1;
        }
        self.cv.notify_all();
    }

    pub fn generation(&self) -> u64 {
        self.lock().generation
    }

    /// Take the current state and drain queued clicks.
    pub fn take_frame(&self) -> InputFrame {
        let mut g = self.lock();
        InputFrame {
            x: g.x,
            y: g.y,
            buttons: g.buttons,
            clicks: std::mem::take(&mut g.clicks),
        }
    }

    /// Block until the generation advances past `seen` or `timeout` elapses. Returns the new
    /// generation.
    pub fn wait_for_activity(&self, seen: u64, timeout: Duration) -> u64 {
        let g = self.lock();
        let (g, _) = self
            .cv
            .wait_timeout_while(g, timeout, |s| s.generation == seen)
            .unwrap_or_else(|p| p.into_inner());
        g.generation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn press_records_click_once() {
        let hub = InputHub::new();
        hub.push_button(0, true, 10.0, 20.0);
        hub.push_button(0, true, 10.0, 20.0); // duplicate press: no second click
        hub.push_button(0, false, 11.0, 21.0);
        let frame = hub.take_frame();
        assert_eq!(frame.clicks, vec![ClickEvent { button: 0, x: 10.0, y: 20.0 }]);
        assert!(!frame.buttons[0]);
        assert!(hub.take_frame().clicks.is_empty(), "clicks are drained");
    }

    #[test]
    fn polled_buttons_detect_edges() {
        let hub = InputHub::new();
        hub.push_move(5.0, 5.0);
        hub.push_polled_buttons(&[false, true, false]);
        hub.push_polled_buttons(&[false, true, false]);
        hub.push_polled_buttons(&[false, false, false]);
        let frame = hub.take_frame();
        assert_eq!(frame.clicks.len(), 1);
        assert_eq!(frame.clicks[0].button, 1);
        assert_eq!((frame.clicks[0].x, frame.clicks[0].y), (5.0, 5.0));
    }

    #[test]
    fn wait_returns_on_timeout_and_on_activity() {
        let hub = Arc::new(InputHub::new());
        let gen0 = hub.generation();
        assert_eq!(hub.wait_for_activity(gen0, Duration::from_millis(10)), gen0);

        let h2 = hub.clone();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(20));
            h2.push_move(1.0, 1.0);
        });
        let gen1 = hub.wait_for_activity(gen0, Duration::from_secs(2));
        assert!(gen1 > gen0);
    }
}
