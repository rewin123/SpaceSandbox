use bevy::{prelude::*, utils::HashMap};
use crate::space_voxel::objected_voxel_map::*;
use crate::space_voxel::solid_voxel_map::SolidVoxelMap;
use crate::space_voxel::*;
use serde::{Deserialize, Serialize};

use self::common::{AllVoxelInstances, VoxelInstance};

pub mod common;
pub mod save_load;
pub mod instance_rotate;

pub mod prelude {
    pub use super::common::*;
    pub use super::save_load::*;
    pub use super::instance_rotate::*;
    pub use super::*;
}

#[derive(Clone, Reflect, FromReflect, Serialize, Deserialize)]
pub enum ShipBlock {
    None
}

pub const VOXEL_SIZE : f32 = 0.25;

#[derive(Component)]
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
    pub state_id : u32
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


#[derive(Reflect, Component, Default)]
#[reflect(Component)]
pub struct DiskShipBase64 {
    pub data : String
}

#[derive(Serialize, Deserialize)]
pub struct DiskShip {
    pub map : SolidVoxelMap<DiskShipVoxel>,
    pub template_names : HashMap<u32, String>,
    pub states : HashMap<u32, Entity>
}

impl DiskShip {
    pub fn from_ship(ship_id : Entity, world : &World, remap : &HashMap<Entity, Entity>) -> DiskShip {
        let all_instances = world.resource::<AllVoxelInstances>();

        let mut template_names = HashMap::new();
        for inst in &all_instances.configs {
            template_names.insert(inst.instance.common_id, inst.name.clone());
        }

        let ship : &Ship = world.entity(ship_id).get().unwrap();

        let mut map = SolidVoxelMap::<DiskShipVoxel>::new(Vec3::ZERO, ship.map.size, ship.map.voxel_size);
        
        let mut entity_id : HashMap<Entity, u32> = HashMap::new();
        let mut id_indexer = 0;

        let mut states : HashMap<u32, Entity> = HashMap::new();

        for z in 0..map.size.z {
            for y in 0..map.size.y {
                for x in 0..map.size.x {

                    let idx = IVec3::new(x, y, z);
                    let v = ship.map.get_by_idx(&idx);

                    let disk_v =
                    match v {
                        VoxelVal::None => DiskShipVoxel::None,
                        VoxelVal::Voxel(block) => DiskShipVoxel::Voxel(block.clone()),
                        VoxelVal::Object(e) => {
                            let template_id = world.entity(*e)
                                .get::<VoxelInstance>().unwrap()
                                .common_id;

                            if let Some(state_id) = entity_id.get(e) {
                                DiskShipVoxel::Instance(InstanceId {template_id, state_id : *state_id })
                            } else {
                                entity_id.insert(*e, id_indexer);
                                let val = DiskShipVoxel::Instance(InstanceId {template_id, state_id : id_indexer });
                                states.insert(id_indexer, *remap.get(e).unwrap());
                                id_indexer += 1;
                                val
                            }
                        },
                    };
                    map.set_voxel_by_idx(&idx, disk_v);
                }
            }
        }

        DiskShip {
            map,
            template_names,
            states
        }
    }

    pub fn to_base64(&self) -> String {
        let bytes =  bincode::serialize(&self).unwrap();
        let compressed_bytes = snap::raw::Encoder::new().compress_vec(&bytes).unwrap();
        let compressed_bytes = snap::raw::Encoder::new().compress_vec(&compressed_bytes).unwrap();
        let compressed_bytes = snap::raw::Encoder::new().compress_vec(&compressed_bytes).unwrap();
        let base64 = base64::encode(compressed_bytes);
        base64
    }

    pub fn from_base64(text : &String) -> DiskShip {
        let bytes = base64::decode(text).unwrap();
        let decompressed_bytes = snap::raw::Decoder::new().decompress_vec(&bytes).unwrap();
        let decompressed_bytes = snap::raw::Decoder::new().decompress_vec(&decompressed_bytes).unwrap();
        let decompressed_bytes = snap::raw::Decoder::new().decompress_vec(&decompressed_bytes).unwrap();
        bincode::deserialize(&decompressed_bytes).unwrap()
    }
}
