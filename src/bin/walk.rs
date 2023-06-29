use std::{fs::*, io::*};

use bevy::{prelude::*, render::{RenderPlugin, settings::{WgpuSettings, WgpuFeatures}}, math::DVec3, core_pipeline::bloom::BloomSettings};
use SpaceSandbox::{prelude::*, ship::save_load::DiskShipBase64, scenes::{main_menu::MainMenuPlugin, station_builder::StationBuilderPlugin, NotificationPlugin, fps_mode::{FPSPlugin, FPSController, self}, settings::SettingsPlugin}, pawn_system::{PawnPlugin, Pawn, ChangePawn}, network::NetworkPlugin, control::SpaceControlPlugin, objects::SpaceObjectsPlugin};
use bevy_egui::{EguiContext, egui::{self, Color32}};
use space_physics::prelude::*;
use bevy_transform64::prelude::*;

fn main() {
    App::default()
        .insert_resource(Msaa::default())
        .register_type::<DiskShipBase64>()
        .add_plugins(bevy::DefaultPlugins.set(AssetPlugin {
            watch_for_changes: true,
            ..default()
        }).set(RenderPlugin {
            wgpu_settings: WgpuSettings {
                features: WgpuFeatures::POLYGON_MODE_LINE,
                ..default()
            }
        }))
        .add_plugin(FPSPlugin)
        .add_plugin(bevy_proto::prelude::ProtoPlugin::default())
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(SpaceSandbox::ship::common::VoxelInstancePlugin)
        .add_plugin(NotificationPlugin)
        .add_plugin(PawnPlugin)
        .add_plugin(DTransformPlugin)
        .add_plugin(SpacePhysicsPlugin)
        .add_plugin(SpaceControlPlugin)
        .add_plugin(SettingsPlugin)

        .add_startup_system(startup)
        .add_startup_system(startup_player)
        .add_system(fps_mode::show_controller_settings)
        .run();
}


fn startup_player(
    mut commands : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
) {
    let pawn = fps_mode::startup_player(&mut commands, &mut pawn_event).pawn;
    commands.entity(pawn).
        insert(GravityScale(1.0));
}

fn startup(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
) {
    prepare_enviroment(meshes, materials, commands);
}

fn prepare_enviroment(mut meshes: ResMut<'_, Assets<Mesh>>, mut materials: ResMut<'_, Assets<StandardMaterial>>, mut commands: Commands<'_, '_>) {
    let plane_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Box::new(100.0, 0.5, 100.0))
    );
    let cube_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Cube::new(1.0))
    );
    let mat = materials.add(
        StandardMaterial {
            base_color : Color::GRAY,
            ..default()
        }
    );

    commands.spawn(
        PbrBundle {
            mesh: plane_mesh.clone(),
            material : mat.clone(),
            ..default()
        }
    ).insert(DTransformBundle::from_transform(
        DTransform::from_xyz(0.0, -0.5, 0.0)
    ))
    .insert(SpaceCollider(
        space_physics::prelude::ColliderBuilder::cuboid(50.0, 0.25, 50.0).build()
    ));

    let cube_poses = vec![
        DVec3::new(5.0, 0.0, 0.0),
        DVec3::new(-5.0, 0.0, 0.0),
        DVec3::new(0.0, 0.0, 5.0),
        DVec3::new(0.0, 0.0, -5.0),
    ];
    let cube_transforms = cube_poses.iter().map(|pos| {
        DTransformBundle::from_transform(
            DTransform::from_xyz(pos.x, pos.y, pos.z)
        )
    }).collect::<Vec<_>>();

    for cube_transform in cube_transforms {
        commands.spawn(PbrBundle {
            mesh: cube_mesh.clone(),
            material : mat.clone(),
            ..default()
        }).insert(cube_transform)
        .insert(SpaceCollider(
            space_physics::prelude::ColliderBuilder::cuboid(0.5, 0.5, 0.5).build()
        ));
    }
    
    // ambient light
    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadows_enabled: true,
            ..default()
        },
        ..default()
    }).insert(DTransformBundle::from_transform(
        DTransform::from_xyz(10.0, 10.0, 10.0).looking_at(DVec3::new(0.0, 0.0, 0.0), DVec3::Y)
    ));
}