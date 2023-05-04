use bevy::{prelude::*, utils::HashMap, math::DVec3};
use bevy_transform64::{DTransformBundle, prelude::DTransform};
use space_physics::prelude::*;
use crate::{space_voxel::objected_voxel_map::*, DSpatialBundle};
use crate::space_voxel::solid_voxel_map::SolidVoxelMap;
use crate::space_voxel::*;
use serde::{Deserialize, Serialize};

use self::common::{AllVoxelInstances, VoxelInstance};

pub mod common;
pub mod save_load;
pub mod instance_rotate;

pub mod prelude {
    pub use super::common::*;
    pub use super::save_load::*;
    pub use super::instance_rotate::*;
    pub use super::*;
}

#[derive(Clone, Reflect, FromReflect, Serialize, Deserialize)]
pub enum ShipBlock {
    None
}

pub const VOXEL_SIZE : f64 = 0.25;

#[derive(Component, Clone)]
pub struct Ship {
    pub map : SolidVoxelMap<VoxelVal<ShipBlock>>
}

impl Default for Ship {
    fn default() -> Self {
        Ship::new_sized([100,100,100].into())
    }
}



pub fn new_default_ship(cmds : &mut Commands) -> Entity {
    cmds.spawn(Ship::new_sized(IVec3::new(100, 100, 100)))
        .insert(DSpatialBundle::from_transform(DTransform::from_xyz(0.0, 0.0, 0.0)))
        .insert(SpaceRigidBodyType::Fixed)
        .insert(GravityScale(0.0))
        .insert(Velocity::default())
        .insert(ExternalImpulse::default())
        .insert(Name::new("Ship"))
        .insert(SpaceDominance(1))
        .id()
}

impl Ship {
    pub fn new_sized(size : IVec3) -> Self {
        let map = SolidVoxelMap::new(DVec3::ZERO, size, VOXEL_SIZE);
        Self {
            map
        }
    }

    pub fn get_grid_idx_by_center(&self, pos : &DVec3, bbox : &IVec3) -> IVec3 {
        let dp = bbox.as_dvec3() / 2.0 * self.map.voxel_size;
        self.map.get_grid_idx(&(*pos - dp))
    }
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Reflect, FromReflect)]
pub struct InstanceId {
    pub template_id : u32,
    pub state_id : u32
}
