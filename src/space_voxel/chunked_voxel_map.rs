use bevy::utils::{HashMap, HashSet};
use bevy::prelude::*;

use super::{VoxelMap, Real};


pub struct ChunkedVoxelMap<T> {
    pub map: HashMap<IVec3, VoxelChunk<T>>,
    pub voxel_size: f64,
    pub chunk_size: IVec3,
    pub dirty_set: HashSet<IVec3>,
}

impl<T> VoxelMap<T> for ChunkedVoxelMap<T> 
    where T : Default + Clone {
    fn get_grid_pos(&self, pos: &Real) -> Real {
        let vp = self.get_voxel_pos(pos);
        Real::new(
            vp.x as f64 * self.voxel_size,
            vp.y as f64 * self.voxel_size,
            vp.z as f64 * self.voxel_size,
        )
    }

    fn get_grid_idx(&self, pos: &Real) -> IVec3 {
        todo!()
    }

    fn get_idx_pos(&self, pos : &IVec3) -> Real {
        todo!()
    }

    fn get_cloned(&self, pos: &Real) -> T {
        todo!()
    }

    fn get(&self, pos: &Real) -> &T {
        todo!()
    }

    fn get_mut(&mut self, pos: &Real) -> Option<&mut T> {
        todo!()
    }

    fn set_voxel(&mut self, pos : &Real, val : T) {
        todo!()
    }

    fn get_cloned_by_idx(&self, pos : &IVec3) -> T {
        todo!()
    }

    fn get_by_idx(&self, pos : &IVec3) -> &T {
        todo!()
    }

    fn get_mut_by_idx(&mut self, pos : &IVec3) -> Option<&mut T> {
        todo!()
    }

    fn set_voxel_by_idx(&mut self, pos : &IVec3, val : T) {
        todo!()
    }

    fn get_bounds(&self) -> super::MapBounds {
        todo!()
    }

    fn get_voxel_size(&self) -> f64 {
        todo!()
    }

    fn test_default() -> Self {
        todo!()
    }
}

impl<T> ChunkedVoxelMap<T>
where
    T: Default + Clone,
{
    pub fn new(voxel_size: f64, chunk_size: IVec3) -> ChunkedVoxelMap<T> {
        ChunkedVoxelMap {
            map: HashMap::new(),
            voxel_size,
            chunk_size,
            dirty_set: HashSet::new(),
        }
    }

    pub fn get_voxel_pos(&self, pos: &Real) -> IVec3 {
        IVec3::new(
            (pos.x / self.voxel_size).round() as i32,
            (pos.y / self.voxel_size).round() as i32,
            (pos.z / self.voxel_size).round() as i32,
        )
    }

    

    pub fn get_origin(&self, pos: &IVec3) -> IVec3 {
        IVec3::new(
            (pos.x as f32 / self.chunk_size.x as f32).floor() as i32 * self.chunk_size.x,
            (pos.y as f32 / self.chunk_size.y as f32).floor() as i32 * self.chunk_size.y,
            (pos.z as f32 / self.chunk_size.z as f32).floor() as i32 * self.chunk_size.z,
        )
    }

    pub fn get_chunk_by_voxel(&self, pos: &IVec3) -> Option<&VoxelChunk<T>> {
        let origin = self.get_origin(&pos);

        if let Some(chunk) = self.map.get(&origin) {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn get_chunk(&self, pos: &Real) -> Option<&VoxelChunk<T>> {
        let vp = self.get_voxel_pos(pos);
        let origin = self.get_origin(&vp);

        if let Some(chunk) = self.map.get(&origin) {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn get_chunk_mut(&mut self, pos: &Real) -> Option<&mut VoxelChunk<T>> {
        let vp = self.get_voxel_pos(pos);
        let origin = self.get_origin(&vp);

        if let Some(chunk) = self.map.get_mut(&origin) {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, pos: &Real) -> &mut T {
        let vp = self.get_voxel_pos(pos);
        let origin = self.get_origin(&vp);
        let chunk_size = self.chunk_size.clone();
        self.dirty_set.insert(origin);

        if !self.map.contains_key(&origin) {
            let mut chunk = VoxelChunk::<T>::new(origin.clone(), chunk_size.clone());

            let lp = vp - origin;
            self.map.insert(origin.clone(), chunk);
        }

        let chunk = self.map.get_mut(&origin).unwrap();
        let lp = vp - chunk.origin;
        chunk.get_mut(lp.x, lp.y, lp.z)
    }

    pub fn get_cloned(&self, pos: &Real) -> T {
        if let Some(chunk) = self.get_chunk(pos) {
            let vp = self.get_voxel_pos(pos) - chunk.origin;
            chunk.get(vp.x, vp.y, vp.z).clone()
        } else {
            T::default()
        }
    }

    pub fn set(&mut self, pos: &Real, val: T) {
        let vp = self.get_voxel_pos(pos);
        let origin = self.get_origin(&vp);
        if let Some(chunk) = self.get_chunk_mut(pos) {
            let lp = vp - chunk.origin;
            *chunk.get_mut(lp.x, lp.y, lp.z) = val;
        } else {
            let origin = self.get_origin(&vp);
            let mut chunk = VoxelChunk::<T>::new(origin.clone(), self.chunk_size.clone());

            let lp = vp - origin;
            *chunk.get_mut(lp.x, lp.y, lp.z) = val;
            self.map.insert(origin, chunk);
        }
        self.dirty_set.insert(origin);
    }
}

pub struct VoxelChunk<T> {
    pub origin: IVec3,
    pub size: IVec3,
    pub data: Vec<T>,
}

impl<T> VoxelChunk<T>
where
    T: Default + Clone,
{
    pub fn new(origin: IVec3, size: IVec3) -> VoxelChunk<T> {
        let data = vec![T::default(); (size.x * size.y * size.z) as usize];
        VoxelChunk { origin, size, data }
    }

    pub fn get(&self, x: i32, y: i32, z: i32) -> &T {
        &self.data[((z * self.size.y + y) * self.size.x + x) as usize]
    }

    pub fn get_mut(&mut self, x: i32, y: i32, z: i32) -> &mut T {
        &mut self.data[((z * self.size.y + y) * self.size.x + x) as usize]
    }

    pub fn fill(&mut self, val: &T) {
        for i in 0..self.data.len() {
            self.data[i] = val.clone();
        }
    }
}