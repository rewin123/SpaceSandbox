use bevy::prelude::*;

pub mod commands;
pub mod components;
pub mod systems;

use systems::*;

#[doc(hidden)]
pub mod prelude {
    #[doc(hidden)]
    pub use crate::{
        commands::BuildChildrenDTransformExt, components::*, DTransformBundle, DTransformPlugin,
    };
}

use prelude::{DTransform, DGlobalTransform};

#[derive(Bundle, Clone, Copy, Debug, Default)]
pub struct DTransformBundle {
    /// The transform of the entity.
    pub local: DTransform,
    /// The global transform of the entity.
    pub global: DGlobalTransform,
}


impl DTransformBundle {
    /// An identity [`TransformBundle`] with no translation, rotation, and a scale of 1 on all axes.
    pub const IDENTITY: Self = DTransformBundle {
        local: DTransform::IDENTITY,
        global: DGlobalTransform::IDENTITY,
    };

    /// Creates a new [`TransformBundle`] from a [`Transform`].
    ///
    /// This initializes [`GlobalTransform`] as identity, to be updated later by the
    /// [`CoreSet::PostUpdate`](crate::CoreSet::PostUpdate) stage.
    #[inline]
    pub const fn from_transform(transform: DTransform) -> Self {
        DTransformBundle {
            local: transform,
            ..Self::IDENTITY
        }
    }
}

impl From<DTransform> for DTransformBundle {
    #[inline]
    fn from(transform: DTransform) -> Self {
        Self::from_transform(transform)
    }
}

/// Set enum for the systems relating to transform propagation
#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub enum DTransformSystem {
    /// Propagates changes in transform to children's [`GlobalTransform`](crate::components::GlobalTransform)
    TransformPropagate,
}

/// The base plugin for handling [`Transform`] components
#[derive(Default)]
pub struct DTransformPlugin;


impl Plugin for DTransformPlugin {
    fn build(&self, app: &mut App) {
        // A set for `propagate_transforms` to mark it as ambiguous with `sync_simple_transforms`.
        // Used instead of the `SystemTypeSet` as that would not allow multiple instances of the system.
        #[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
        struct PropagateTransformsSet;

        app.register_type::<Transform>()
            .register_type::<GlobalTransform>()
            .add_plugin(ValidParentCheckPlugin::<GlobalTransform>::default())
            // add transform systems to startup so the first update is "correct"
            .configure_set(DTransformSystem::TransformPropagate.in_base_set(CoreSet::PostUpdate))
            .configure_set(PropagateTransformsSet.in_set(DTransformSystem::TransformPropagate))
            .edit_schedule(CoreSchedule::Startup, |schedule| {
                schedule.configure_set(
                    DTransformSystem::TransformPropagate.in_base_set(StartupSet::PostStartup),
                );
            })
            .add_startup_systems((
                sync_simple_transforms
                    .in_set(DTransformSystem::TransformPropagate)
                    // FIXME: https://github.com/bevyengine/bevy/issues/4381
                    // These systems cannot access the same entities,
                    // due to subtle query filtering that is not yet correctly computed in the ambiguity detector
                    .ambiguous_with(PropagateTransformsSet),
                propagate_transforms.in_set(PropagateTransformsSet),
            ))
            .add_systems((
                sync_simple_transforms
                    .in_set(DTransformSystem::TransformPropagate)
                    .ambiguous_with(PropagateTransformsSet),
                propagate_transforms.in_set(PropagateTransformsSet),
            ));
    }
}