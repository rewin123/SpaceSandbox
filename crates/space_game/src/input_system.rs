use std::collections::HashMap;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Default)]
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

    pub fn get_key_state(&self, key : VirtualKeyCode) -> bool {
        if let Some(state) = self.key_state.get(&key) {
            *state
        } else {
            false
        }
    }
}