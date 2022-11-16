use std::collections::HashMap;
use bevy::prelude::Resource;
use winit::dpi::PhysicalPosition;
use winit::event::{ElementState, KeyboardInput, MouseButton};
pub use winit::event::VirtualKeyCode as KeyCode;

#[derive(Default, Resource)]
pub struct InputSystem {
    key_state : HashMap<winit::event::VirtualKeyCode, bool>,
    mouse_buttons : HashMap<winit::event::MouseButton, bool>,
    pos : nalgebra::Point2<f32>
}

impl InputSystem {
    pub fn process_event(&mut self, input : &KeyboardInput) {
        if let Some(key) = input.virtual_keycode {
            // log::info!("New {:?} state {:?}", &key, &input.state);
            self.key_state.insert(key, input.state == ElementState::Pressed);
        }
    }

    pub fn process_mouse_event(&mut self, button : &MouseButton, state : &ElementState) {
        let is_pressed = *state == ElementState::Pressed;
        self.mouse_buttons.insert(button.clone(), is_pressed);
    }

    pub fn process_cursor_move(&mut self, pos : PhysicalPosition<f64>) {
        self.pos.x = pos.x as f32;
        self.pos.y = pos.y as f32;
    }

    pub fn get_key_state(&self, key : KeyCode) -> bool {
        if let Some(state) = self.key_state.get(&key) {
            *state
        } else {
            false
        }
    }

    pub fn get_mouse_button_state(&self, button : &MouseButton) -> bool {
        if let Some(state) = self.mouse_buttons.get(button) {
            state.clone()
        } else {
            false
        }
    }

    pub fn get_mouse_pos(&self) -> nalgebra::Point2<f32> {
        self.pos.clone()
    }

}