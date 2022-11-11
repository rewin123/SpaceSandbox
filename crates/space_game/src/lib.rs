mod api_base;
mod game;
mod input_system;
mod gui;
pub mod plugins;

use winit::dpi::PhysicalSize;
pub use api_base::*;
pub use game::*;
pub use input_system::*;
pub use gui::*;
use space_assets::Location;
use space_core::ecs::StageLabel;

#[derive(PartialEq, Debug)]
pub enum GlobalStageStep {
    RenderPrepare,
    RenderStart,
    Render,
    PostRender,
    Update,
    PostUpdate,
    Gui
}

impl StageLabel for GlobalStageStep {
    fn as_str(&self) -> &'static str {
        match self {
            GlobalStageStep::RenderPrepare => {"RenderPrepare"}
            GlobalStageStep::RenderStart => {"RenderStart"}
            GlobalStageStep::Render => {"Render"}
            GlobalStageStep::Update => {"Update"}
            GlobalStageStep::PostUpdate => {"PostUpdate"},
            GlobalStageStep::PostRender => {"PostRender"}
            GlobalStageStep::Gui => {"Gui"}
        }
    }
}

#[derive(PartialEq)]
pub enum PluginName {
    Text(String),
}

pub trait SchedulePlugin {
    fn get_name(&self) -> PluginName;
    fn add_system(&self, game : &mut Game, builder : &mut space_core::ecs::Schedule);
}

pub trait GuiPlugin {
    fn shot_top_panel(&mut self, game : &mut Game, ui : &mut egui::Ui) -> Vec<GameCommands> {vec![]}
    fn show_ui(&mut self, game : &mut Game, ctx : egui::Context) -> Vec<GameCommands> {vec![]}
}

pub trait RenderPlugin {
    fn update(&mut self, game : &mut Game) {}
    fn show_top_panel(&mut self, game : &mut Game, ui : &mut egui::Ui) {}
    fn show_ui(&mut self, game : &mut Game, ctx : egui::Context) {}
    fn render(&mut self, game : &mut Game) {}
    fn window_resize(&mut self, game : &mut Game, new_size : PhysicalSize<u32>) {}
}

pub enum GameCommands {
    AbstractChange(Box<dyn FnOnce(&mut Game)>),
    Exit
}