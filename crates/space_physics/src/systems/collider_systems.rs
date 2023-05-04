use crate::prelude::*;
use bevy::prelude::*;
use bevy_transform64::prelude::*;
use nalgebra as na;
use rapier3d_f64::na::Vector3;


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
                        let mut col: Collider = collider.0.clone();
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

pub fn collider_disabled_system(
    mut context : ResMut<RapierContext>,
    mut disbled_colliders : Query<(Entity, &RapierColliderHandle), Added<ColliderDisabled>>,
    mut enabled_colliders : RemovedComponents<ColliderDisabled>
) {
    for (e, handle) in disbled_colliders.iter() {
        context.collider_set.get_mut(handle.0).unwrap().set_enabled(false);
    }

    for e in enabled_colliders.iter() {
        let handle = *context.entity2collider.get(&e).unwrap();
        context.collider_set.get_mut(handle).unwrap().set_enabled(true);
    }
}