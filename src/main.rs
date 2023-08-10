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
use bevy::asset::ChangeWatcher;
use bevy::prelude::*;
use bevy::render::RenderPlugin;
use bevy::render::settings::{WgpuSettings, WgpuFeatures};
use bevy_transform64::DTransformPlugin;

fn main() {
    App::default()
        .insert_resource(Msaa::default())
        .register_type::<DiskShipBase64>()
        .add_plugins(bevy::DefaultPlugins.set(AssetPlugin {
            watch_for_changes: ChangeWatcher::with_delay(std::time::Duration::from_secs(1)),
            ..default()
        }).set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            }
        }))
        .add_plugins(bevy_proto::prelude::ProtoPlugin::default())
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(SpaceSandbox::ship::common::VoxelInstancePlugin)
        .add_plugins(SpaceSandbox::ship::save_load::ShipPlugin)
        .add_plugins(MainMenuPlugin {})
        .add_plugins(StationBuilderPlugin {})
        .add_plugins(NotificationPlugin)
        .add_plugins(FPSPlugin)
        .add_plugins(PawnPlugin)
        .add_plugins(NetworkPlugin)
        .add_plugins(SpaceControlPlugin)
        .add_plugins(SpaceObjectsPlugin)
        .add_plugins(SettingsPlugin)
        .add_plugins(DTransformPlugin)
        .add_plugins(SpaceSandbox::editor::EditorPlugin)
        .add_plugins(SpaceSandbox::scenes::asset_editor::AssetEditorPlugin)
        .run();
}
