#![feature(test)]

pub mod ui;
pub mod scenes;
pub mod ship;
pub mod space_voxel;

use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use std::default::Default;
// use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";


#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum SceneType {
    MainMenu,
    ShipBuilding
}