// #![feature(test)]

pub mod ui;
pub mod scenes;
pub mod ship;
pub mod space_voxel;
pub mod pawn_system;
pub mod network;
pub mod asset_utils;
pub mod control;
pub mod objects;

use std::default::Default;

use bevy::prelude::*;
// use winit::window::Window;

pub mod prelude {
    pub use bevy::prelude::*;
    pub use crate::*;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, States, Default)]
pub enum SceneType {
    #[default]
    MainMenu,
    ShipBuilding
}


#[derive(Clone, Hash, PartialEq, Eq, Debug, States, Default)]
pub enum Gamemode {
    #[default]
    Godmode,
    FPS
}