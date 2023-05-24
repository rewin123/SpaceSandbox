use bevy::{prelude::*, math::DVec3};
use bevy_proto::prelude::{Schematic, ReflectSchematic};
use bevy_prototype_debug_lines::*;
use bevy_transform64::prelude::{DTransform, DGlobalTransform};

use crate::DSpatialBundle;

#[derive(Component)]
pub struct RadarDetected {
    pub color : Color
}

#[derive(Component, Reflect, FromReflect, Schematic)]
#[reflect(Schematic)]
pub struct Radar {
    pub points : Vec<Entity>,
    pub radius : f64,
    pub scale : f64,
    pub central_object : Option<Entity>,
}

impl Default for Radar {
    fn default() -> Self {
        Radar {
            points : Vec::new(),
            radius : 10000.0,
            scale : 0.25,
            central_object : None,
        }
    }
}

#[derive(Component)]
pub struct RadarPoint;

pub struct RadarPlugin;

#[derive(Resource)]
struct RadarResource {
    pub mesh : Handle<Mesh>,
    pub material : Handle<StandardMaterial>
}

impl Plugin for RadarPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(radar_resource_init);
        app.register_type::<Radar>();
        app.add_system(
            radar
        );
    }
}

fn radar_resource_init(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>
) {
    let cube_mesh = meshes.add(Mesh::from(shape::Cube { size: 0.01 }));
    let cube_material = materials.add(StandardMaterial {
        base_color : Color::rgb_linear(1.0, 1.0, 0.0),
        unlit : true,
        ..default()
    });
    commands.insert_resource(RadarResource {
        mesh : cube_mesh,
        material : cube_material
    });
}

fn radar(
    mut commands : Commands,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>,
    mut radars : Query<(Entity, &DGlobalTransform, &mut Radar)>,
    mut objects : Query<(&DGlobalTransform, &RadarDetected), Without<Radar>>,
    mut radar_points : Query<&mut DTransform, With<RadarPoint>>,
    radar_res : Res<RadarResource>
) {
    for (radar_e, radar_transform, mut radar) in radars.iter_mut() {
        let mut idx = 0;

        if radar.central_object.is_none() {
            let cube = meshes.add(Mesh::from(shape::Cube { size: 0.01 }));
            let cube_material = materials.add(StandardMaterial {
                base_color : Color::rgb_linear(0.0, 0.0, 1.0),
                unlit : true,
                ..default()
            });

            let central_object = commands.spawn(PbrBundle {
                mesh : cube.clone(),
                material : cube_material,
                transform : Transform::from_xyz(0.0, 0.0, 0.0),
                ..Default::default()
            }).id();

            radar.central_object = Some(central_object);
            commands.entity(radar_e).add_child(central_object);

            //x axis cube
            let x_axis_material = materials.add(StandardMaterial {
                base_color : Color::rgb_linear(0.3, 0.0, 0.0),
                unlit : true,
                ..default()
            });
            
            //z axis cube
            let z_axis_material = materials.add(StandardMaterial {
                base_color : Color::rgb_linear(0.0, 0.3, 0.0),
                unlit : true,
                ..default()
            });

            let ticks = 10;
            for dx in -ticks..=ticks {
                let dx_pos = Vec3::new(dx as f32, 0.0, 0.0) / ticks as f32 * radar.scale as f32;
                let x_axis = commands.spawn(PbrBundle {
                    mesh : cube.clone(),
                    material : x_axis_material.clone(),
                    transform : Transform::from_xyz(0.0, -radar.scale as f32, dx_pos.x).with_scale(Vec3::new(50.0, 0.1, 0.1)),
                    ..Default::default()
                }).id();
                commands.entity(radar_e).add_child(x_axis);

                let z_axis = commands.spawn(PbrBundle {
                    mesh : cube.clone(),
                    material : z_axis_material.clone(),
                    transform : Transform::from_xyz(dx_pos.x, -radar.scale as f32, 0.0).with_scale(Vec3::new(0.1, 0.1, 50.0)),
                    ..Default::default()
                }).id();

                commands.entity(radar_e).add_child(z_axis);
            }
           

           
        }

        let radar_forward = radar_transform.forward();
        let radar_right = radar_transform.right();
        let radar_up = radar_transform.up();
        let radar_pos = radar_transform.translation();

        for (object_transform, mut detected) in objects.iter_mut() {
            let dp = object_transform.translation() - radar_transform.translation();
            let distance = dp.length();
            if distance < radar.radius {
                let radar_pos = dp / radar.radius * radar.scale;
                let radar_pos = DVec3::new(radar_right.dot(radar_pos),
                radar_up.dot(radar_pos),
                       -radar_forward.dot(radar_pos));
                
                if idx >= radar.points.len() {
                    let point = commands.spawn(
                        DSpatialBundle::from_transform(DTransform::from_xyz(radar_pos.x, radar_pos.y, radar_pos.z))
                    ).insert(radar_res.mesh.clone())
                    .insert(radar_res.material.clone())
                    .insert(RadarPoint).id();

                    radar.points.push(point);
                    commands.entity(radar_e).add_child(point);
                } else {
                    if let Ok(mut point) = radar_points.get_mut(radar.points[idx]) {
                        point.translation = radar_pos;
                    }
                }
                idx += 1;
            }
        }

        let despawn_vec = radar.points[idx..].to_vec();
        for despawn_e in despawn_vec {
            commands.entity(despawn_e).despawn_recursive();
        }
        radar.points.truncate(idx);
    }
}