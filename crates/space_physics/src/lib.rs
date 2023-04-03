pub mod components;
pub mod systems;
pub mod resources;
pub mod debug_draw;

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
    RigidBodyUpdate,
    ColliderUpdate,
    ContextUpdate,
    WriteToWorld
}

impl Plugin for SpacePhysicsPlugin {
    fn build(&self, app: &mut App) {

        app.insert_resource(RapierContext::default());
        app.insert_resource(GlobalGravity::default());

        app.configure_set(SpacePhysicSystem::RigidBodyUpdate.before(SpacePhysicSystem::ColliderUpdate).in_base_set(CoreSet::PostUpdate));
        app.configure_set(SpacePhysicSystem::ColliderUpdate.before(SpacePhysicSystem::ContextUpdate).in_base_set(CoreSet::PostUpdate));
        app.configure_set(SpacePhysicSystem::ContextUpdate.in_base_set(CoreSet::PostUpdate));
        app.configure_set(SpacePhysicSystem::WriteToWorld.after(SpacePhysicSystem::ContextUpdate).before(DTransformSystem::TransformPropagate).in_base_set(CoreSet::PostUpdate));


        app.add_systems((add_rigidbody, apply_system_buffers).chain().in_set(SpacePhysicSystem::RigidBodyUpdate));
        app.add_systems((add_collider, apply_system_buffers).chain().in_set(SpacePhysicSystem::ColliderUpdate));
        
        app.add_system(update_context.in_set(SpacePhysicSystem::ContextUpdate));

        app.add_system(from_physics_engine.in_set(SpacePhysicSystem::WriteToWorld));

        app.add_plugin(debug_draw::SpacePhysicsDebugDrawPlugin);
    }
}