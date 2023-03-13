use bevy::{prelude::*, math::DVec3};

#[derive(Resource, Default)]
pub struct GlobalGravity {
    pub gravity: DVec3,
}