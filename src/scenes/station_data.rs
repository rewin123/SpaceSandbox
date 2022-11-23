use std::ops::Add;
use bevy::log::info;
use bevy::utils::{HashMap, HashSet};
use space_assets::{GMesh, LocationInstancing, Material, SubLocation};
use space_core::ecs::*;
use space_core::asset::*;
use space_core::app::*;
use space_core::{nalgebra, Pos3i};
use space_core::nalgebra::{inf, Point3};
use space_voxel::VoxelMap;
use crate::scenes::RonBlockDesc;

pub struct BlockDesc {
    pub mesh : Handle<GMesh>,
    pub material : Handle<Material>,
    pub name : String
}

#[derive(Resource, Default)]
pub struct BlockHolder {
    pub map : HashMap<BlockID, BlockDesc>
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
    Update(Entity, BlockID, Point3<i32>)
}

#[derive(Resource)]
pub struct Station {
    pub map : VoxelMap<BlockID>
}

#[derive(Resource, Default)]
pub struct StationRender {
    pub instances : HashMap<Pos3i, AutoInstanceHolder>
}

impl Default for Station {
    fn default() -> Self {
        Self {
            map : VoxelMap::new(2.0, [16, 16, 16].into())
        }
    }
}

pub struct ChunkUpdateEvent {
    pub origin : Pos3i,
    pub id : BlockID
}

impl Station {

    pub fn get_grid_pos(
        &self,
        pos : &nalgebra::Point3<f32>
    ) -> nalgebra::Point3<f32> {
        self.map.get_grid_pos(pos)
    }

    pub fn add_block_event(
        &mut self,
        cmds : &mut Commands,
        event : &AddBlockEvent,
        update_events : &mut EventWriter<ChunkUpdateEvent>,
        block_holder : &BlockHolder) {


        let pos = nalgebra::Point3::from_slice(
            event.world_pos.as_slice());

        let origin = self.map.get_origin(&self.map.get_voxel_pos(
            &pos));

        info!("Origin: {:?}", &origin);


        let old_id = self.map.get_cloned(&pos);
        self.map.set(&pos, event.id);

        if event.id != BlockID::None {
            update_events.send(ChunkUpdateEvent {
                origin,
                id: event.id.clone()
            });
        }

        if old_id != event.id && old_id != BlockID::None {
            update_events.send(ChunkUpdateEvent {
                origin,
                id: old_id
            });
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
                    update_instance_evemts.send(
                        InstancingUpdateEvent::Update(*inst, e.id, self.origin.clone()));
                } else {
                    let desc = &block_holder.map[&e.id];
                    let entity = cmds.spawn((desc.mesh.clone(), desc.material.clone()))
                        .insert(LocationInstancing {
                            locs: vec![],
                            buffer: None,
                        }).id();
                    info!("Spawn new instancing {:?}", &entity);
                    self.auto_instance.instance_renders.insert(e.id, entity);
                }
            }

            if old_id == BlockID::None {

            } else {
                if let Some(inst) = self.auto_instance.instance_renders.get(&old_id) {
                    update_instance_evemts.send(
                        InstancingUpdateEvent::Update(*inst, old_id, self.origin));
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
                            (x + self.origin.x) as f32 * self.voxel_size,
                            (y + self.origin.y) as f32 * self.voxel_size,
                            (z + self.origin.z) as f32 * self.voxel_size,
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

        info!("Idx: {x} {y} {z}");

        nalgebra::Point3::new(x, y, z)
    }
}


pub struct AddBlockEvent {
    pub id : BlockID,
    pub world_pos : nalgebra::Vector3<f32>
}
