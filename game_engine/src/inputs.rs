use std::collections::HashSet;

use winit::{
    event::ElementState,
    keyboard::{KeyCode, PhysicalKey},
};

#[derive(Default)]
pub struct InputKeys {
    pressed_keys: HashSet<PhysicalKey>,
}

impl InputKeys {
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
    }

    pub fn is_key_pressed(&self, key: KeyCode) -> bool {
        let key = PhysicalKey::Code(key);
        self.pressed_keys.contains(&key)
    }

    pub fn is_physical_key_pressed(&self, key: &PhysicalKey) -> bool {
        self.pressed_keys.contains(key)
    }
}
