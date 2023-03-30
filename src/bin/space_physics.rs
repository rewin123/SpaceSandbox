use std::sync::Arc;

use bevy::{prelude::*, math::{DVec3, DQuat}};
use bevy_transform64::prelude::*;
use rand::Rng;
use space_physics::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DTransformPlugin)
        .add_plugin(bevy_egui::EguiPlugin)
        .add_plugin(SpacePhysicsPlugin)

        .add_startup_system(setup)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(GlobalGravity {
        gravity: DVec3::new(0.0, -9.81, 0.0),
    });

    // Add a cuboid plane with a collider
    let plane_mesh = meshes.add(Mesh::from(bevy::prelude::shape::Box::new(10.0, 0.1, 10.0)));
    let plane_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    commands.spawn(PbrBundle {
        mesh: plane_mesh,
        material: plane_material,
        ..Default::default()
    })
    .insert(SpaceCollider {
        collider: ColliderBuilder::cuboid(5.0, 0.05, 5.0).build(),
    });

    // Add ten random posed cubes with colliders
    let cube_mesh = meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 1.0 }));
    let cube_material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    let cube_collider = ColliderBuilder::cuboid(0.5, 0.5, 0.5).build();
    let sphere_mesh = meshes.add(Mesh::from(bevy::prelude::shape::UVSphere { radius: 0.5, sectors: 32, stacks : 32 }));
    let mut rng = rand::thread_rng();
    for i in 0..10 {
        let x = rng.gen_range(-5.0..5.0);
        let y = rng.gen_range(0.5..50.0);
        let z = rng.gen_range(-5.0..5.0);
        let cube_transform = DTransform::from_xyz(x, y, z);
        let cube_rigid_body = RigidBodyBuilder::dynamic()
            .translation(DVec3::new(x, y, z).into())
            .gravity_scale(1.0)
            .can_sleep(true)
            .build();
        commands.spawn(PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            ..Default::default()
        })
        .insert(DTransformBundle::from_transform(cube_transform))
        .insert(SpaceCollider {
            collider: cube_collider.clone(),
        })
        .insert(SpaceRigidBody {
            rigid_body: cube_rigid_body,
        });
    }

    for i in 0..10 {
        let x = rng.gen_range(-5.0..5.0);
        let y = rng.gen_range(0.5..5.0);
        let z = rng.gen_range(-5.0..5.0);
        let sphere_transform = DTransform::from_xyz(x, y, z);
        let sphere_rigid_body = RigidBodyBuilder::dynamic()
            .translation(DVec3::new(x, y, z).into())
            .gravity_scale(1.0)
            .can_sleep(true)
            .build();
        commands.spawn(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: cube_material.clone(),
            ..Default::default()
        })
        .insert(DTransformBundle::from_transform(sphere_transform))
        .insert(SpaceCollider {
            collider: ColliderBuilder::ball(0.5).build(),
        })
        .insert(SpaceRigidBody {
            rigid_body: sphere_rigid_body,
        });
    }

    // Add a camera
    commands.spawn(Camera3dBundle {
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
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-2.5),
            ..default()
        },
        ..default()
    }).insert(DTransformBundle::from_transform(DTransform {
        translation: DVec3::new(0.0, 2.0, 0.0),
        rotation: DQuat::from_rotation_x(-2.5),
        ..default()
    }));

}