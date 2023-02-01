use bevy::prelude::*;
use crate::space_voxel::objected_voxel_map::*;
use crate::space_voxel::solid_voxel_map::SolidVoxelMap;
use crate::space_voxel::*;

pub mod common;

#[derive(Clone)]
pub enum ShipBlock {
    None
}

pub type ShipVoxel = VoxelVal<ShipBlock>;

pub const VOXEL_SIZE : f32 = 0.5;

#[derive(Component)]
pub struct Ship {
    pub map : SolidVoxelMap<ShipVoxel>
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