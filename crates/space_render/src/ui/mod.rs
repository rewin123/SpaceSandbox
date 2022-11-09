mod fps_counter;
mod api_info;
mod gltf_to_load;

use std::{sync::Arc, time::Instant};

use egui::{FontDefinitions, Style, Window, FullOutput};
use egui_wgpu_backend::ScreenDescriptor;
pub use fps_counter::*;
pub use api_info::*;
pub use gltf_to_load::*;
use space_core::RenderBase;
