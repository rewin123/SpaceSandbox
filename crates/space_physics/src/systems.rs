use crate::prelude::*;
use bevy::{prelude::*, math::{DVec3, DQuat}};
use bevy_transform64::prelude::*;
use rapier3d_f64::{prelude::RigidBody, na::Vector3};
use rapier3d_f64::na as na;

pub fn delete_detection(
    mut context : ResMut<RapierContext>,
    mut collider_del : RemovedComponents<RapierColliderHandle>,
    mut rigid_del : RemovedComponents<RapierRigidBodyHandle>
) {
    let context = &mut *context;
    for e in collider_del.iter() {
        if let Some(handle) = context.entity2collider.get(&e) {
            context.collider_set.remove(*handle, &mut context.island_manager, &mut context.rigid_body_set, true);
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
        }
    }
}

pub fn update_context(
    mut context : ResMut<RapierContext>,
    gravity : Res<GlobalGravity>,
    time : Res<Time>,
) {
    context.step(time.delta_seconds() as f64, &gravity.gravity);
    context.propagate_modified_body_positions_to_colliders();
    
}


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

// Define a public function called add_collider that takes in three parameters:
pub fn add_collider(
    mut commands : Commands, // A mutable reference to the command buffer used to create, delete or modify entities
    mut context : ResMut<RapierContext>, // A mutable reference to the RapierContext resource, which stores the physics simulation state
    mut rigidbodies : Query<&RapierRigidBodyHandle>,
    mut added_colliders : Query<(Entity, &SpaceCollider, Option<&RapierRigidBodyHandle>, Option<&Parent>, Option<&DTransform>), (Added<SpaceCollider>, Without<RapierColliderHandle>)>, // A mutable query that finds entities with SpaceCollider components that have not yet been associated with RapierColliderHandles
) {
    // Create a mutable reference to the RapierContext resource
    let context = &mut *context;

    // Loop through each (entity, collider, rigidbody) tuple in the query result
    for (e, collider, rigidbody, parent, transform) in added_colliders.iter() {

        // Create mutable references to the rigid body set and collider set in the RapierContext resource
        let rigid_body_set = &mut context.rigid_body_set;
        let collider_set = &mut context.collider_set;
        
        // If the collider is associated with a rigid body
        if let Some(rigidbody) = rigidbody {
            // Create a RapierColliderHandle that is associated with the collider and rigid body
            let handle = RapierColliderHandle(
                collider_set.insert_with_parent(collider.0.clone(), rigidbody.0.clone(), rigid_body_set),
            );
            context.entity2collider.insert(e, handle.0);
            // Add the RapierColliderHandle component to the entity
            commands.entity(e).insert(handle);
            println!("Create collider with rigidbody");
        } else {
            if let Some(parent) = parent {
                if let Ok(parent_handle) = rigidbodies.get(parent.get()) {
                    if let Some(transform) = transform {
                        let mut col = collider.0.clone();
                        col.set_translation(Vector3::new(transform.translation.x, transform.translation.y, transform.translation.z));
                        let handle = RapierColliderHandle(
                            collider_set.insert_with_parent(col, parent_handle.0.clone(), rigid_body_set),
                        );
                        context.entity2collider.insert(e, handle.0);
                        commands.entity(e).insert(handle);
                        println!("Create collider with rigidbody parent and transform");
                        continue;
                    } else {
                        let handle = RapierColliderHandle(
                            collider_set.insert_with_parent(collider.0.clone(), parent_handle.0.clone(), rigid_body_set),
                        );
                        context.entity2collider.insert(e, handle.0);
                        commands.entity(e).insert(handle);
                        println!("Create collider with rigidbody parent and without transform");
                        continue;
                    }
                }    
            } 

            {
                // Create a RapierColliderHandle that is associated with the collider only
                let handle = RapierColliderHandle(
                    context.collider_set.insert(collider.0.clone()),
                );
                context.entity2collider.insert(e, handle.0);
                // Add the RapierColliderHandle component to the entity
                commands.entity(e).insert(handle);
                println!("Create collider");
            }
        }
    }
}

pub fn detect_position_change(
    mut context : ResMut<RapierContext>,
    mut rigidbodies : Query<(&DTransform, &RapierRigidBodyHandle), Changed<DTransform>>,
    mut colliders : Query<(&DTransform, &RapierColliderHandle), (Changed<DTransform>, Without<RapierRigidBodyHandle>)>
) {
    let context = &mut *context;
    for (transform, rigidbody_handle) in rigidbodies.iter_mut() {
        let mut rigid_body = context.rigid_body_set.get_mut(rigidbody_handle.0).unwrap();
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

    for (transform, collider_handle) in colliders.iter_mut() {
        let collider = context.collider_set.get_mut(collider_handle.0).unwrap();
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