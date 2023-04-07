pub mod objected_voxel_map;
pub mod chunked_voxel_map;
pub mod objected_mesh_generate;
pub mod solid_voxel_map;
pub mod voxel_test;

use bevy::{prelude::*, math::DVec3};

pub enum MapBounds {
    Infinity,
    Limited {from : Real, to : Real}
}

pub type Real = DVec3;

pub trait VoxelMap<T>
{
    fn get_grid_pos(&self, pos: &Real) -> Real;
    fn get_grid_idx(&self, pos: &Real) -> IVec3;
    fn get_idx_pos(&self, pos : &IVec3) -> Real;

    fn get_cloned(&self, pos: &Real) -> T;
    fn get(&self, pos: &Real) -> &T;
    fn get_mut(&mut self, pos: &Real) -> Option<&mut T>;
    fn set_voxel(&mut self, pos : &Real, val : T);

    fn get_cloned_by_idx(&self, pos : &IVec3) -> T;
    fn get_by_idx(&self, pos : &IVec3) -> &T;
    fn get_mut_by_idx(&mut self, pos : &IVec3) -> Option<&mut T>;
    fn set_voxel_by_idx(&mut self, pos : &IVec3, val : T);

    fn get_bounds(&self) -> MapBounds;
    fn get_voxel_size(&self) -> f64;

    fn test_default() -> Self;
}