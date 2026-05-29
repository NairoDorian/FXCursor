use device_query::{DeviceQuery, DeviceState, MouseState};

pub struct MouseTracker {
    device_state: DeviceState,
    last_position: (f32, f32),
    buttons: Vec<bool>,
}

impl Default for MouseTracker {
    fn default() -> Self {
        Self {
            device_state: DeviceState::new(),
            last_position: (0.0, 0.0),
            buttons: Vec::with_capacity(5),
        }
    }
}

impl MouseTracker {
    pub fn new() -> Self {
        Self::default()
    }

    /// Queries the current global mouse coordinates and button states.
    /// Returns: `(x, y, buttons_pressed)` where coordinates are in screen pixels.
    pub fn update(&mut self) -> (f32, f32, &[bool]) {
        let state: MouseState = self.device_state.get_mouse();

        self.last_position = (state.coords.0 as f32, state.coords.1 as f32);

        // Rebuild button state from hardware query.
        // device_query uses 0-indexed booleans; we pad to at least 5 for uniform safety.
        self.buttons.clear();
        self.buttons.extend(state.button_pressed.iter().copied());
        while self.buttons.len() < 5 {
            self.buttons.push(false);
        }

        (self.last_position.0, self.last_position.1, &self.buttons)
    }
}
