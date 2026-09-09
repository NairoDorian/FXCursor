#[cfg(windows)]
pub mod win32 {
    use windows_sys::Win32::Foundation::*;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::*;
    use windows_sys::Win32::UI::WindowsAndMessaging::*;

    pub struct Win32InputTracker {
        pub last_x: f32,
        pub last_y: f32,
    }

    impl Win32InputTracker {
        pub fn new() -> Self {
            let mut pt = POINT { x: 0, y: 0 };
            unsafe {
                GetCursorPos(&mut pt);
            }
            Self {
                last_x: pt.x as f32,
                last_y: pt.y as f32,
            }
        }
    }

    impl Default for Win32InputTracker {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Win32InputTracker {
        pub fn poll_cursor(&mut self) -> (f32, f32) {
            let mut pt = POINT { x: 0, y: 0 };
            unsafe {
                GetCursorPos(&mut pt);
            }
            self.last_x = pt.x as f32;
            self.last_y = pt.y as f32;
            (self.last_x, self.last_y)
        }

        pub fn poll_buttons(&self) -> (bool, bool, bool) {
            unsafe {
                let left = (GetAsyncKeyState(VK_LBUTTON as i32) as u16 & 0x8000) != 0;
                let right = (GetAsyncKeyState(VK_RBUTTON as i32) as u16 & 0x8000) != 0;
                let middle = (GetAsyncKeyState(VK_MBUTTON as i32) as u16 & 0x8000) != 0;
                (left, right, middle)
            }
        }
    }
}
