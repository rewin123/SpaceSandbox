use std::ops::Add;
use bevy::utils::HashMap;
use space_assets::{GMesh, LocationInstancing, Material, SubLocation};
use space_core::ecs::*;
use space_core::asset::*;
use space_core::app::*;
use space_core::nalgebra;

#[derive(Resource, Default)]
pub struct BlockHolder {
    pub map : HashMap<BlockID, (Handle<GMesh>, Handle<Material>)>
}

#[derive(Hash, Eq, PartialEq, Copy, Clone, Component)]
pub enum BlockID {
    None,
    Id(usize)
}

impl Default for BlockID {
    fn default() -> Self {
        BlockID::None
    }
}

#[derive(Default)]
pub struct AutoInstanceHolder {
    pub instance_renders : HashMap<BlockID, Entity>
}

pub enum InstancingUpdateEvent {
    Update(Entity, BlockID)
}

#[derive(Resource)]
pub struct Station {
    pub chunks : HashMap<nalgebra::Point3<i32>, StationChunk>,
    pub chunks_size : nalgebra::Vector3<i32>,
    pub voxel_size : f32
}

impl Default for Station {
    fn default() -> Self {
        Self {
            chunks : HashMap::new(),
            chunks_size : nalgebra::Vector3::new(16, 16, 16),
            voxel_size : 2.0
        }
    }
}

impl Station {

    pub fn get_grid_pos(
        &self,
        pos : &nalgebra::Point3<f32>
    ) -> nalgebra::Point3<f32> {
        let mut dpos = pos / self.voxel_size;
        nalgebra::Point3::new(
            dpos.x.round(),
            dpos.y.round(),
            dpos.z.round()
        ) * self.voxel_size
    }

    pub fn get_chunk_origin(
        &self,
        pos : &nalgebra::Point3<f32>
    ) -> nalgebra::Point3<i32> {
        let mut dpos = pos / self.voxel_size;
        let mut f_size : nalgebra::Vector3<f32> = nalgebra::Vector3::new(
            self.chunks_size.x as f32,
            self.chunks_size.y as f32,
            self.chunks_size.z as f32,
        );

        let mut chunked_pos = nalgebra::Point3::<f32>::new(
            dpos.x / f_size.x,
            dpos.y / f_size.y,
            dpos.z / f_size.z);

        chunked_pos.x = chunked_pos.x.floor();
        chunked_pos.y = chunked_pos.y.floor();
        chunked_pos.z = chunked_pos.z.floor();

        nalgebra::Point3::new(
            chunked_pos.x as i32 * self.chunks_size.x,
            chunked_pos.y as i32 * self.chunks_size.y,
            chunked_pos.z as i32 * self.chunks_size.z,
        )
    }

    pub fn add_block_event(
        &mut self,
        cmds : &mut Commands,
        event : &AddBlockEvent,
        update_instance_evemts : &mut EventWriter<InstancingUpdateEvent>,
        block_holder : &BlockHolder) {

        let origin = self.get_chunk_origin(
            &nalgebra::Point3::from_slice(event.world_pos.as_slice()));

        if let Some(chunk) = self.chunks.get_mut(&origin) {
            chunk.add_block(cmds, event, update_instance_evemts, block_holder);
        } else {
            let mut chunk = StationChunk::default();
            chunk.origin = origin.clone();
            chunk.add_block(cmds, event, update_instance_evemts, block_holder);
            self.chunks.insert(origin, chunk);
        }
    }
}


#[derive(Resource)]
pub struct StationChunk {
    pub origin : nalgebra::Point3<i32>,
    pub voxel_size : f32,
    pub floors : Vec<BlockID>,
    pub chunk_size : nalgebra::Vector3<i32>,
    pub auto_instance : AutoInstanceHolder
}

impl Default for StationChunk {
    fn default() -> Self {
        Self {
            origin : nalgebra::Point3::default(),
            voxel_size : 2.0,
            floors : vec![BlockID::None; 16 * 16 * 16],
            chunk_size : nalgebra::Vector3::new(16, 16, 16),
            auto_instance : AutoInstanceHolder::default()
        }
    }
}

impl StationChunk {

    pub fn add_block(
        &mut self,
        cmds : &mut Commands,
        e : &AddBlockEvent,
        update_instance_evemts : &mut EventWriter<InstancingUpdateEvent>,
        block_holder : &BlockHolder) {
        let idx = self.get_idx_3d(&nalgebra::Point3::new(
            e.world_pos.x,
            e.world_pos.y,
            e.world_pos.z));
        if idx.x >= 0 && idx.y >= 0 && idx.z >= 0
            && idx.x < self.chunk_size.x && idx.y < self.chunk_size.y && idx.z < self.chunk_size.z {
            let chunk_size = self.chunk_size.clone();
            let old_id = self.floors[((idx.z * chunk_size.y + idx.y) * chunk_size.x + idx.x) as usize];
            self.floors[((idx.z * chunk_size.y + idx.y) * chunk_size.x + idx.x) as usize] = e.id;
            if e.id == BlockID::None {

            } else {
                if let Some(inst) = self.auto_instance.instance_renders.get(&e.id) {
                    update_instance_evemts.send(InstancingUpdateEvent::Update(*inst, e.id));
                } else {
                    let entity = cmds.spawn(block_holder.map[&e.id].clone())
                        .insert(LocationInstancing {
                            locs: vec![],
                            buffer: None,
                        }).id();
                    self.auto_instance.instance_renders.insert(e.id, entity);
                }
            }

            if old_id == BlockID::None {

            } else {
                if let Some(inst) = self.auto_instance.instance_renders.get(&old_id) {
                    update_instance_evemts.send(InstancingUpdateEvent::Update(*inst, old_id));
                } else {

                }
            }
        }
    }

    pub fn get_block_id(&self, x : i32, y : i32, z : i32) -> &BlockID {
        &self.floors[((z * self.chunk_size.z + y) * self.chunk_size.y + x) as usize]
    }

    pub fn collect_sub_locs(&self, id : BlockID) -> Vec<SubLocation> {
        let mut res = vec![];
        for z in 0..self.chunk_size.z {
            for y in 0..self.chunk_size.y {
                for x in 0..self.chunk_size.x {
                    if id == *self.get_block_id(x, y, z) {
                        let mut sub = SubLocation {
                            pos: [0.0, 0.0, 0.0].into(),
                            rotation: [0.0, 0.0, 0.0].into(),
                            scale: [1.0, 1.0, 1.0].into(),
                        };
                        sub.pos = nalgebra::Vector3::new(
                            (x + self.origin) as f32 * self.voxel_size,
                            y as f32 * self.voxel_size,
                            z as f32 * self.voxel_size,
                        );
                        res.push(sub);
                    }
                }
            }
        }
        res
    }

    pub fn get_idx_3d(&self, pos : &nalgebra::Point3<f32>) -> nalgebra::Point3<i32> {
        let origin_world = nalgebra::Vector3::<f32>::new(
            self.origin.x as f32,
            self.origin.y as f32,
            self.origin.z as f32
        ) * self.voxel_size;

        let dp = pos - origin_world;
        if dp.x < 0.0 || dp.y < 0.0 || dp.z < 0.0 {

        }
        let x = (dp.x / self.voxel_size).round() as i32;
        let y = (dp.y / self.voxel_size).round() as i32;
        let z = (dp.z / self.voxel_size).round() as i32;

        nalgebra::Point3::new(x, y, z)
    }
}


pub struct AddBlockEvent {
    pub id : BlockID,
    pub world_pos : nalgebra::Vector3<f32>
}
