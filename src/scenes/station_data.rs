use std::ops::Add;
use bevy::log::info;
use bevy::utils::{HashMap, HashSet};
use space_assets::{GMesh, LocationInstancing, Material, SubLocation};
use space_core::ecs::*;
use space_core::asset::*;
use space_core::app::*;
use space_core::{nalgebra, Pos3, Pos3i, Vec3, Vec3i};
use space_core::nalgebra::{inf, Point3};
use space_voxel::objected_voxel_map::VoxelVal;
use space_voxel::solid_voxel_map::VoxelMap;
use crate::scenes::RonBlockDesc;

pub struct BlockDesc {
    pub mesh : Handle<GMesh>,
    pub material : Handle<Material>,
    pub name : String,
    pub bbox : Vec3i
}

#[derive(Resource, Default)]
pub struct BlockHolder {
    pub map : HashMap<BlockId, BlockDesc>
}

#[derive(Clone, Eq, Hash, PartialEq, Debug)]
pub struct BlockId(pub usize);

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct VoxelId(pub usize);

#[derive(Clone)]
pub enum BuildCommand {
    None,
    Block(BlockId),
    Voxel(VoxelId)
}

impl Default for BuildCommand {
    fn default() -> Self {
        BuildCommand::None
    }
}

pub type StationBlock = VoxelVal<VoxelId>;

#[derive(Component)]
pub struct StationLocation {
    pub pos : Pos3,
    pub rot : Vec3,
    pub id : BlockId
}

#[derive(Component)]
pub struct AutoInstanceLinks {
    pub set : HashSet<Entity>
}

#[derive(Default, Resource)]
pub struct AutoInstanceHolder {
    pub instance_renders : HashMap<BlockId, Entity>
}


pub enum InstancingUpdateEvent {
    Update(Entity, BlockId, Point3<i32>)
}

#[derive(Resource)]
pub struct Station {
    pub map : VoxelMap<StationBlock>
}

#[derive(Resource, Default)]
pub struct StationRender {
    pub instances : HashMap<Pos3i, AutoInstanceHolder>
}

impl Default for Station {
    fn default() -> Self {
        Self {
            map : VoxelMap::new(0.5, [16, 16, 16].into())
        }
    }
}

pub struct AutoInstnacningLinks {
    pub map : HashSet<Entity>
}

pub struct ChunkUpdateEvent {
    pub origin : Pos3i,
    pub id : StationBlock
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
            event.world_pos.coords.as_slice());

        let origin = self.map.get_origin(&self.map.get_voxel_pos(
            &pos));

        info!("Origin: {:?}", &origin);

        let old_id = self.map.get_cloned(&pos);

        // self.map.get_mut(&pos).y = event.id;

        // if event.id != StationBlock::None {
        //     // update_events.send(ChunkUpdateEvent {
        //     //     origin,
        //     //     id: event.id.clone()
        //     // });
        // }

        // if old_id.y != event.id && old_id.y != StationBlock::None {
        //     update_events.send(ChunkUpdateEvent {
        //         origin,
        //         id: old_id.y
        //     });
        // }

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
    pub id : BuildCommand,
    pub world_pos : Pos3,
    pub rot : BlockAxis
}

#[derive(Component)]
pub struct StationPart {
    pub bbox : Vec3i
}

