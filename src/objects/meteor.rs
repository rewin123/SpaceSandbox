use bevy::prelude::*;
use rand::Rng;
use bevy_rapier3d::prelude::*;

use super::radar::RadarDetected;

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
    asset_server : Res<AssetServer>
) {
    let asteroid_scene = asset_server.load("space_objects/asteroid_1.glb#Scene0");
    for event in events.iter() {
        match event {
            MeteorFieldCommand::Spawn => {
                for entity in query.iter() {
                    commands.entity(entity).despawn_recursive();
                }

                //spawn new meteors in 1km radius
                let mut rng = rand::thread_rng();
                let count = 100;
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
                    commands.spawn(SceneBundle {
                        scene : asteroid_scene.clone(),
                        transform: Transform::from_xyz(
                            spawn_pos.x,
                            spawn_pos.y,
                            spawn_pos.z,
                        ).with_scale(Vec3::new(200.0, 200.0,200.0)),
                        ..Default::default()
                    }).insert(Meteor{})
                    .insert(RadarDetected{ color : Color::YELLOW})
                    .insert(Collider::ball(1.0));
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