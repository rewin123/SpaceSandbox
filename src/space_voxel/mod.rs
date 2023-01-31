pub mod objected_voxel_map;
pub mod chunked_voxel_map;
pub mod objected_mesh_generate;
pub mod solid_voxel_map;
pub mod voxel_test;

use bevy::prelude::*;

pub enum MapBounds {
    Infinity,
    Limited {from : Vec3, to : Vec3}
}

pub trait VoxelMap<T>
{
    fn get_grid_pos(&self, pos: &Vec3) -> Vec3;
    fn get_grid_idx(&self, pos: &Vec3) -> IVec3;
    fn get_idx_pos(&self, pos : &IVec3) -> Vec3;

    fn get_cloned(&self, pos: &Vec3) -> T;
    fn get(&self, pos: &Vec3) -> &T;
    fn get_mut(&mut self, pos: &Vec3) -> Option<&mut T>;
    fn set(&mut self, pos : &Vec3, val : T);

    fn get_cloned_by_idx(&self, pos : &IVec3) -> T;
    fn get_by_idx(&self, pos : &IVec3) -> &T;
    fn get_mut_by_idx(&mut self, pos : &IVec3) -> Option<&mut T>;
    fn set_by_idx(&mut self, pos : &IVec3, val : T);

    fn get_bounds(&self) -> MapBounds;
    fn get_voxel_size(&self) -> f32;

    fn test_default() -> Self;
}