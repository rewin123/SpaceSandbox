use crate::prelude::*;
use bevy::{prelude::*, math::{DVec3, DQuat}};
use bevy_transform64::prelude::*;
use rapier3d_f64::prelude::RigidBody;

pub fn update_context(
    mut context : ResMut<RapierContext>,
    gravity : Res<GlobalGravity>,
    time : Res<Time>,
) {
    context.step(time.delta_seconds() as f64, &gravity.gravity);
    context.propagate_modified_body_positions_to_colliders();
    
}

pub fn add_rigidbody(
    mut commands : Commands,
    mut context : ResMut<RapierContext>,
    mut added_rigidbodies : Query<(Entity, &SpaceRigidBody), (Added<SpaceRigidBody>, Without<RapierRigidBodyHandle>)>,
) {
    for (e, body) in added_rigidbodies.iter() {
        let handle = RapierRigidBodyHandle {
            handle : context.rigid_body_set.insert(body.rigid_body.clone()),
        };
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
            let handle = RapierColliderHandle {
                handle : collider_set.insert_with_parent(collider.collider.clone(), rigidbody.handle.clone(), rigid_body_set),
            };
            // Add the RapierColliderHandle component to the entity
            commands.entity(e).insert(handle);
            println!("Create collider with rigidbody");
        } else {
            if let Some(parent) = parent {
                if let Ok(parent_handle) = rigidbodies.get(parent.get()) {
                    if let Some(transform) = transform {
                        let mut col = collider.collider.clone();
                        col.set_translation(transform.translation.into());
                        let handle = RapierColliderHandle {
                            handle : collider_set.insert_with_parent(col, parent_handle.handle.clone(), rigid_body_set),
                        };
                        commands.entity(e).insert(handle);
                        println!("Create collider with rigidbody parent and transform");
                        continue;
                    } else {
                        let handle = RapierColliderHandle {
                            handle : collider_set.insert_with_parent(collider.collider.clone(), parent_handle.handle.clone(), rigid_body_set),
                        };
                        commands.entity(e).insert(handle);
                        println!("Create collider with rigidbody parent and without transform");
                        continue;
                    }
                }    
            } 

            {
                // Create a RapierColliderHandle that is associated with the collider only
                let handle = RapierColliderHandle {
                    handle : context.collider_set.insert(collider.collider.clone()),
                };
                // Add the RapierColliderHandle component to the entity
                commands.entity(e).insert(handle);
                println!("Create collider");
            }
        }
    }
}

// pub fn update_collider(
//     mut commands : Commands,
//     mut context : ResMut<RapierContext>,
//     mut changed_collider : Query<(&mut RapierColliderHandle, &SpaceCollider), (Changed<SpaceCollider>)>,
//     changed_collider_without_handle : Query<(Entity, &SpaceCollider), (Changed<SpaceCollider>, Without<RapierColliderHandle>, Without<RapierRigidBodyHandle>, Without<SpaceRigidBody>)>,
// ) {
//     for (mut handle, collider) in changed_collider.iter_mut() {
//         if let Some(collider_in_set) = context.collider_set.get_mut(handle.handle) {
//             *collider_in_set = collider.collider.clone();
//         } else {
//             handle.handle = context.collider_set.insert(collider.collider.clone());
//         }
//     }

//     for (e, collider) in changed_collider_without_handle.iter() {
//         let handle = RapierColliderHandle {
//             handle : context.collider_set.insert(collider.collider.clone()),
//         };
//         // println!("Create just collider {:?}", e);
//         commands.entity(e).insert(handle);
//     }
// }

// pub fn update_collider_rigidbody(
//     mut commands : Commands,
//     mut context : ResMut<RapierContext>,
//     changed_collider_without_handle : Query<(Entity, &SpaceCollider, &RapierRigidBodyHandle), (Changed<SpaceCollider>, Without<RapierColliderHandle>)>,
// ) {
//     let context = &mut *context;
//     for (e, collider, rigidbody) in changed_collider_without_handle.iter() {
//         let handle = RapierColliderHandle {
//             handle : context.collider_set.insert_with_parent(
//                 collider.collider.clone(), rigidbody.handle, &mut context.rigid_body_set)
//         };
//         // println!("Create collider with rigidbody {:?}", e);
//         commands.entity(e).insert(handle);
//     }
// }

// pub fn update_rigidbody(
//     mut commands : Commands,
//     mut context : ResMut<RapierContext>,
//     mut changed_rigidbody : Query<(Entity, &SpaceRigidBody, &mut RapierRigidBodyHandle, Option<&SpaceCollider>), (Changed<SpaceRigidBody>)>,
// ) {
//     let context = &mut *context;
//     for (e, rigidbody, mut handle, collider) in changed_rigidbody.iter_mut() {
//         // if let Some(mut handle) = handle {
//             if let Some(rapier_rigidbody) = context.rigid_body_set.get_mut(handle.handle) {
//                 *rapier_rigidbody = rigidbody.rigid_body.clone();
//             } else {
//                 handle.handle = context.rigid_body_set.insert(rigidbody.rigid_body.clone());
//             }
            
//         // } else {
//         //     let handle = context.rigid_body_set.insert(rigidbody.rigid_body.clone());
//         //     let comp = RapierRigidBodyHandle {
//         //         handle : handle.clone()
//         //     };
//         //     if let Some(collider) = collider {
//         //         let collider_handle = RapierColliderHandle {
//         //             handle : context.collider_set.insert_with_parent(
//         //                 collider.collider.clone(), handle, &mut context.rigid_body_set)
//         //         };
//         //         commands.entity(e).insert(collider_handle);
//         //         println!("Create rigidbody with collider {:?}", e);
//         //     };
            
//         //     commands.entity(e).insert(comp);
//         // };
//     }
// }

// pub fn add_rigidbody(
//     mut commands : Commands,
//     mut context : ResMut<RapierContext>,
//     mut added_rigidbody : Query<(Entity, &SpaceRigidBody, &SpaceCollider), (Added<SpaceRigidBody>, Added<SpaceCollider>)>,
// ) {
//     let context = &mut *context;
//     for (e, rigidbody, collider) in added_rigidbody.iter_mut() {
//         let handle = context.rigid_body_set.insert(rigidbody.rigid_body.clone());
//         let collider_handle = context.collider_set.insert_with_parent(
//             collider.collider.clone(), handle, &mut context.rigid_body_set);
//         commands.entity(e).insert(RapierRigidBodyHandle {
//             handle : handle.clone()
//         }).insert(RapierColliderHandle {
//             handle : collider_handle.clone()
//         });

//         println!("Added rigidbody with handle {:?} and collider", handle);
//     }
// }

pub fn from_physics_engine(
    mut context : ResMut<RapierContext>,
    mut rigidbodies : Query<(&mut DTransform, &RapierRigidBodyHandle)>
) {
    let context = &mut *context;
    for (mut transform, rigidbody_handle) in rigidbodies.iter_mut() {
        let rigid_body = context.rigid_body_set.get(rigidbody_handle.handle).unwrap();
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
}