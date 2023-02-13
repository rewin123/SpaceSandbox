#![feature(test)]

pub mod ui;
pub mod scenes;
pub mod ship;
pub mod space_voxel;
pub mod pawn_system;
pub mod network;

use std::default::Default;
// use winit::window::Window;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use iyes_loopless::prelude::*;
    pub use crate::*;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum SceneType {
    MainMenu,
    ShipBuilding
}


#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum Gamemode {
    Godmode,
    FPS
}