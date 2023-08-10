// #![feature(test)]

pub mod scenes;
pub mod ship;
pub mod space_voxel;
pub mod pawn_system;
pub mod network;
pub mod asset_utils;
pub mod control;
pub mod objects;
pub mod space_bundle;
pub mod editor;
pub mod mission;
use std::default::Default;

use bevy::prelude::*;
use bevy_proto::prelude::{Schematic};
use bevy_transform64::prelude::*;
use bevy_xpbd_3d::prelude::*;
// use winit::window::Window;

pub mod ext {
    pub use bevy::prelude::*;
    pub use bevy_transform64::prelude::*;
    pub use bevy_proto::prelude::*;
    pub use bevy_egui::*;
    pub use bevy_xpbd_3d::prelude::*;
}

pub mod prelude {
    pub use bevy::prelude::*;
    pub use crate::*;
    pub use space_bundle::*;
}

#[derive(Clone, Hash, PartialEq, Eq, Debug, States, Default)]
pub enum SceneType {
    #[default]
    MainMenu,
    ShipBuilding,
    AssetEditor
}



#[derive(Bundle, Debug, Default)]
pub struct DSpatialBundle {
    /// The visibility of the entity.
    pub visibility: Visibility,
    /// The computed visibility of the entity.
    pub computed: ComputedVisibility,
    /// The transform of the entity.
    pub transform: DTransform,
    /// The global transform of the entity.
    pub global_transform: DGlobalTransform,
}

impl DSpatialBundle {
    pub fn from_transform(transform: DTransform) -> Self {
        let global_transform = DGlobalTransform::from(transform);
        Self {
            visibility: Visibility::default(),
            computed: ComputedVisibility::default(),
            transform,
            global_transform
        }
    }
}

pub struct SpaceExamplePlguin;

impl Plugin for SpaceExamplePlguin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DefaultPlugins.build().disable::<TransformPlugin>())
            .add_plugins(DTransformPlugin)
            .add_plugins(bevy_egui::EguiPlugin)
            .add_plugins(bevy_xpbd_3d::prelude::PhysicsPlugins::default().build().disable::<bevy_xpbd_3d::prelude::SyncPlugin>().add(PhysicsSync))
            .insert_resource(Msaa::Off)
            .insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4096 })
            .add_plugins(bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin);
    }
}


struct PhysicsSync;

impl Plugin for PhysicsSync {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, position_to_transform.in_set(PhysicsSet::Sync));   
    }
}


type PosToTransformComponents = (
    &'static mut DTransform,
    &'static Position,
    &'static bevy_xpbd_3d::prelude::Rotation,
    Option<&'static Parent>,
);

type PosToTransformFilter = Or<(Changed<Position>, Changed<bevy_xpbd_3d::prelude::Rotation>)>;

type ParentComponents = (
    &'static DGlobalTransform,
    Option<&'static Position>,
    Option<&'static bevy_xpbd_3d::prelude::Rotation>,
);

// fn transform_to_position()

fn position_to_transform(
    mut query: Query<PosToTransformComponents, PosToTransformFilter>,
    parents: Query<ParentComponents, With<Children>>,
) {
    for (mut transform, pos, rot, parent) in &mut query {
        if let Some(parent) = parent {
            if let Ok((parent_transform, parent_pos, parent_rot)) = parents.get(**parent) {
                // Compute the global transform of the parent using its Position and Rotation
                let parent_transform = parent_transform.compute_transform();
                let parent_pos =
                    parent_pos.map_or(parent_transform.translation, |pos| pos.0);
                let parent_rot = parent_rot.map_or(parent_transform.rotation, |rot| rot.0);
                let parent_scale = parent_transform.scale;
                let parent_transform = DTransform::from_translation(parent_pos)
                    .with_rotation(parent_rot)
                    .with_scale(parent_scale);

                // The new local transform of the child body,
                // computed from the its global transform and its parents global transform
                let new_transform = DGlobalTransform::from(
                    DTransform::from_translation(pos.0).with_rotation(rot.0),
                )
                .reparented_to(&DGlobalTransform::from(parent_transform));

                transform.translation = new_transform.translation;
                transform.rotation = new_transform.rotation;
            }
        } else {
            transform.translation = pos.0;
            transform.rotation = rot.0;
        }
    }
}