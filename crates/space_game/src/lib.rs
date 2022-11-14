mod api_base;
mod game;
mod input_system;
mod gui;
pub mod plugins;

use std::{sync::Arc, ops::Deref};
use std::marker::PhantomData;
use std::ops::DerefMut;
use bevy::app::prelude::{App, Plugin};
use bevy::asset::*;

use bevy::ecs::prelude::Component;
use winit::dpi::PhysicalSize;
pub use api_base::*;
pub use game::*;
pub use input_system::*;
pub use gui::*;
use space_assets::Location;
use space_core::{ecs::StageLabel, RenderBase};
use space_core::asset::*;
use space_core::serde::*;
use space_core::ecs::Resource;
use bevy::reflect::TypeUuid;

#[derive(Resource)]
pub struct RenderCommands {
    pub encoder : wgpu::CommandEncoder
}

impl Deref for RenderCommands {
    type Target = wgpu::CommandEncoder;

    fn deref(&self) -> &Self::Target {
        &self.encoder
    }
}

impl DerefMut for RenderCommands {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.encoder
    }
}

pub struct WallRonPlugin {}

impl SchedulePlugin for WallRonPlugin {
    fn get_name(&self) -> PluginName {
        PluginName::Text("WallRon loader".into())
    }

    fn add_system(&self, app: &mut App) {
        app.add_plugin(RonAssetPlugin::<WallRon> {
            ext: vec!["wall"],
            phantom: Default::default()
        });
    }
}

#[derive(Default, Serialize, Deserialize, TypeUuid)]
#[uuid = "4cbf98d8-2039-4262-992b-baaa12dcc6c8"]
pub struct WallRon {
    pub name : String,
    pub gltf_path : String
}

#[derive(Default)]
pub struct RonLoader<T> {
    pub ext : Vec<&'static str>,
    phantom : PhantomData<T>
}

impl<T> RonLoader<T> {
    pub fn new(ext : &'static str) -> RonLoader<T> {
        Self {
            ext : vec![ext],
            phantom : PhantomData::default()
        }
    }
}

impl<T> AssetLoader for RonLoader<T>
        where for<'de> T: space_core::serde::Deserialize<'de> + Asset {
    fn load<'a>(&'a self, bytes: &'a [u8], load_context: &'a mut LoadContext) -> BoxedFuture<'a,  Result<(), space_core::bevy::asset::Error>> {
        Box::pin(async move {
           let asset = space_core::ron::de::from_bytes::<T>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &self.ext
    }
}

pub struct RonAssetPlugin<T> {
    pub ext : Vec<&'static str>,
    phantom : PhantomData<T>
}

impl<T> Plugin for RonAssetPlugin<T>
where
        for<'de> T: space_core::serde::Deserialize<'de> + Asset{
    fn build(&self, app: &mut App) {
        app.add_asset::<T>().add_asset_loader(RonLoader::<T> {
            ext: self.ext.clone(),
            phantom : PhantomData::default()
        });
    }
}

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

#[derive(Resource)]
pub struct CameraBuffer {
    pub buffer : wgpu::Buffer
}

#[derive(Resource)]
pub struct ScreenSize {
    pub size : winit::dpi::PhysicalSize<u32>,
    pub format : wgpu::TextureFormat
}

#[derive(Resource)]
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