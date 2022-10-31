mod api_base;
mod game;
mod input_system;
mod gui;
pub mod plugins;

use legion::systems::Builder;
use winit::dpi::PhysicalSize;
pub use api_base::*;
pub use game::*;
pub use input_system::*;
pub use gui::*;
use space_assets::Location;

use legion::*;

#[derive(PartialEq, Debug)]
pub enum PluginType {
    RenderPrepare,
    Render,
    Logic
}

#[derive(PartialEq)]
pub enum PluginName {
    Text(String),
}

pub trait SchedulePlugin {
    fn get_name(&self) -> PluginName;
    fn get_plugin_type(&self) -> PluginType;
    fn add_system(&self, game : &mut Game, builder : &mut legion::systems::Builder);
}

pub trait GuiPlugin {
    fn shot_top_panel(&mut self, game : &mut Game, ui : &mut egui::Ui);
    fn show_ui(&mut self, game : &mut Game, ctx : egui::Context) {}
}

pub trait RenderPlugin {
    fn update(&mut self, game : &mut Game) {}
    fn show_top_panel(&mut self, game : &mut Game, ui : &mut egui::Ui) {}
    fn show_ui(&mut self, game : &mut Game, ctx : egui::Context) {}
    fn render(&mut self, game : &mut Game, encoder : &mut wgpu::CommandEncoder) {}
    fn window_resize(&mut self, game : &mut Game, new_size : PhysicalSize<u32>) {}
}
