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

    let size = [chunk.size.x as u32 + 2, chunk.size.y as u32 + 2, chunk.size.z as u32 + 2];

    let mut padded_data = vec![VoxelVal::None; (size[0] * size[1] * size[2]) as usize];
    for z in 0..chunk.size.z {
        for y in 0..chunk.size.y {
            for x in 0..chunk.size.x {
                let dz = z + 1;
                let dy = y + 1;
                let dx = x + 1;

                padded_data[((dz as u32 * size[1] + dy as u32) * size[0] + dx as u32) as usize] = chunk.get(x, y, z).clone();
            }
        }
    }

    greedy_quads(
        &padded_data,
        &block_mesh::ndshape::RuntimeShape::<u32, 3>::new(size.clone()),
        [1; 3],
        [chunk.size.x as u32 + 1, chunk.size.y as u32 + 1, chunk.size.z as u32 + 1],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer
    );
    buffer
}

impl<T> Default for VoxelVal<T> {
    fn default() -> Self {
        VoxelVal::None
    }
}

#[cfg(test)]
mod tests {
    use crate::objected_voxel_map::{generate_mesh, VoxelVal};
    use crate::solid_voxel_map::VoxelChunk;

    #[test]
    fn greedy_test() {
        let mut chunk = VoxelChunk::<VoxelVal<usize>>::new([0,0,0].into(), [10,10,10].into());
        chunk.fill(&VoxelVal::Voxel(10));
        let buffer = generate_mesh(&chunk);
        for g in buffer.quads.groups {
            println!("{:?}", g);
        }
    }
}