
use bevy::prelude::*;
use bevy_proto::prelude::{Schematic, ReflectSchematic};

#[derive(Component, Reflect, Default, Schematic)]
#[reflect(Schematic)]
pub struct ShipCamera;

