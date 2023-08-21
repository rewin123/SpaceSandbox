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
pub mod physics_sync;

use std::default::Default;

use bevy::prelude::*;
use bevy_transform64::prelude::*;
use bevy_xpbd_3d::prelude::*;
// use winit::window::Window;

pub mod ext {
    pub use bevy::prelude::*;
    pub use bevy::math::{DVec3, DQuat};
    pub use bevy_transform64::prelude::*;
    pub use bevy_proto::prelude::*;
    pub use bevy_egui::*;
    pub use bevy_xpbd_3d::prelude::*;
    pub use space_editor::prelude::*;
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
            .add_plugins(bevy_xpbd_3d::prelude::PhysicsPlugins::default().build().disable::<bevy_xpbd_3d::prelude::SyncPlugin>().add(physics_sync::PhysicsSync::default()))
            .add_plugins(bevy_proto::prelude::ProtoPlugin::default())
            .insert_resource(Msaa::Off)
            .insert_resource(bevy::pbr::DirectionalLightShadowMap { size: 4096 })
            .add_plugins(bevy::core_pipeline::experimental::taa::TemporalAntiAliasPlugin);
    }
}

