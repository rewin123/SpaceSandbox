use std::sync::Arc;

use bevy::{prelude::*, math::{DVec3, DQuat}, pbr::{CascadeShadowConfigBuilder, ScreenSpaceAmbientOcclusionBundle, DirectionalLightShadowMap}, core_pipeline::experimental::taa::{TemporalAntiAliasBundle, TemporalAntiAliasPlugin}};
use bevy_transform64::prelude::*;
use rand::Rng;
use bevy_xpbd_3d::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.build().disable::<TransformPlugin>())
        .add_plugins(DTransformPlugin)
        .add_plugins(bevy_egui::EguiPlugin)
        .add_plugins(PhysicsPlugins::default().build().disable::<SyncPlugin>().add(PhysicsSync))
        .insert_resource(Msaa::Off)
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(TemporalAntiAliasPlugin)
        .add_systems(Startup,setup)
        .run();
}


struct PhysicsSync;

impl Plugin for PhysicsSync {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, position_to_transform.in_set(PhysicsSet::Sync));   
    }
}


type PosToTransformComponents = (
    &'static mut DTransform,
    &'static Position,
    &'static bevy_xpbd_3d::prelude::Rotation,
    Option<&'static Parent>,
);

type PosToTransformFilter = Or<(Changed<Position>, Changed<bevy_xpbd_3d::prelude::Rotation>)>;

type ParentComponents = (
    &'static DGlobalTransform,
    Option<&'static Position>,
    Option<&'static bevy_xpbd_3d::prelude::Rotation>,
);

fn position_to_transform(
    mut query: Query<PosToTransformComponents, PosToTransformFilter>,
    parents: Query<ParentComponents, With<Children>>,
) {
    for (mut transform, pos, rot, parent) in &mut query {
        if let Some(parent) = parent {
            if let Ok((parent_transform, parent_pos, parent_rot)) = parents.get(**parent) {
                // Compute the global transform of the parent using its Position and Rotation
                let parent_transform = parent_transform.compute_transform();
                let parent_pos =
                    parent_pos.map_or(parent_transform.translation, |pos| pos.0.clone());
                let parent_rot = parent_rot.map_or(parent_transform.rotation, |rot| rot.0.clone());
                let parent_scale = parent_transform.scale;
                let parent_transform = DTransform::from_translation(parent_pos)
                    .with_rotation(parent_rot)
                    .with_scale(parent_scale);

                // The new local transform of the child body,
                // computed from the its global transform and its parents global transform
                let new_transform = DGlobalTransform::from(
                    DTransform::from_translation(pos.0.clone()).with_rotation(rot.0.clone()),
                )
                .reparented_to(&DGlobalTransform::from(parent_transform));

                transform.translation = new_transform.translation;
                transform.rotation = new_transform.rotation;
            }
        } else {
            transform.translation = pos.0.clone();
            transform.rotation = rot.0.clone();
        }
    }
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
    for i in 0..10 {
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

    for i in 0..10 {
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

    for i in 0..10 {
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