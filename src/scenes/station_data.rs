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

#[derive(Default, Hash, Eq, PartialEq, Copy, Clone, Component)]
pub struct WallVoxel {
    pub x : BlockID,
    pub y : BlockID,
    pub z : BlockID
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
    pub map : VoxelMap<WallVoxel>
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

        match event.axis {
            BlockAxis::X => {
                self.map.get_mut(&pos).x = event.id;
            }
            BlockAxis::Y => {
                self.map.get_mut(&pos).y = event.id;
            }
            BlockAxis::Z => {
                self.map.get_mut(&pos).z = event.id;
            }
        }
        // self.map.get_mut(&pos).y = event.id;

        if event.id != BlockID::None {
            update_events.send(ChunkUpdateEvent {
                origin,
                id: event.id.clone()
            });
        }

        if old_id.y != event.id && old_id.y != BlockID::None {
            update_events.send(ChunkUpdateEvent {
                origin,
                id: old_id.y
            });
        }

    }
}


#[derive(Clone, Debug, PartialEq)]
pub enum BlockAxis {
    X,
    Y,
    Z
}

impl Default for BlockAxis {
    fn default() -> Self {
        BlockAxis::Y
    }
}

pub struct AddBlockEvent {
    pub id : BlockID,
    pub world_pos : nalgebra::Vector3<f32>,
    pub axis : BlockAxis
}
