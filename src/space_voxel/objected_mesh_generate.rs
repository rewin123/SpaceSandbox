use bevy::reflect::FromReflect;
use bevy::reflect::Typed;
use block_mesh::{
    greedy_quads, GreedyQuadsBuffer, MergeVoxel, Voxel, VoxelVisibility, RIGHT_HANDED_Y_UP_CONFIG,
};
use super::objected_voxel_map::*;
use super::chunked_voxel_map::*;


pub fn generate_mesh<T: PartialEq + Eq + Clone + Typed + FromReflect>(
    chunk: &VoxelChunk<VoxelVal<T>>,
) -> GreedyQuadsBuffer {
    let mut buffer = GreedyQuadsBuffer::new(chunk.data.len());

    let size = [
        chunk.size.x as u32 + 2,
        chunk.size.y as u32 + 2,
        chunk.size.z as u32 + 2,
    ];

    let mut padded_data = vec![VoxelVal::None; (size[0] * size[1] * size[2]) as usize];
    for z in 0..chunk.size.z {
        for y in 0..chunk.size.y {
            for x in 0..chunk.size.x {
                let dz = z + 1;
                let dy = y + 1;
                let dx = x + 1;

                padded_data[((dz as u32 * size[1] + dy as u32) * size[0] + dx as u32) as usize] =
                    chunk.get(x, y, z).clone();
            }
        }
    }

    greedy_quads(
        &padded_data,
        &block_mesh::ndshape::RuntimeShape::<u32, 3>::new(size.clone()),
        [1; 3],
        [
            chunk.size.x as u32 + 1,
            chunk.size.y as u32 + 1,
            chunk.size.z as u32 + 1,
        ],
        &RIGHT_HANDED_Y_UP_CONFIG.faces,
        &mut buffer,
    );
    buffer
}


impl<VoxelID : Typed + FromReflect> Voxel for VoxelVal<VoxelID> {
    fn get_visibility(&self) -> VoxelVisibility {
        match self {
            VoxelVal::None => VoxelVisibility::Empty,
            VoxelVal::Voxel(_) => VoxelVisibility::Opaque,
            VoxelVal::Object(_) => VoxelVisibility::Empty,
        }
    }
}

impl<VoxelID: PartialEq + Eq + Clone + Typed + FromReflect> MergeVoxel for VoxelVal<VoxelID> {
    type MergeValue = Self;

    fn merge_value(&self) -> Self::MergeValue {
        (*self).clone()
    }
}


#[cfg(test)]
mod tests {
    use super::{generate_mesh, VoxelVal};
    use super::super::chunked_voxel_map::VoxelChunk;

    #[test]
    fn greedy_test() {
        let mut chunk = VoxelChunk::<VoxelVal<usize>>::new([0, 0, 0].into(), [10, 10, 10].into());
        chunk.fill(&VoxelVal::Voxel(10));
        let buffer = generate_mesh(&chunk);
        for g in buffer.quads.groups {
            println!("{:?}", g);
        }
    }
}
