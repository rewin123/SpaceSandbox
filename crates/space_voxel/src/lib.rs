
pub use space_core::*;
use crate::bevy::utils::{HashMap, HashSet};

pub struct VoxelMap<T> {
    pub map : HashMap<Pos3i, VoxelChunk<T>>,
    pub voxel_size : f32,
    pub chunk_size : Vec3i,
    pub dirty_set : HashSet<Pos3i>
}

impl<T> VoxelMap<T>
    where T : Default + Clone {

    pub fn new(voxel_size : f32, chunk_size : Vec3i) -> VoxelMap<T> {
        VoxelMap {
            map : HashMap::new(),
            voxel_size,
            chunk_size,
            dirty_set : HashSet::new()
        }
    }

    pub fn get_voxel_pos(&self, pos : &Pos3) -> Pos3i {
        Pos3i::new(
            (pos.x / self.voxel_size).round() as i32,
            (pos.y / self.voxel_size).round() as i32,
            (pos.z / self.voxel_size).round() as i32,
        )
    }

    pub fn get_grid_pos(&self, pos : &Pos3) -> Pos3 {
        let vp = self.get_voxel_pos(pos);
        Pos3::new(
            vp.x as f32 * self.voxel_size,
            vp.y as f32 * self.voxel_size,
            vp.z as f32 * self.voxel_size,
        )
    }

    pub fn get_origin(&self, pos : &Pos3i) -> Pos3i {
        Pos3i::new(
            (pos.x as f32 / self.chunk_size.x as f32).floor() as i32 * self.chunk_size.x,
            (pos.y as f32 / self.chunk_size.y as f32).floor() as i32 * self.chunk_size.y,
            (pos.z as f32 / self.chunk_size.z as f32).floor() as i32 * self.chunk_size.z,
        )
    }

    pub fn get_chunk_by_voxel(&self, pos : &Pos3i) -> Option<&VoxelChunk<T>> {
        let origin = self.get_origin(&pos);

        if let Some(chunk) = self.map.get(&origin) {
            Some(chunk)
        } else {
            None
        }

    }

    pub fn get_chunk(&self, pos : &Pos3) -> Option<&VoxelChunk<T>> {
        let vp = self.get_voxel_pos(pos);
        let origin = self.get_origin(&vp);

        if let Some(chunk) = self.map.get(&origin) {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn get_chunk_mut(&mut self, pos : &Pos3) -> Option<&mut VoxelChunk<T>> {
        let vp = self.get_voxel_pos(pos);
        let origin = self.get_origin(&vp);

        if let Some(chunk) = self.map.get_mut(&origin) {
            Some(chunk)
        } else {
            None
        }
    }

    pub fn get_cloned(&self, pos : &Pos3) -> T {
        if let Some(chunk) = self.get_chunk(pos) {
            let vp = self.get_voxel_pos(pos) - chunk.origin;
            chunk.get(vp.x, vp.y, vp.z).clone()
        } else {
            T::default()
        }
    }

    pub fn set(&mut self, pos : &Pos3, val : T) {
        let vp = self.get_voxel_pos(pos);
        let origin = self.get_origin(&vp);
        if let Some(chunk) = self.get_chunk_mut(pos) {
            let lp = vp - chunk.origin;
            *chunk.get_mut(lp.x, lp.y, lp.z) = val;
        } else {
            let origin = self.get_origin(&vp);
            let mut chunk =
                VoxelChunk::<T>::new(origin.clone(), self.chunk_size.clone());

            let lp = vp - origin;
            *chunk.get_mut(lp.x, lp.y, lp.z) = val;
            self.map.insert(origin, chunk);
        }
        self.dirty_set.insert(origin);
    }
}

#[cfg(test)]
mod chunk_map_tests {
    use super::*;

    #[test]
    fn get_voxel_pos() {
        let map = VoxelMap::<i32>::new(2.0, [10,10,10].into());
        let pos = map.get_voxel_pos(&[0.0, 2.0, -2.0].into());
        assert_eq!(pos.x, 0);
        assert_eq!(pos.y, 1);
        assert_eq!(pos.z, -1);
    }

    #[test]
    fn get_origin() {
        let map = VoxelMap::<i32>::new(2.0, [10,10,10].into());
        let origin = map.get_origin(&[0, 11, -1].into());
        assert_eq!(origin.x, 0);
        assert_eq!(origin.y, 10);
        assert_eq!(origin.z, -10);
    }

    #[test]
    fn get_chunk() {
        let map = VoxelMap::<i32>::new(2.0, [10,10,10].into());
        assert!(map.get_chunk(&[0.0,0.0,0.0].into()).is_none());
    }

    #[test]
    fn get_set() {
        let mut map = VoxelMap::<i32>::new(2.0, [10,10,10].into());
        let pos = Pos3::new(11.0,10.0,-9.0);
        assert_eq!(map.get_cloned(&pos), 0);

        map.set(&pos, 11);
        assert_eq!(map.get_cloned(&pos), 11);
        assert!(map.dirty_set.contains(&map.get_origin(&map.get_voxel_pos(&pos))));
    }
}

pub struct VoxelChunk<T> {
    pub origin : Pos3i,
    pub size : Vec3i,
    pub data : Vec<T>,
}

impl<T> VoxelChunk<T>
    where T : Default + Clone {

    pub fn new(origin : Pos3i, size : Vec3i) -> VoxelChunk<T> {
        let data = vec![T::default(); (size.x * size.y * size.z) as usize];
        VoxelChunk {
            origin,
            size,
            data,
        }
    }

    pub fn get(&self, x : i32, y : i32, z : i32) -> &T {
        &self.data[((z * self.size.y + y) * self.size.x + x) as usize]
    }

    pub fn get_mut(&mut self, x : i32, y : i32, z : i32) -> &mut T {
        &mut self.data[((z * self.size.y + y) * self.size.x + x) as usize]
    }

    pub fn fill(&mut self, val : &T) {
        for i in 0..self.data.len() {
            self.data[i] = val.clone();
        }
    }
}

#[cfg(test)]
mod chunk_tests {
    use super::*;

    #[test]
    fn get_set_test() {
        let mut chunk =
            VoxelChunk::<i32>::new(
                Pos3i::new(0,0,0),
                Vec3i::new(10,10,10));
        assert_eq!(*chunk.get(5,5,5), 0);

        *chunk.get_mut(5,5,5) = 10;
        assert_eq!(*chunk.get(5,5,5), 10);
    }

    #[test]
    fn fill_test() {
        let mut chunk =
            VoxelChunk::<i32>::new(
                Pos3i::new(0,0,0),
                Vec3i::new(10,10,10));
        assert_eq!(*chunk.get(5,5,5), 0);

        chunk.fill(&11);
        assert_eq!(*chunk.get(5,5,5), 11);
    }
}