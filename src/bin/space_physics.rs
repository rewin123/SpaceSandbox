

use bevy::{prelude::*, math::{DVec3, DQuat}, pbr::{CascadeShadowConfigBuilder, ScreenSpaceAmbientOcclusionBundle, DirectionalLightShadowMap}, core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin}};
use bevy_transform64::prelude::*;
use rand::Rng;
use bevy_xpbd_3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(SpaceSandbox::SpaceExamplePlguin)
        .add_systems(Startup,setup)
        .run();
}



fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // commands.insert_resource(GlobalGravity {
    //     gravity: DVec3::new(0.0, -9.81, 0.0),
    // });

    // Add a cuboid plane with a collider
    let plane_mesh = meshes.add(Mesh::from(bevy::prelude::shape::Box::new(10.0, 0.1, 10.0)));
    let plane_material = materials.add(Color::rgb(0.5, 0.5, 0.5).into());
    commands.spawn(PbrBundle {
        mesh: plane_mesh,
        material: plane_material,
        ..Default::default()
    })
    .insert(Collider::cuboid(10.0, 0.05, 10.0))
    .insert(RigidBody::Static)
    .insert(Position(DVec3::new(0.0, 0.0, 0.0)));

    // Add ten random posed cubes with colliders
    let cube_mesh = meshes.add(Mesh::from(bevy::prelude::shape::Cube { size: 1.0 }));
    let cube_material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());
    let sphere_mesh = meshes.add(Mesh::from(bevy::prelude::shape::UVSphere { radius: 0.5, sectors: 32, stacks : 32 }));
    let mut rng = rand::thread_rng();
    for _i in 0..10 {
        let x = rng.gen_range(-5.0..5.0);
        let y = rng.gen_range(0.5..50.0);
        let z = rng.gen_range(-5.0..5.0);
        let cube_transform = DTransform::from_xyz(x, y, z);
        commands.spawn(PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            ..Default::default()
        })
        .insert(DTransformBundle::from_transform(cube_transform))
        .insert(RigidBody::Dynamic)
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(Position(cube_transform.translation));
    }

    for _i in 0..10 {
        let x = rng.gen_range(-5.0..5.0);
        let y = rng.gen_range(0.5..50.0);
        let z = rng.gen_range(-5.0..5.0);
        let cube_transform = DTransform::from_xyz(x, y, z);
        let parent_id = commands.spawn(PbrBundle {
            mesh: cube_mesh.clone(),
            material: cube_material.clone(),
            ..Default::default()
        })
        .insert(DTransformBundle::from_transform(cube_transform))
        .insert(Collider::cuboid(1.0, 1.0, 1.0))
        .insert(RigidBody::Dynamic)
        .insert(Position(cube_transform.translation)).id();

        //create child 
        {
            let child_transform = DTransform::from_xyz(1.0, 0.0, 0.0);
            let child_id = commands.spawn(PbrBundle {
                mesh: sphere_mesh.clone(),
                material: cube_material.clone(),
                ..Default::default()
            }).insert(DTransformBundle::from_transform(child_transform))
            .insert(Collider::ball(0.5))
            .insert(Position(child_transform.translation)).id();

            commands.entity(parent_id).add_child(child_id);
        }
    }

    for _i in 0..10 {
        let x = rng.gen_range(-5.0..5.0);
        let y = rng.gen_range(0.5..5.0);
        let z = rng.gen_range(-5.0..5.0);
        let sphere_transform = DTransform::from_xyz(x, y, z);
        commands.spawn(PbrBundle {
            mesh: sphere_mesh.clone(),
            material: cube_material.clone(),
            ..Default::default()
        })
        .insert(DTransformBundle::from_transform(sphere_transform))
        .insert(
            Collider::ball(0.5),
        )
        .insert(RigidBody::Dynamic)
        .insert(Position(sphere_transform.translation));
    }

    // Add a camera
    commands.spawn(Camera3dBundle {
        camera : Camera {
            hdr: true,
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(DTransformBundle::from_transform(
        DTransform::from_xyz(5.0, 5.0, 5.0).looking_at(DVec3::ZERO, DVec3::Y),
    ))
    .insert(ScreenSpaceAmbientOcclusionBundle::default())
    .insert(TemporalAntiAliasBundle::default());

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
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 4.0,
            maximum_distance: 100.0,
            ..default()
        }.into(),
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