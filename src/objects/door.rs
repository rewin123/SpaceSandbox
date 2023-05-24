use bevy::prelude::*;
use bevy_proto::prelude::{Schematic, ReflectSchematic};

#[derive(Component, Reflect, FromReflect, Default, Schematic)]
#[reflect(Schematic)]
pub struct Door {
    pub is_open : bool,
    pub opened_pos : Vec3,
    pub closed_pos : Vec3,
}

pub struct DoorPlugin;

impl Plugin for DoorPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Door>();
    }
}

fn init_door(
    mut doors : Query<&mut Door, Added<Door>>
) {
    
}

fn open_door__system(
    mut doors : Query<&mut Door>,
) {

}