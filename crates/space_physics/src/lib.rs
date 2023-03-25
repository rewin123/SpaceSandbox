pub mod components;
pub mod systems;
pub mod resources;

use bevy::prelude::*;
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
    ContextUpdate,
    WriteToWorld
}

impl Plugin for SpacePhysicsPlugin {
    fn build(&self, app: &mut App) {

        app.insert_resource(RapierContext::default());
        app.insert_resource(GlobalGravity::default());

        app.configure_set(SpacePhysicSystem::CaptureChanges.before(SpacePhysicSystem::ContextUpdate).in_base_set(CoreSet::PostUpdate));
        app.configure_set(SpacePhysicSystem::ContextUpdate.in_base_set(CoreSet::PostUpdate));
        app.configure_set(SpacePhysicSystem::WriteToWorld.after(SpacePhysicSystem::ContextUpdate).before(DTransformSystem::TransformPropagate).in_base_set(CoreSet::PostUpdate));

        app.add_system(update_collider.in_set(SpacePhysicSystem::CaptureChanges));
        app.add_system(update_collider_rigidbody.in_set(SpacePhysicSystem::CaptureChanges));
        app.add_system(add_rigidbody.in_set(SpacePhysicSystem::CaptureChanges));
        app.add_system(update_rigidbody.in_set(SpacePhysicSystem::CaptureChanges));
        
        app.add_system(update_context.in_set(SpacePhysicSystem::ContextUpdate));

        app.add_system(from_physics_engine.in_set(SpacePhysicSystem::WriteToWorld));
        

    }
}