use SpaceSandbox::control::SpaceControlPlugin;
use SpaceSandbox::network::NetworkPlugin;
use SpaceSandbox::objects::SpaceObjectsPlugin;
use SpaceSandbox::pawn_system::PawnPlugin;
use SpaceSandbox::scenes::NotificationPlugin;
use SpaceSandbox::scenes::fps_mode::FPSPlugin;
use SpaceSandbox::scenes::main_menu::MainMenuPlugin;
use SpaceSandbox::scenes::settings::SettingsPlugin;
use SpaceSandbox::scenes::station_builder::StationBuilderPlugin;
use SpaceSandbox::ship::save_load::DiskShipBase64;
use bevy::prelude::*;
use bevy_transform64::DTransformPlugin;
use space_physics::SpacePhysicsPlugin;

fn main() {
    App::default()
        .insert_resource(Msaa::default())
        .register_type::<DiskShipBase64>()
        .add_plugins(bevy::DefaultPlugins)
        .add_plugin(bevy_egui::EguiPlugin)
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
        .add_plugin(SettingsPlugin)
        .add_plugin(DTransformPlugin)
        .add_plugin(SpacePhysicsPlugin)
        .add_plugin(SpaceSandbox::editor::EditorPlugin)

        .run();
}
