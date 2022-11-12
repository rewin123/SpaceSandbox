mod api_base;
mod game;
mod input_system;
mod gui;
pub mod plugins;

use std::{sync::Arc, ops::Deref};

use bevy_ecs::prelude::Component;
use winit::dpi::PhysicalSize;
pub use api_base::*;
pub use game::*;
pub use input_system::*;
pub use gui::*;
use space_assets::Location;
use space_core::{ecs::StageLabel, RenderBase};

#[derive(PartialEq, Debug)]
pub enum GlobalStageStep {
    PreRender,
    Render,
    PostRender,
    Gui
}

impl StageLabel for GlobalStageStep {
    fn as_str(&self) -> &'static str {
        match self {
            GlobalStageStep::PreRender => {"PreRender"}
            GlobalStageStep::Render => {"Render"}
            GlobalStageStep::PostRender => {"PostRender"}
            GlobalStageStep::Gui => {"Gui"}
        }
    }
}

pub struct CameraBuffer {
    pub buffer : wgpu::Buffer
}
pub struct ScreenSize {
    pub size : winit::dpi::PhysicalSize<u32>,
    pub format : wgpu::TextureFormat
}

pub struct RenderApi {
    pub base : Arc<RenderBase>
}

impl Deref for RenderApi {
    type Target = Arc<RenderBase>;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

#[derive(PartialEq)]
pub enum PluginName {
    Text(String),
}

pub trait SchedulePlugin {
    fn get_name(&self) -> PluginName;
    fn add_system(&self, app : &mut space_core::app::App);
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