#![feature(default_free_fn)]

pub mod ui;
pub mod scenes;

use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use std::default::Default;
// use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";
