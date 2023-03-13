use bevy::prelude::*;
use rapier3d_f64::prelude::*;

#[derive(Component)]
pub struct RapierColliderHandle {
    pub handle : ColliderHandle,
}

#[derive(Component)]
pub struct SpaceCollider {
    pub collider : Collider,
}