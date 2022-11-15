use std::collections::HashMap;
use bevy::prelude::Resource;
use winit::event::{ElementState, KeyboardInput};
pub use winit::event::VirtualKeyCode as KeyCode;

#[derive(Default, Resource)]
pub struct InputSystem {
    key_state : HashMap<winit::event::VirtualKeyCode, bool>
}

impl InputSystem {
    pub fn process_event(&mut self, input : &KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            // log::info!("New {:?} state {:?}", &key, &input.state);
            self.key_state.insert(key, input.state == ElementState::Pressed);
        }
    }

    pub fn get_key_state(&self, key : KeyCode) -> bool {
        if let Some(state) = self.key_state.get(&key) {
            *state
        } else {
            false
        }
    }
}