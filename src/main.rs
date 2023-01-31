

use SpaceSandbox::scenes::StationBuildMenu;

use SpaceSandbox::ui::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::*;

fn main() {
    App::default()
        .add_plugins(bevy::DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(SpaceSandbox::ship::common::VoxelInstancePlugin)
        .add_plugin(MainMenuPlugin {})
        .add_plugin(StationBuildMenu {})
        .run();
}
