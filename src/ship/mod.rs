use bevy::{prelude::*, utils::HashMap};
use crate::space_voxel::objected_voxel_map::*;
use crate::space_voxel::solid_voxel_map::SolidVoxelMap;
use crate::space_voxel::*;
use serde::{Deserialize, Serialize};

use self::common::AllVoxelInstances;

pub mod common;

#[derive(Clone, Reflect, FromReflect, Serialize, Deserialize)]
pub enum ShipBlock {
    None
}

pub const VOXEL_SIZE : f32 = 0.5;

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct Ship {
    pub map : SolidVoxelMap<VoxelVal<ShipBlock>>
}

impl Default for Ship {
    fn default() -> Self {
        Ship::new_sized([100,100,100].into())
    }
}

impl Ship {
    pub fn new_sized(size : IVec3) -> Self {
        let map = SolidVoxelMap::new(Vec3::ZERO, size, VOXEL_SIZE);
        Self {
            map
        }
    }

    pub fn get_grid_idx_by_center(&self, pos : &Vec3, bbox : &IVec3) -> IVec3 {
        let dp = bbox.as_vec3() / 2.0 * self.map.voxel_size;
        self.map.get_grid_idx(&(*pos - dp))
    }

}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq, Reflect, FromReflect)]
pub struct InstanceId {
    pub template_id : u32,
    pub state_id : Entity
}

#[derive(Serialize, Deserialize, Clone, Reflect, FromReflect)]
pub enum DiskShipVoxel {
    None, 
    Voxel(ShipBlock),
    Instance(InstanceId)
}

impl Default for DiskShipVoxel {
    fn default() -> Self {
        DiskShipVoxel::None
    }
}

#[derive(Serialize, Deserialize, Reflect)]
pub struct DiskShip {
    pub map : SolidVoxelMap<DiskShipVoxel>,
    pub template_names : HashMap<u32, String>,
}

impl DiskShip {
    pub fn from_ship(ship_id : Entity, world : &World) {
        let all_instances = world.resource::<AllVoxelInstances>();
        let template_indexer = 0;
    }
}