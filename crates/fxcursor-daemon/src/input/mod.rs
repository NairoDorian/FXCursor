//! Input tracking: polls pointer position and mouse button state.

#[cfg(windows)]
pub mod windows;

/// Tracks global cursor position and mouse button states.
pub struct InputTracker {
    #[cfg(windows)]
    tracker: windows::win32::Win32InputTracker,
    #[cfg(not(windows))]
    last_pos: (f32, f32),
}

impl InputTracker {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            tracker: windows::win32::Win32InputTracker::new(),
            #[cfg(not(windows))]
            last_pos: (0.0, 0.0),
        }
    }

    pub fn poll(&mut self) -> ((f32, f32), (bool, bool, bool)) {
        #[cfg(windows)]
        {
            let pos = self.tracker.poll_cursor();
            let buttons = self.tracker.poll_buttons();
            (pos, buttons)
        }
        #[cfg(not(windows))]
        {
            (self.last_pos, (false, false, false))
        }
    }
}

impl Default for InputTracker {
    fn default() -> Self {
        Self::new()
    }
}
