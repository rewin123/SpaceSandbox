use bevy::prelude::*;
use rapier3d_f64::prelude::*;

#[derive(Component)]
pub struct RapierRigidBodyHandle(pub RigidBodyHandle);


#[derive(Component, Debug)]
pub struct SpaceRigidBody(pub RigidBody);
