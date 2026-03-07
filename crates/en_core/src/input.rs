use bevy_ecs::prelude::Resource;
use std::collections::HashSet;
pub use winit::keyboard::KeyCode;

#[derive(Default, Resource)]
pub struct Input {
    pressed: HashSet<KeyCode>,
    just_pressed: HashSet<KeyCode>,
    just_released: HashSet<KeyCode>,
}

impl Input {
    pub fn pressed(&self, key: KeyCode) -> bool { self.pressed.contains(&key) }
    pub fn just_pressed(&self, key: KeyCode) -> bool { self.just_pressed.contains(&key) }
    
    pub fn clear_frame(&mut self) {
        self.just_pressed.clear();
        self.just_released.clear();
    }

    pub fn press(&mut self, key: KeyCode) {
        if !self.pressed.contains(&key) {
            self.just_pressed.insert(key);
        }
        self.pressed.insert(key);
    }

    pub fn release(&mut self, key: KeyCode) {
        self.pressed.remove(&key);
        self.just_released.insert(key);
    }
}