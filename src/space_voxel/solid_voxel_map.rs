
use super::*;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Component, Clone)]
pub struct SolidVoxelMap<T> where T : Default + Clone {
    pub data : Vec<T>,
    pub size : IVec3,
    pub first_voxel_pos : Real,
    pub voxel_size : f64,
    pub dummpy : T
}

impl<T> SolidVoxelMap<T>
    where T : Default + Clone
{
    pub fn new(origin : Real, size : IVec3, voxel_size : f64) -> SolidVoxelMap<T> {
        let data = vec![T::default(); (size.x * size.y * size.z) as usize];
        let first_voxel_pos = origin - size.as_dvec3() / 2.0 * voxel_size;

        SolidVoxelMap {
            data,
            first_voxel_pos,
            size,
            voxel_size,
            dummpy : T::default()
        }
    }
}

impl<T> SolidVoxelMap<T>
    where T : Default + Clone
{
    #[inline]
    fn get_idx(&self, pos : &IVec3) -> usize {
        let idx = (pos.z * self.size.y + pos.y) * self.size.x + pos.x;
        if idx >= 0 {
            idx as usize
        } else {
            self.data.len()
        }
    }

    #[inline]
    fn get_line(&self, z : i32, y : i32) -> &[T] {
        let start_idx = (z * self.size.y + y * self.size.x) as usize;

        &self.data[start_idx..(start_idx + self.size.x as usize)]
    }

    #[inline]
    fn get_line_mut(&mut self, z : i32, y : i32) -> &mut [T] {
        let start_idx = (z * self.size.y + y * self.size.x) as usize;

        &mut self.data[start_idx..(start_idx + self.size.x as usize)]
    }
}

impl<T> VoxelMap<T> for SolidVoxelMap<T>
    where T : Default + Clone
{
    fn get_grid_pos(&self, pos: &Real) -> Real {
        let dp = *pos - self.first_voxel_pos;
        let dp_round = (dp / self.voxel_size).floor();
        dp_round * self.voxel_size + self.first_voxel_pos
    }

    fn get_grid_idx(&self, pos: &Real) -> IVec3 {
        let dp = *pos - self.first_voxel_pos;
        let dp_round = (dp / self.voxel_size).floor();
        dp_round.as_ivec3()
    }

    fn get_idx_pos(&self, pos : &IVec3) -> Real {
        pos.as_dvec3() * self.voxel_size + self.first_voxel_pos
    }

    fn get_cloned(&self, pos: &Real) -> T {
        let vec_idx = self.get_grid_idx(pos);
        let idx = self.get_idx(&vec_idx);
        if idx < self.data.len() {
            self.data[idx].clone()
        } else {
            T::default()
        }
    }

    fn get(&self, pos: &Real) -> &T {
        let vec_idx = self.get_grid_idx(pos);
        let idx = self.get_idx(&vec_idx);
        self.data.get(idx).unwrap_or(&self.dummpy)
    }

    fn get_mut(&mut self, pos: &Real) -> Option<&mut T> {
        let vec_idx = self.get_grid_idx(pos);
        let idx = self.get_idx(&vec_idx);
        self.data.get_mut(idx)
    }

    fn set_voxel(&mut self, pos : &Real, val : T) {
        let vec_idx = self.get_grid_idx(pos);
        let idx = self.get_idx(&vec_idx);
        if idx < self.data.len() {
            self.data[idx] = val;
        }
    }

    fn get_cloned_by_idx(&self, pos : &IVec3) -> T {
        let idx = self.get_idx(pos);
        if idx < self.data.len() {
            self.data[idx].clone()
        } else {
            T::default()
        }
    }

    fn get_by_idx(&self, pos : &IVec3) -> &T {
        let idx = self.get_idx(pos);
        self.data.get(idx).unwrap_or(&self.dummpy)
    }

    fn get_mut_by_idx(&mut self, pos : &IVec3) -> Option<&mut T> {
        let idx = self.get_idx(pos);
        self.data.get_mut(idx)
    }

    fn set_voxel_by_idx(&mut self, pos : &IVec3, val : T) {
        let idx = self.get_idx(pos);
        if idx < self.data.len() {
            self.data[idx] = val;
        }
    }

    fn get_bounds(&self) -> MapBounds {
        MapBounds::Limited { 
            from: self.first_voxel_pos, 
            to: self.first_voxel_pos + self.get_voxel_size() * self.size.as_dvec3() 
        }
    }

    fn get_voxel_size(&self) -> f64 {
        self.voxel_size
    }

    fn test_default() -> Self {
        SolidVoxelMap::new(Real::new(0.0,0.0,0.0), IVec3::new(100,100,100), 0.5)
    }
}