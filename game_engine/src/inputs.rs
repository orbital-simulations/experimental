use std::collections::{HashMap, HashSet};

use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseButton},
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub struct Inputs {
    // keyboard
    pub pressed_keys: HashSet<PhysicalKey>,
    pub key_events: HashMap<PhysicalKey, ElementState>,

    // mouse
    pub mouse_pressed_keys: HashSet<MouseButton>,
    pub mouse_events: HashMap<MouseButton, ElementState>,
    pub current_position: PhysicalPosition<f32>,
    pub cursor_delta: Option<(f32, f32)>,
    // TODO: Scroll wheel
}

impl Inputs {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_key(&mut self, key: &PhysicalKey, state: &ElementState) {
        match state {
            ElementState::Pressed => {
                self.pressed_keys.insert(*key);
            }
            ElementState::Released => {
                self.pressed_keys.remove(key);
            }
        }
        self.key_events.insert(*key, *state);
    }

    pub fn update_mouse_buttons(&mut self, button: &MouseButton, state: &ElementState) {
        match state {
            ElementState::Pressed => {
                self.mouse_pressed_keys.insert(*button);
            }
            ElementState::Released => {
                self.mouse_pressed_keys.remove(button);
            }
        }
        self.mouse_events.insert(*button, *state);
    }

    pub fn update_cursor_move(&mut self, position: PhysicalPosition<f32>) {
        self.current_position = position;
    }

    pub fn update_cursor_delta(&mut self, delta: (f32, f32)) {
        self.cursor_delta = Some(delta);
    }

    pub fn reset_events(&mut self) {
        self.key_events.clear();
        self.mouse_events.clear();
        self.cursor_delta = None;
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        let key = PhysicalKey::Code(key);
        self.pressed_keys.contains(&key)
    }

    pub fn cursor_moved(&self) -> bool {
        self.cursor_delta.is_some()
    }

    pub fn is_button_pressed(&self, button: MouseButton) -> bool {
        self.mouse_pressed_keys.contains(&button)
    }

    pub fn is_physical_key_pressed(&self, key: &PhysicalKey) -> bool {
        self.pressed_keys.contains(key)
    }
}
