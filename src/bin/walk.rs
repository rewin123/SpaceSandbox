use std::{fs::*, io::*};

use bevy::{prelude::*, render::{RenderPlugin, settings::{WgpuSettings, WgpuFeatures}}, math::DVec3, core_pipeline::bloom::BloomSettings};
use SpaceSandbox::{prelude::*, ship::save_load::DiskShipBase64, scenes::{main_menu::MainMenuPlugin, station_builder::StationBuilderPlugin, NotificationPlugin, fps_mode::{FPSPlugin, FPSController}, settings::SettingsPlugin}, pawn_system::{PawnPlugin, Pawn, ChangePawn}, network::NetworkPlugin, control::SpaceControlPlugin, objects::SpaceObjectsPlugin};
use bevy_egui::{EguiContext, egui};
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
        .add_system(show_controller_settings)
        .run();
}

fn show_controller_settings(
    mut ctx : Query<&mut EguiContext>,
    mut query : Query<(Entity, &mut FPSController)>
) {
    if let Ok(mut ctx) = ctx.get_single_mut() {
        egui::Window::new("Controller Settings").show(ctx.get_mut(), |ui| {
            for (entity, mut con) in query.iter_mut() {
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


                if ui.button("Save").clicked() {
                    let mut file = File::create(PATH_TO_CONTROLLER).unwrap();
                    file.write(
                        ron::to_string(con.as_ref()).unwrap().as_bytes()
                    );
                }
            }
        });
    }
}

const PATH_TO_CONTROLLER : &str = "conroller.ron";

fn startup_player(
    mut commands : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
) {
    let mut cam = Camera::default();
    cam.hdr = false;
    cam.is_active = false;

    let controller_setting = {
        let mut con = FPSController::default();
        if let Ok(mut file) = File::open(PATH_TO_CONTROLLER) {
            let mut data = String::new();
            file.read_to_string(&mut data);
            if let Ok(file_con) = ron::from_str::<FPSController>(&data) {
                con = file_con;
            }
        }
        con
    };

    let pos = DVec3::new(0.0, 3.0, 0.0);
    let pawn = commands.spawn(
        SpaceCollider(
        ColliderBuilder::capsule_y(0.75, 0.25).build()))
    .insert(DSpatialBundle::from_transform(DTransform::from_xyz(pos.x, pos.y, pos.z)))
    .insert(SpaceRigidBodyType::Dynamic)
    .insert(SpaceLockedAxes::ROTATION_LOCKED)
    .insert(GravityScale(1.0))
    .insert(Velocity::default())
    .insert(controller_setting)
    .id();

    info!("Locked rotation {:?}", SpaceLockedAxes::ROTATION_LOCKED);

    let cam_pawn = commands.spawn(Camera3dBundle {
        camera : cam,
        camera_3d : Camera3d {
            clear_color : bevy::core_pipeline::clear_color::ClearColorConfig::Custom(Color::Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }),
            ..default()
        },
        ..default()
    })
    .insert(DTransformBundle::from_transform(
        DTransform::from_xyz(0.0, 1.0, 0.0).looking_at(DVec3::new(0.0, 1.0, -1.0), DVec3::Y)
    ))
    .insert(BloomSettings::default()).id();

    commands.entity(pawn).add_child(cam_pawn);

    commands.entity(pawn).insert(Pawn { camera_id: cam_pawn });

    pawn_event.send(ChangePawn { new_pawn: pawn, save_stack: true });
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