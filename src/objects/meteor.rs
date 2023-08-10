use bevy::prelude::*;
use bevy_proto::prelude::{PrototypesMut, ProtoCommands, Schematic, ReflectSchematic};
use rand::Rng;
#[derive(Component, Reflect, Default, Schematic)]
#[reflect(Schematic)]
pub struct Meteor {

}

#[derive(Hash, PartialEq, Eq, Clone, Debug, Event)]
pub enum MeteorFieldCommand {
    Spawn,
    Despawn,
}

pub struct MetorFieldPlugin;

#[derive(Resource, Default)]
struct MeteorFieldState {
    pub protos : Vec<HandleUntyped>
}

impl Plugin for MetorFieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<MeteorFieldCommand>();

        app.insert_resource(MeteorFieldState::default());
        app.add_systems(Update, meteor_field_spawn);
        app.add_systems(Update, proto_loading);
        app.register_type::<Meteor>();
    }
}

fn proto_loading(mut prototypes : PrototypesMut, mut field : ResMut<MeteorFieldState>) {
    prototypes.load("space_objects/asteroid_1.prototype.ron");
}

fn meteor_field_spawn(
    mut commands: Commands,
    mut proto_commands : ProtoCommands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut events : EventReader<MeteorFieldCommand>,
    query : Query<Entity, With<Meteor>>,
    asset_server : Res<AssetServer>
) {
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
                    let mut entity = proto_commands.spawn("Asteroid 1");
                    // entity.entity_commands().insert(
                    //     DSpatialBundle::from_transform(DTransform::from_xyz(
                    //         spawn_pos.x as f64,
                    //         spawn_pos.y as f64,
                    //         spawn_pos.z as f64,
                    //     ).with_scale(DVec3::new(200.0, 200.0,200.0))),
                    // ).insert(Meteor{})
                    // .insert(RadarDetected{ color : Color::YELLOW})
                    // .insert(SpaceCollider(ColliderBuilder::ball(0.5).build()));
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