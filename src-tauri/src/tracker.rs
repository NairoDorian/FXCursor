//! `device_query` polling for platforms without an OS hook (macOS, Linux).

use device_query::{DeviceQuery, DeviceState, MouseState};

/// Hub button order: left, right, middle, x1, x2 (see `input::BUTTON_COUNT`).
const HUB_BUTTONS: usize = 5;

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
            buttons: vec![false; HUB_BUTTONS],
        }
    }
}

/// Maps `device_query`'s 1-based `button_pressed` (index 0 is always false) to the hub order.
///
/// * X11 (Linux): 1 = left, 2 = middle, 3 = right; 4/5 are the scroll wheel, not side buttons.
/// * macOS: 1 = left, 2 = right, 3 = middle.
/// * Windows (unused here, the hook covers it): 1 = left, 2 = right, 3 = middle, 4/5 = X1/X2.
fn map_buttons(pressed: &[bool], out: &mut [bool]) {
    let at = |i: usize| pressed.get(i).copied().unwrap_or(false);
    out.fill(false);
    if cfg!(target_os = "linux") {
        out[0] = at(1);
        out[1] = at(3);
        out[2] = at(2);
    } else {
        out[0] = at(1);
        out[1] = at(2);
        out[2] = at(3);
        out[3] = at(4);
        out[4] = at(5);
    }
}

impl MouseTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update(&mut self) -> (f32, f32, &[bool]) {
        let state: MouseState = self.device_state.get_mouse();
        self.last_position = (state.coords.0 as f32, state.coords.1 as f32);
        map_buttons(&state.button_pressed, &mut self.buttons);
        (self.last_position.0, self.last_position.1, &self.buttons)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn index_zero_is_never_a_button_and_left_maps_to_zero() {
        let mut out = [true; HUB_BUTTONS];
        map_buttons(&[true, true, false, false], &mut out);
        assert!(out[0], "device_query button 1 is the left button");
        assert!(!out[1] && !out[2], "slot 0 is padding, not a press");
    }

    #[test]
    fn right_button_lands_in_the_right_slot() {
        let mut out = [false; HUB_BUTTONS];
        let right = if cfg!(target_os = "linux") { 3 } else { 2 };
        let mut pressed = vec![false; 6];
        pressed[right] = true;
        map_buttons(&pressed, &mut out);
        assert_eq!(out, [false, true, false, false, false]);
    }
}
