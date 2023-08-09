pub mod components;
pub mod systems;
pub mod resources;
pub mod debug_draw;

use bevy::{prelude::*, transform::TransformSystem};
use bevy_transform64::DTransformSystem;
use resources::{RapierContext, GlobalGravity};
use systems::*;

pub mod prelude {
    pub use crate::{
        components::*,
        resources::*,
        SpacePhysicsPlugin,
    };
    pub use rapier3d_f64::prelude::*;
}

pub struct SpacePhysicsPlugin;

/// Set enum for the systems relating to transform propagation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum SpacePhysicSystem {
    CaptureChanges,
    RigidBodyUpdate,
    ColliderUpdate,
    ContextUpdate,
    WriteToWorld
}

impl Plugin for SpacePhysicsPlugin {
    fn build(&self, app: &mut App) {

        app.insert_resource(RapierContext::default());
        app.insert_resource(GlobalGravity::default());

        app.configure_set(PostUpdate, SpacePhysicSystem::CaptureChanges.before(SpacePhysicSystem::RigidBodyUpdate));
        app.configure_set(PostUpdate, SpacePhysicSystem::RigidBodyUpdate
            .before(DTransformSystem::TransformPropagate)
            .before(SpacePhysicSystem::ColliderUpdate));
        app.configure_set(PostUpdate, SpacePhysicSystem::ColliderUpdate.before(SpacePhysicSystem::ContextUpdate));
        app.configure_set(PostUpdate, SpacePhysicSystem::ContextUpdate.before(SpacePhysicSystem::WriteToWorld));
        app.configure_set(PostUpdate, SpacePhysicSystem::WriteToWorld.before(DTransformSystem::TransformPropagate));


        app.add_systems(PostUpdate, (
            add_rigidbody, 
            apply_deferred, 
            delete_detection, 
            change_gravity_scale,
            change_velosity)
            .chain().in_set(SpacePhysicSystem::RigidBodyUpdate));
        app.add_systems(PostUpdate, (
            collider_change_detection, 
            add_collider, 
            apply_deferred).chain().in_set(SpacePhysicSystem::ColliderUpdate));
        
        app.add_system(update_context.in_set(SpacePhysicSystem::ContextUpdate));

        app.add_system(from_physics_engine.in_set(SpacePhysicSystem::WriteToWorld));

        app.add_systems(PostUpdate, (
            detect_position_change,
            change_external_impule,
            rigidbody_disabled_system,
            collider_disabled_system,
            change_rigidbody_type,
            locked_axes_system
        ).in_set(SpacePhysicSystem::CaptureChanges));

        app.add_plugin(debug_draw::SpacePhysicsDebugDrawPlugin);
    }
}