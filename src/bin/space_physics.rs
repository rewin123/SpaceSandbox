use std::sync::Arc;

use bevy::{prelude::*, math::DVec3};
use bevy_transform64::prelude::*;
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
    let cube_mesh = meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 1.0 }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    let sphere_mesh = meshes.add(Mesh::from(bevy::prelude::shape::UVSphere { radius: 1.0, sectors: 32, stacks: 32 }));

    commands.insert_resource(GlobalGravity {
        gravity: DVec3::new(0.0, -9.81, 0.0),
    });

   commands.spawn(PbrBundle {
            mesh: cube_mesh.clone(),
            material: material.clone(),
            visibility : Visibility::Visible,
            ..Default::default()
    }).insert(DTransformBundle::from_transform(DTransform::from_xyz(0.0, 0.0, 0.0)))
    .insert(SpaceCollider{
        collider: ColliderBuilder::cuboid(0.25, 0.25, 0.25).build()
    });

    commands.spawn(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: material.clone(),
            visibility : Visibility::Visible,
            ..Default::default()
    }).insert(DTransformBundle::from_transform(DTransform::from_xyz(0.0, 3.0, 0.0)))
    .insert(SpaceCollider{
        collider: ColliderBuilder::ball(0.4).build()
    })
    .insert(SpaceRigidBody{
        rigid_body: RigidBodyBuilder::dynamic()
            .translation(DVec3::new(0.0, 3.0, 0.0).into())
            .gravity_scale(1.0)
            .can_sleep(false)
            .build()
    });

    commands.spawn(Camera3dBundle {
            ..Default::default()
    }).insert(DTransformBundle::from_transform(DTransform::from_xyz(5.0, 0.0, 5.0).looking_at(DVec3::ZERO, DVec3::Y)));


}