use std::sync::Arc;

use bevy::{prelude::*, math::{DVec3, DQuat}};
use bevy_transform64::prelude::*;
use rand::Rng;
use space_physics::prelude::*;

#[derive(Resource, Default)]
struct Loading {
    pub handle : Option<Handle<Scene>>
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DTransformPlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(SpacePhysicsPlugin)
        .insert_resource(Loading::default())

        .add_startup_system(setup)
        .add_system(loading_info)
        .run();
}

fn setup(
    mut commands : Commands,
    asset_server : Res<AssetServer>,
    mut loading : ResMut<Loading>
) {
    // let asset_path = "space_objects/asteroid_1.glb#Scene0";
    let asset_path = "space_objects/asteroid_1.glb#Scene0";

    let handle = asset_server.load(asset_path);

    loading.handle = Some(handle.clone());

    commands.spawn(SceneBundle {
        scene: handle.clone(),
        ..default()
    })
    .insert(DTransformBundle::from_transform(
        DTransform::from_xyz(0.0, 0.0, 0.0),
    ));

    // Add a camera
    commands.spawn(Camera3dBundle {
        camera_3d : Camera3d {
            clear_color : bevy::core_pipeline::clear_color::ClearColorConfig::Custom(Color::Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }),
            ..default()
        },
        ..Default::default()
    })
    .insert(DTransformBundle::from_transform(
        DTransform::from_xyz(5.0, 5.0, 5.0).looking_at(DVec3::ZERO, DVec3::Y),
    ));

    commands.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    const HALF_SIZE: f32 = 100.0;
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadows_enabled: true,
            ..default()
        },
        ..default()
    }).insert(DTransformBundle::from_transform(DTransform {
        translation: DVec3::new(0.0, 2.0, 0.0),
        rotation: DQuat::from_rotation_z(-0.5),
        ..default()
    }));
}

fn loading_info(
    mut asset_server : ResMut<AssetServer>,
    mut loading : ResMut<Loading>
) {
    if let Some(handle) = &loading.handle {
        match asset_server.get_load_state(handle) {
            bevy::asset::LoadState::NotLoaded => {
                println!("Not loaded");
                loading.handle = None;
            },
            bevy::asset::LoadState::Loading => {
                println!("Loading");
            },
            bevy::asset::LoadState::Loaded => {
                println!("Loaded");
                loading.handle = None;
            },
            bevy::asset::LoadState::Failed => {
                println!("Failed");
                loading.handle = None;
            },
            bevy::asset::LoadState::Unloaded => {
                println!("Unloaded");
                loading.handle = None;
            },
        }
    }
}