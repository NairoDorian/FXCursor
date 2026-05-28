use device_query::{DeviceState, DeviceQuery, MouseState};

pub struct MouseTracker {
    device_state: DeviceState,
    last_position: (f32, f32),
    buttons: Vec<bool>,
}

impl MouseTracker {
    pub fn new() -> Self {
        Self {
            device_state: DeviceState::new(),
            last_position: (0.0, 0.0),
            buttons: vec![false; 5],
        }
    }

    /// Queries the current global mouse coordinates and button states.
    /// Returns: (x, y, buttons_pressed) where coordinates are in screen pixels.
    pub fn update(&mut self) -> (f32, f32, &[bool]) {
        let state: MouseState = self.device_state.get_mouse();
        
        self.last_position = (state.coords.0 as f32, state.coords.1 as f32);
        
        // device_query buttons are 0-indexed. Let's copy the state safely.
        self.buttons.clear();
        for &pressed in &state.button_pressed {
            self.buttons.push(pressed);
        }
        // Pad to at least 5 buttons for shader uniforms safety
        while self.buttons.len() < 5 {
            self.buttons.push(false);
        }

        (self.last_position.0, self.last_position.1, &self.buttons)
    }

    #[allow(dead_code)]
    pub fn last_position(&self) -> (f32, f32) {
        self.last_position
    }

}
