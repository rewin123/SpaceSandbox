use crate::prelude::*;
use bevy::prelude::*;
use bevy_transform64::prelude::*;
use nalgebra as na;
use rapier3d_f64::na::Vector3;



pub type AddRigidBody<'a> = (
    Entity,
    &'a DTransform,
    &'a SpaceRigidBodyType,
    Option<&'a mut Velocity>
);

pub fn add_rigidbody(
    mut commands : Commands,
    mut context : ResMut<RapierContext>,
    mut added_rigidbodies : Query<AddRigidBody, (Added<SpaceRigidBodyType>, Without<RapierRigidBodyHandle>)>,
) {
    for (e, transform, body_type, vel) in added_rigidbodies.iter() {
        let mut body = RigidBody::default();
        match body_type {
            SpaceRigidBodyType::Dynamic => {
                body.set_body_type(RigidBodyType::Dynamic, true);
            },
            SpaceRigidBodyType::Fixed => {
                body.set_body_type(RigidBodyType::Fixed, true);
            },
        }
        let mut body_pos = body.position().clone();
        body_pos.translation = na::Vector3::new(transform.translation.x, transform.translation.y, transform.translation.z).into();
        body_pos.rotation = na::Unit::new_normalize(na::Quaternion::new(transform.rotation.w, transform.rotation.x, transform.rotation.y, transform.rotation.z));
        body.set_position(body_pos, true);

        if let Some(vel) = vel {
            body.set_linvel(na::Vector3::new(vel.linvel.x, vel.linvel.y, vel.linvel.z).into(), true);
            body.set_angvel(na::Vector3::new(vel.angvel.x, vel.angvel.y, vel.angvel.z).into(), true);
        }

        let handle = RapierRigidBodyHandle(
            context.rigid_body_set.insert(body));

        context.entity2rigidbody.insert(e, handle.0);

        commands.entity(e).insert(handle);
    }
}

pub fn change_rigidbody_type(
    mut context : ResMut<RapierContext>,
    mut rigidbodies : Query<(&RapierRigidBodyHandle, &SpaceRigidBodyType), Changed<SpaceRigidBodyType>>
) {
    for (handle, body_type) in rigidbodies.iter_mut() {
        let body = context.rigid_body_set.get_mut(handle.0).unwrap();
        match body_type {
            SpaceRigidBodyType::Dynamic => {
                body.set_body_type(RigidBodyType::Dynamic, true);
            },
            SpaceRigidBodyType::Fixed => {
                body.set_body_type(RigidBodyType::Fixed, true);
            },
        }
        info!("Rigidbody type changed to {:?}", body_type);
    }
}


pub fn change_gravity_scale(
    mut context : ResMut<RapierContext>,
    gravity_scale : Query<(&RapierRigidBodyHandle, &GravityScale), Changed<GravityScale>>,
    added_gravity_scale : Query<(&RapierRigidBodyHandle, &GravityScale), Added<GravityScale>>
) {
    for (handle, scale) in gravity_scale.iter() {
        let rigid_body = context.rigid_body_set.get_mut(handle.0).unwrap();
        rigid_body.set_gravity_scale(scale.0, true);
    }

    for (handle, scale) in added_gravity_scale.iter() {
        let rigid_body = context.rigid_body_set.get_mut(handle.0).unwrap();
        rigid_body.set_gravity_scale(scale.0, true);
    }
}


pub fn change_external_impule(
    mut context : ResMut<RapierContext>,
    changed_impulse : Query<(&RapierRigidBodyHandle, &ExternalImpulse), Changed<ExternalImpulse>>
) {
    for (handle, impulse) in changed_impulse.iter() {
        let rigid_body = context.rigid_body_set.get_mut(handle.0).unwrap();
        rigid_body.apply_impulse(Vector3::new(impulse.impulse.x, impulse.impulse.y, impulse.impulse.z), true);
        rigid_body.apply_torque_impulse(Vector3::new(impulse.torque_impulse.x, impulse.torque_impulse.y, impulse.torque_impulse.z), true);
    }
}

pub fn rigidbody_disabled_system(
    mut context : ResMut<RapierContext>,
    mut disabled_rigidbodies : Query<(Entity, &RapierRigidBodyHandle), Added<RigidBodyDisabled>>,
    mut enabled_rigidbodies : RemovedComponents<RigidBodyDisabled>
) {
    let context = &mut context;
    for (e, mut disabled) in disabled_rigidbodies.iter_mut() {
        context.rigid_body_set.get_mut(disabled.0).unwrap().set_enabled(false);
    }

    for e in enabled_rigidbodies.iter() {
        let handle = *context.entity2rigidbody.get(&e).unwrap();
        context.rigid_body_set.get_mut(handle).unwrap().set_enabled(true);
    }
}