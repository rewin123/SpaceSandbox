use crate::prelude::*;
use bevy::{prelude::*, math::*};
use bevy_transform64::prelude::*;
use nalgebra as na;


pub fn update_context(
    mut context : ResMut<RapierContext>,
    gravity : Res<GlobalGravity>,
    time : Res<Time>,
) {
    context.step(time.delta_seconds() as f64, &gravity.gravity);
    context.propagate_modified_body_positions_to_colliders();
    
}


pub fn delete_detection(
    mut context : ResMut<RapierContext>,
    mut collider_del : RemovedComponents<RapierColliderHandle>,
    mut rigid_del : RemovedComponents<RapierRigidBodyHandle>
) {
    let context = &mut *context;
    for e in collider_del.iter() {
        if let Some(handle) = context.entity2collider.get(&e) {
            context.collider_set.remove(*handle, &mut context.island_manager, &mut context.rigid_body_set, true);
            info!("Collider deleted: {:?}", e);
        } else {
            error!("Collider not found: {:?}", e);
        }
    }

    for e in rigid_del.iter() {
        if let Some(handle) = context.entity2rigidbody.get(&e) {
            context.rigid_body_set.remove(
                *handle, 
                &mut context.island_manager, 
                &mut context.collider_set, 
                &mut context.impulse_joint_set,
                &mut context.multibody_joint_set,
                true);
            info!("Rigid body deleted: {:?}", e);
        } else {
            error!("Rigid body not found: {:?}", e);
        }
    }
}


pub fn detect_position_change(
    mut context : ResMut<RapierContext>,
    mut rigidbodies : Query<(&DTransform, &RapierRigidBodyHandle), Changed<DTransform>>,
    mut colliders : Query<(&DGlobalTransform, &RapierColliderHandle, &DTransform), (Changed<DGlobalTransform>, Without<RapierRigidBodyHandle>)>
) {
    let context = &mut *context;
    for (transform, rigidbody_handle) in rigidbodies.iter_mut() {
        let rigid_body = context.rigid_body_set.get_mut(rigidbody_handle.0).unwrap();
        // let transform = transform.compute_transform();
        rigid_body.set_translation(
            na::Vector3::new(
                transform.translation.x, 
                transform.translation.y, 
                transform.translation.z), 
                true);
        
        rigid_body.set_rotation(
             na::Unit::new_normalize(na::Quaternion::new(
                transform.rotation.w, 
                transform.rotation.x, 
                transform.rotation.y, 
                transform.rotation.z)), 
                true);
    }

    for (transform, collider_handle, loc_transform) in colliders.iter_mut() {
        let collider = context.collider_set.get_mut(collider_handle.0).unwrap();
        if collider.parent().is_none() {
            let transform = transform.compute_transform();
            collider.set_translation(
                na::Vector3::new(
                    transform.translation.x, 
                    transform.translation.y, 
                    transform.translation.z));
            
            collider.set_rotation(
                na::Unit::new_normalize(na::Quaternion::new(
                    transform.rotation.w, 
                    transform.rotation.x, 
                    transform.rotation.y, 
                    transform.rotation.z)));
        } else {
            let transform = loc_transform;
            collider.set_translation(
                na::Vector3::new(
                    transform.translation.x, 
                    transform.translation.y, 
                    transform.translation.z));
            
            collider.set_rotation(
                na::Unit::new_normalize(na::Quaternion::new(
                    transform.rotation.w, 
                    transform.rotation.x, 
                    transform.rotation.y, 
                    transform.rotation.z)));
        }
        
    }

}
