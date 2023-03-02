use bevy::prelude::*;
use rand::Rng;
use bevy_rapier3d::prelude::*;

#[derive(Component, Reflect, FromReflect, Default)]
pub struct Meteor {

}

#[derive(Hash, PartialEq, Eq, Clone, Debug)]
pub enum MeteorFieldCommand {
    Spawn,
    Despawn,
}

pub struct MetorFieldPlugin;

impl Plugin for MetorFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MeteorFieldCommand>();

        app.add_system(meteor_field_spawn);
    }
}

fn meteor_field_spawn(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events : EventReader<MeteorFieldCommand>,
    query : Query<Entity, With<Meteor>>,
) {
    for event in events.iter() {
        match event {
            MeteorFieldCommand::Spawn => {
                for entity in query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                let mesh = meshes.add(Mesh::from(shape::Icosphere {
                    radius: 200.0,
                    ..Default::default()
                }));
                let material = materials.add(StandardMaterial {
                    //brown color
                    base_color: Color::rgb(0.5, 0.2, 0.1),
                    ..Default::default()
                });
                //spawn new meteors in 1km radius
                let mut rng = rand::thread_rng();
                let count = 1000;
                let radius = 10000.0;
                let min_dist = 100.0;
                for _ in 0..count {
                    let spawn_pos = Vec3::new(
                        rng.gen_range(-1.0..1.0) * radius,
                        rng.gen_range(-1.0..1.0) * radius,
                        rng.gen_range(-1.0..1.0) * radius,
                    );
                    if spawn_pos.length() < min_dist {
                        continue;
                    }
                    commands.spawn(PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Transform::from_xyz(
                            spawn_pos.x,
                            spawn_pos.y,
                            spawn_pos.z,
                        ),
                        ..Default::default()
                    }).insert(Meteor{})
                    .insert(Collider::ball(200.0));
                }
            }
            MeteorFieldCommand::Despawn => {
                for entity in query.iter() {
                    commands.entity(entity).despawn_recursive();
                }
            }
        }
    }
}