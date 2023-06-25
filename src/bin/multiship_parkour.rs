use std::{fs::*, io::*, fmt::format};

use bevy::{prelude::*, render::{RenderPlugin, settings::{WgpuSettings, WgpuFeatures}}, math::DVec3, core_pipeline::bloom::BloomSettings};
use SpaceSandbox::{prelude::*, ship::save_load::DiskShipBase64, scenes::{main_menu::MainMenuPlugin, station_builder::StationBuilderPlugin, NotificationPlugin, fps_mode::{FPSPlugin, FPSController, self}, settings::SettingsPlugin}, pawn_system::{PawnPlugin, Pawn, ChangePawn}, network::NetworkPlugin, control::SpaceControlPlugin, objects::{SpaceObjectsPlugin, prelude::{GravityGenerator, GravityGeneratorPlugin}}};
use bevy_egui::{EguiContext, egui::{self, Color32}};
use space_physics::prelude::*;
use bevy_transform64::prelude::*;

#[derive(Component)]
struct RotateMe {
    pub ang_vel : DVec3
}

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

        .add_startup_system(startup)
        .add_startup_system(startup_player)

        .add_system(show_controller_settings)
        .add_system(rotate_my_system)

        .run();
}

fn show_controller_settings(
    mut ctx : Query<&mut EguiContext>,
    mut query : Query<(Entity, &DTransform, &mut FPSController)>,
    time : Res<Time>
) {
    if let Ok(mut ctx) = ctx.get_single_mut() {
        egui::Window::new("Controller Settings").show(ctx.get_mut(), |ui| {
            for (entity, tr, mut con) in query.iter_mut() {
                ui.label(format!("{:?}", entity));

                ui.add(
                    egui::DragValue::new(&mut con.walk_speed)
                    .prefix("Walk Speed:")
                    .fixed_decimals(1)
                );
                ui.add(
                    egui::DragValue::new(&mut con.run_speed)
                    .prefix("Run Speed:")
                    .fixed_decimals(1)
                );
                ui.add(
                    egui::DragValue::new(&mut con.jump_force)
                    .prefix("Jump Force:")
                    .fixed_decimals(1)
                );
                ui.add(
                    egui::Checkbox::new(&mut con.capture_control, "Capture Control")
                );

                ui.add(
                    egui::DragValue::new(&mut con.speed_relax)
                        .prefix("Speed Relax:")
                        .fixed_decimals(3)
                );
                ui.label(format!("Current speed: {:.2}", con.current_move.length()));

                ui.add(
                    egui::DragValue::new(&mut con.dash_speed)
                        .prefix("Dash Speed:")
                );
                ui.add(
                    egui::DragValue::new(&mut con.dash_interval)
                        .prefix("Dash Interval:")
                );
                ui.label(format!("Dash time: {:.2}", con.dash_time));

                if time.elapsed_seconds_f64() - con.dash_time > con.dash_interval {
                    ui.colored_label(Color32::GREEN, "Dash");
                } else {
                    ui.colored_label(Color32::YELLOW, "No dash");
                }

                ui.checkbox(&mut con.is_sprinting, "Is sprinting");

                ui.horizontal(|ui| {
                    ui.label("Default Up:");
                    ui.add(
                        egui::DragValue::new(&mut con.default_up.x)
                    );
                    ui.add(
                        egui::DragValue::new(&mut con.default_up.y)
                    );
                    ui.add(
                        egui::DragValue::new(&mut con.default_up.z)
                    );
                });

                ui.label(format!("Current Up: {:.2} {:.2} {:.2}", con.current_up.x, con.current_up.y, con.current_up.z));
                ui.label(format!("Current transform Up: {:.2} {:.2} {:.2}", tr.up().x, tr.up().y, tr.up().z));

                if ui.button("Save").clicked() {
                    let mut file = File::create(fps_mode::PATH_TO_CONTROLLER).unwrap();
                    file.write(
                        ron::to_string(con.as_ref()).unwrap().as_bytes()
                    );
                }
            }
        });
    }
}


fn startup_player(
    mut commands : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
) {
    fps_mode::startup_player(&mut commands, &mut pawn_event);
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
    spawn_ship(
        DVec3::new(11.0, -0.5, 0.0),
        &mut meshes,
        &mut materials,
        &mut commands);
    
    spawn_ship(
        DVec3::new(11.0, 11.0, 0.0),
        &mut meshes,
        &mut materials,
        &mut commands);
}

fn spawn_ship(
    pos : DVec3,
    mut meshes: &mut Assets<Mesh>, 
    mut materials: &mut Assets<StandardMaterial>, 
    mut commands: &mut Commands) {

    let plane_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Box::new(10.0, 0.5, 10.0))
    );

    let mat = materials.add(
        StandardMaterial {
            base_color : Color::DARK_GREEN,
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
        DTransform::from_xyz(pos.x, pos.y, pos.z)
    ))
    .insert(SpaceCollider(
        space_physics::prelude::ColliderBuilder::cuboid(5.0, 0.25, 5.0).build()
    ))
    .insert(SpaceRigidBodyType::Dynamic)
    .insert(GravityScale(0.0))
    .insert(
        RotateMe {
            ang_vel : DVec3::new(0.3, 0.0, 0.0)
        }
    )
    .insert(SpaceDominance(1))
    .insert(Velocity::default())
    .insert(GravityGenerator {
        gravity_force: DVec3::new(0.0, -9.81, 0.0),
        radius: 5.0,
    });
}

fn rotate_my_system(
    mut query : Query<(&RotateMe, &mut Velocity)>,

) {
    for (me, mut vel) in query.iter_mut() {
        vel.angvel = me.ang_vel;
    }
}

fn prepare_enviroment(
        mut meshes: &mut Assets<Mesh>, 
        mut materials: &mut Assets<StandardMaterial>, 
        mut commands: &mut Commands) {
    let plane_mesh = meshes.add(
        Mesh::from(bevy::prelude::shape::Box::new(10.0, 0.5, 10.0))
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
        space_physics::prelude::ColliderBuilder::cuboid(5.0, 0.25, 5.0).build()
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