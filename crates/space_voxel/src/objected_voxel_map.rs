use block_mesh::{greedy_quads, GreedyQuadsBuffer, MergeVoxel, RIGHT_HANDED_Y_UP_CONFIG, Voxel, VoxelVisibility};
use space_core::ecs::Entity;
use space_core::Vec3i;
use crate::solid_voxel_map::{VoxelChunk, VoxelMap};

use block_mesh::ndshape::*;

pub enum VoxelFilling {
    Point,
    BBox(Vec3i),
    Map(Vec<bool>, Vec3i)
}

pub trait VoxelObject {

}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum VoxelVal<VoxelID> {
    None,
    Voxel(VoxelID),
    Object(Entity)
}

impl<VoxelID> Voxel for VoxelVal<VoxelID> {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            VoxelVal::None => {VoxelVisibility::Empty}
            VoxelVal::Voxel(_) => {VoxelVisibility::Opaque}
            VoxelVal::Object(_) => {VoxelVisibility::Empty}
        }
    }
}

impl<VoxelID : PartialEq + Eq + Clone> MergeVoxel for VoxelVal<VoxelID> {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        (*self).clone()
    }
}


pub fn generate_mesh<T: PartialEq + Eq + Clone>(chunk : &VoxelChunk<VoxelVal<T>>) -> GreedyQuadsBuffer {
    let mut buffer = GreedyQuadsBuffer::new(chunk.data.len());

    let size = [chunk.size.x as u32, chunk.size.y as u32, chunk.size.z as u32];
    greedy_quads(
        &chunk.data,
        &block_mesh::ndshape::RuntimeShape::<u32, 3>::new(size.clone()),
        [0; 3],
        size.clone(),
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer
    );
    buffer
}

#[cfg(test)]
mod tests {
    use crate::objected_voxel_map::{generate_mesh, VoxelVal};
    use crate::solid_voxel_map::VoxelChunk;

    #[test]
    fn greedy_test() {
        let chunk = VoxelChunk::<VoxelVal<usize>>::new([0,0,0].into(), [10,10,10].into());
        let buffer = generate_mesh(&chunk);
        for v in buffer.quads.into_iter() {

        }
    }
}