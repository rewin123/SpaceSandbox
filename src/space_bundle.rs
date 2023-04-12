
use bevy::{prelude::*, render::{camera::*, view::*, primitives::Frustum}, core_pipeline::{tonemapping::{Tonemapping, DebandDither}, core_3d::graph}};
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

