
use bevy::prelude::*;
use bevy_proto::prelude::{Schematic, ReflectSchematic};

#[derive(Component, Reflect, FromReflect, Default, Schematic)]
#[reflect(Schematic)]
pub struct ShipCamera;

