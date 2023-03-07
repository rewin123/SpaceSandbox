use SpaceSandbox::control::SpaceControlPlugin;
use SpaceSandbox::network::NetworkPlugin;
use SpaceSandbox::objects::SpaceObjectsPlugin;
use SpaceSandbox::pawn_system::PawnPlugin;
use SpaceSandbox::scenes::fps_mode::FPSPlugin;
use SpaceSandbox::scenes::station_builder::StationBuilderPlugin;
use SpaceSandbox::ship::{DiskShipBase64};
use SpaceSandbox::ui::*;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn main() {
    App::default()
        .register_type::<DiskShipBase64>()
        .add_plugins(bevy::DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(SpaceSandbox::ship::common::VoxelInstancePlugin)
        .add_plugin(SpaceSandbox::ship::save_load::ShipPlugin)
        .add_plugin(MainMenuPlugin {})
        .add_plugin(StationBuilderPlugin {})
        .add_plugin(NotificationPlugin)
        .add_plugin(FPSPlugin)
        .add_plugin(PawnPlugin)
        .add_plugin(NetworkPlugin)
        .add_plugin(SpaceControlPlugin)
        .add_plugin(SpaceObjectsPlugin)

        .run();
}
