use std::fs::File;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::{Arc, Mutex};

use std::default::Default;
use simplelog::*;
// use winit::window::Window;

const EngineName : &str = "Rewin engine";
const AppName : &str = "SpaceSandbox";

pub mod ui;


pub fn init_logger() {
    let _ = CombinedLogger::init(
        vec![
            TermLogger::new(LevelFilter::Error, Config::default(), TerminalMode::Mixed, ColorChoice::Auto),
            WriteLogger::new(LevelFilter::Error, Config::default(), File::create("detailed.log").unwrap())
        ]
    );
}
