

use SpaceSandbox::scenes::StationBuildMenu;

use SpaceSandbox::ship::{Ship, ShipBlock};
use SpaceSandbox::space_voxel::objected_voxel_map::VoxelVal;
use SpaceSandbox::space_voxel::solid_voxel_map::SolidVoxelMap;
use SpaceSandbox::ui::*;
use bevy::prelude::*;
use bevy_inspector_egui::quick::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::default()
        .register_type::<ShipBlock>()
        .register_type::<VoxelVal<ShipBlock>>()
        .register_type::<SolidVoxelMap<VoxelVal<ShipBlock>>>()
        .register_type::<Ship>()
        .add_plugins(bevy::DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(WorldInspectorPlugin)
        .add_plugin(SpaceSandbox::ship::common::VoxelInstancePlugin)
        .add_plugin(MainMenuPlugin {})
        .add_plugin(StationBuildMenu {})
        .run();
}
