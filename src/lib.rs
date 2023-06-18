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
use bevy_proto::prelude::{Schematic, ReflectSchematic};
use bevy_transform64::prelude::*;
// use winit::window::Window;

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


#[derive(Clone, Hash, PartialEq, Eq, Debug, States, Default)]
pub enum Gamemode {
    #[default]
    Godmode,
    FPS
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

// #[derive(Debug, Reflect, FromReflect, Schematic, Default)]
// #[reflect(Schematic)]
// pub struct ProroTransform {
//     pub transform: DTransform
// }