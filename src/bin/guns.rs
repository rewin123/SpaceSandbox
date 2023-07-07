use std::{fs::*, io::*, fmt::format, ops::{Add, Sub, Mul}};

use bevy::{prelude::*, render::{RenderPlugin, settings::{WgpuSettings, WgpuFeatures}}, math::{DVec3, DQuat}, core_pipeline::bloom::BloomSettings};
use SpaceSandbox::{prelude::*, ship::save_load::DiskShipBase64, scenes::{main_menu::MainMenuPlugin, station_builder::StationBuilderPlugin, NotificationPlugin, fps_mode::{FPSPlugin, FPSController, self}, settings::SettingsPlugin}, pawn_system::{PawnPlugin, Pawn, ChangePawn}, network::NetworkPlugin, control::SpaceControlPlugin, objects::{SpaceObjectsPlugin, prelude::{GravityGenerator, GravityGeneratorPlugin}, guns::gun_grab::{GunGrabPlugin, GunGrab}}};
use bevy_egui::{EguiContext, egui::{self, Color32}};
use space_physics::prelude::*;
use bevy_transform64::prelude::*;

#[derive(Debug, Component, Clone, Copy)]
pub struct GunTag;

#[derive(Debug, Component, Clone)]
pub struct ProjectileGun {
    damage : Damage
}

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
pub struct Damage {
    kinetic_damage : f32,
    radiation_damage : f32,
    termal_damage : f32,
    pressure_damage : f32,
    emissive_damage : f32,   
    radio_damage : f32,
}

impl Add for Damage {
    type Output = Self;
    fn add(self, rhs: Self) -> Self {
        Self {
            kinetic_damage : self.kinetic_damage + rhs.kinetic_damage,
            radiation_damage : self.radiation_damage + rhs.radiation_damage,
            termal_damage : self.termal_damage + rhs.termal_damage,
            pressure_damage : self.pressure_damage + rhs.pressure_damage,
            emissive_damage : self.emissive_damage + rhs.emissive_damage,
            radio_damage : self.radio_damage + rhs.radio_damage,
        }
    }
}

impl Sub for Damage {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self {
        Self {
            kinetic_damage : self.kinetic_damage - rhs.kinetic_damage,
            radiation_damage : self.radiation_damage - rhs.radiation_damage,
            termal_damage : self.termal_damage - rhs.termal_damage,
            pressure_damage : self.pressure_damage - rhs.pressure_damage,
            emissive_damage : self.emissive_damage - rhs.emissive_damage,
            radio_damage : self.radio_damage - rhs.radio_damage,
        }
    }
}

impl Mul for Damage {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self {
        Self {
            kinetic_damage : self.kinetic_damage * rhs.kinetic_damage,
            radiation_damage : self.radiation_damage * rhs.radiation_damage,
            termal_damage : self.termal_damage * rhs.termal_damage,
            pressure_damage : self.pressure_damage * rhs.pressure_damage,
            emissive_damage : self.emissive_damage * rhs.emissive_damage,
            radio_damage : self.radio_damage * rhs.radio_damage,
        }
    }
}

#[derive(Component, Copy, Clone, Default)]
pub struct Health {
    pub health : f32,
    pub max_health : f32,
}

#[derive(Component, Copy, Clone, Default)]
pub struct Armor {
    pub k : Damage
}

#[derive(Component, Debug)]
pub struct EnemyTag;

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
        .add_plugin(GravityGeneratorPlugin)
        .add_plugin(GunGrabPlugin)

        .add_startup_system(startup)
        .add_startup_system(startup_player)
        .add_startup_system(prepare_enemies)

        .add_system(fps_mode::show_controller_settings)

        .run();
}

#[derive(Clone)]
struct EnemyConfig {
    mesh : Handle<Mesh>,
    material : Handle<StandardMaterial>,
    health :  Health
}

fn prepare_enemies(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
) {
    let config = EnemyConfig {
        mesh : meshes.add(Mesh::from(bevy::prelude::shape::Capsule {
            radius: 0.5,
            depth: 2.0,
            ..default()
        })),
        material : materials.add(StandardMaterial {
            base_color : Color::RED,
            ..default()
        }),
        health : Health {
            health : 1.0,
            max_health : 1.0
        }
    };

    for i in 0..10 {
        let pos = DVec3::new(
            i as f64 * 2.0,
            1.0,
            10.0
        );
        spawn_enemy(
            &mut commands,
            &mut meshes,
            &mut materials,
            pos,
            &config
        );
    }
}

fn spawn_enemy(
    mut commands : &mut Commands,
    mut meshes : &mut Assets<Mesh>,
    mut materials : &mut Assets<StandardMaterial>,
    pos : DVec3,
    config : &EnemyConfig
) {
    let enemy = commands.spawn(
        PbrBundle {
            mesh: config.mesh.clone(),
            material : config.material.clone(),
        ..default()
    })
    .insert(EnemyTag)
    .insert(DTransformBundle::from_transform(
        DTransform::from_translation(pos)
    ))
    .insert(config.health.clone())
    .id();
}

fn startup_player(
    mut commands : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>
) {
    let player = fps_mode::startup_player(&mut commands, &mut pawn_event);

    let gun = commands.spawn(
        PbrBundle {
            mesh: meshes.add(Mesh::from(bevy::prelude::shape::Capsule {
                radius: 0.1,
                depth: 1.0,
                ..default()
            })),
            material : materials.add(StandardMaterial {
                base_color : Color::GRAY,
                ..default()
            }),
            ..default()
        }
    ).insert(DTransformBundle::from_transform(
        DTransform::from_xyz(-0.5, 0.5, 0.5)
        .with_rotation(DQuat::from_axis_angle(DVec3::X, 3.14 / 2.0))
    ))
    .id();

    commands.entity(player.pawn).
        insert(GravityScale(1.0))
        .insert(GunGrab {
            gun_id : gun,
            cam_id : player.camera,
            shift : DVec3::new(0.5, 0.5, 0.5)
        });

    commands.entity(player.pawn)
        .add_child(gun);
}

fn startup(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
) {
    prepare_enviroment(
        &mut meshes,
        &mut materials, 
        &mut commands);
}

fn prepare_enviroment(
        mut meshes: &mut Assets<Mesh>, 
        mut materials: &mut Assets<StandardMaterial>, 
        mut commands: &mut Commands) {
    let size : f64 = 100.0;
    let plane_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Box::new(size as f32, 0.5, size as f32))
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
        space_physics::prelude::ColliderBuilder::cuboid(size / 2.0, 0.25, size / 2.0).build()
    ))
    .insert(GravityGenerator {
        gravity_force : DVec3::new(0.0, -9.81, 0.0),
        radius : 5.0
    });

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