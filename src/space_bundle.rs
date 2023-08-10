
use bevy::{prelude::*};
use bevy_transform64::prelude::*;

#[derive(Default, Bundle)]
pub struct DSceneBundle {
    /// Handle to the scene to spawn
    pub scene: Handle<Scene>,
    pub transform: DTransform,
    pub global_transform: DGlobalTransform,
    pub visibility: Visibility,
    pub computed_visibility: ComputedVisibility,
}

