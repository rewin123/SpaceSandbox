use crate::prelude::*;
use bevy::{prelude::*, math::*};
use bevy_transform64::prelude::*;
use nalgebra as na;


pub fn from_physics_engine(
    mut context : ResMut<RapierContext>,
    mut rigidbodies : Query<(&mut DTransform, &RapierRigidBodyHandle)>,
    mut vels : Query<(&mut Velocity, &RapierRigidBodyHandle)>,
) {
    let context = &mut *context;
    for (mut transform, rigidbody_handle) in rigidbodies.iter_mut() {
        let rigid_body = context.rigid_body_set.get(rigidbody_handle.0).unwrap();
        let pos = rigid_body.position().translation.vector;
        transform.translation = DVec3::new(pos.x, pos.y, pos.z);
        // println!("Pos: {:?}", pos);

        let rot = rigid_body.rotation();
        transform.rotation = DQuat {
            x : rot.i,
            y : rot.j,
            z : rot.k,
            w : rot.w,
        };
    }
    
    for (mut velocity, rigidbody_handle) in vels.iter_mut() {
        let rigid_body = context.rigid_body_set.get(rigidbody_handle.0).unwrap();
        let linvel = rigid_body.linvel();
        let angvel = rigid_body.angvel();
        velocity.linvel = DVec3::new(linvel.x, linvel.y, linvel.z);
        velocity.angvel = DVec3::new(angvel.x, angvel.y, angvel.z);
    }
}
