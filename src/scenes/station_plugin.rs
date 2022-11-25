use std::f32::consts::PI;
use std::os::linux::raw::stat;
use space_assets::{GMesh, Location, LocationInstancing, Material, SubLocation};
use space_core::ecs::*;
use space_core::{nalgebra, Pos3, Vec3, Vec3i};
use space_game::RenderApi;
use space_voxel::objected_voxel_map::VoxelVal;
use space_voxel::solid_voxel_map::VoxelChunk;
use crate::scenes::station_data::*;


pub fn setup_blocks(
    mut cmds : Commands,
    block_holder : Res<BlockHolder>,
    mut station : ResMut<Station>,
    mut events : EventReader<AddBlockEvent>,
    render : Res<RenderApi>) {

    for e in events.iter() {
        // station.add_block_event(
        //     &mut cmds,
        //     e,
        // &mut update_instance_evemts,
        // &block_holder);

        match &e.id {
            BuildCommand::None => {
                let val = station.map.get_cloned(&e.world_pos);
                match &val {
                    StationBlock::None => {}
                    StationBlock::Voxel(_) => {}
                    StationBlock::Object(entity) => {
                        for dz in -1..2 {
                            for dy in -1..2 {
                                for dx in -1..2 {
                                    let test_pos =
                                        e.world_pos
                                            + Vec3::new(dx as f32, dy as f32, dz as f32) * 16.0 * station.map.voxel_size;

                                    if let Some(chunk) = station.map.get_chunk_mut(&test_pos) {
                                        for idx in 0..chunk.data.len() {
                                            if chunk.data[idx] == val {
                                                chunk.data[idx] = StationBlock::None;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        cmds.entity(*entity).despawn();
                    }
                }

            }
            BuildCommand::Block(id) => {
                let bundle = block_holder.map.get(&id).unwrap();

                let mut bbox = bundle.bbox.clone();

                let rot;
                match e.rot {
                    BlockAxis::Y => {
                        rot = Vec3::new(0.0,0.0,0.0);
                    }
                    BlockAxis::X => {
                        rot = Vec3::new(0.0, 0.0, 3.14 / 2.0);
                        bbox = Vec3i::new(bbox.y, bbox.x, bbox.z);
                    }
                    BlockAxis::Z => {
                        rot = Vec3::new(3.14 / 2.0, 0.0, 0.0);
                        bbox = Vec3i::new(bbox.x, bbox.z, bbox.y);
                    }
                }

                let shift = Vec3::new(
                  bbox.x as f32 * station.map.voxel_size / 2.0,
                  bbox.y as f32 * station.map.voxel_size / 2.0,
                  bbox.z as f32 * station.map.voxel_size / 2.0,
                );

                let vp = station.map.get_voxel_pos(&(e.world_pos - shift));

                //test bbox
                let mut is_free = true;
                for z in 0..bbox.z {
                    for y in 0..bbox.y {
                        for x in 0..bbox.x {
                            let pos_i = (vp + Vec3i::new(x, y, z));
                            let pos = Pos3::new(
                                pos_i.x as f32 * station.map.voxel_size,
                                pos_i.y as f32 * station.map.voxel_size,
                                pos_i.z as f32 * station.map.voxel_size,
                            );
                            if station.map.get_cloned(&pos) != StationBlock::None {
                                is_free = false;
                            }
                        }
                    }
                }

                if is_free {
                    let mut loc = Location::new(&render.device);
                    loc.rotation = rot;
                    loc.pos = e.world_pos.coords;
                    let entity = cmds.spawn((bundle.material.clone(), bundle.mesh.clone()))
                        .insert(StationPart { bbox: Default::default() })
                        .insert(loc).id();

                    for z in 0..bbox.z {
                        for y in 0..bbox.y {
                            for x in 0..bbox.x {
                                let pos_i = (vp + Vec3i::new(x, y, z));
                                let pos = Pos3::new(
                                    pos_i.x as f32 * station.map.voxel_size,
                                    pos_i.y as f32 * station.map.voxel_size,
                                    pos_i.z as f32 * station.map.voxel_size,
                                );
                                station.map.set(&pos, VoxelVal::Object(entity.clone()));
                                if station.map.get_cloned(&pos) != StationBlock::None {
                                    is_free = false;
                                }
                            }
                        }
                    }
                }
            }
            BuildCommand::Voxel(id) => {

            }
        }


    }
}

fn collect_sub_locs(
    chunk : &VoxelChunk<StationBlock>,
    id : StationBlock,
    voxel_size : f32
) -> Vec<SubLocation> {
    let mut res = vec![];
    for z in 0..chunk.size.z {
        for y in 0..chunk.size.y {
            for x in 0..chunk.size.x {
                let voxel = chunk.get(x, y, z);

            }
        }
    }
    res
}

pub fn catch_update_events(
    mut cmds : Commands,
    mut station_render : ResMut<StationRender>,
    mut events : EventReader<ChunkUpdateEvent>,
    mut render_events : EventWriter<InstancingUpdateEvent>,
    block_holder : Res<BlockHolder>
) {
    for ev in events.iter() {
        if let Some(chunk) = station_render.instances.get_mut(&ev.origin) {
            // if let Some(inst) = chunk.instance_renders.get(&ev.id) {
            //     render_events.send(InstancingUpdateEvent::Update(
            //         *inst, ev.id.clone(), ev.origin.clone()));
            //
            // } else {
            //     let desc = block_holder.map.get(&ev.id).unwrap();
            //     let inst = cmds.spawn((desc.mesh.clone(), desc.material.clone()))
            //         .insert(LocationInstancing {
            //             locs: vec![],
            //             buffer: None
            //         }).id();
            //     chunk.instance_renders.insert(ev.id.clone(), inst.clone());
            //     render_events.send(InstancingUpdateEvent::Update(
            //         inst.clone(), ev.id.clone(), ev.origin.clone()));
            // }
        } else {
            // let mut chunk = AutoInstanceHolder::default();
            // if let Some(inst) = chunk.instance_renders.get(&ev.id) {
            //     render_events.send(InstancingUpdateEvent::Update(
            //         *inst, ev.id.clone(), ev.origin.clone()));
            //
            // } else {
            //     let desc = block_holder.map.get(&ev.id).unwrap();
            //     let inst = cmds.spawn((desc.mesh.clone(), desc.material.clone()))
            //         .insert(LocationInstancing {
            //             locs: vec![],
            //             buffer: None
            //         }).id();
            //     chunk.instance_renders.insert(ev.id.clone(), inst.clone());
            //     render_events.send(InstancingUpdateEvent::Update(
            //         inst.clone(), ev.id.clone(), ev.origin.clone()));
            // }
            // station_render.instances.insert(ev.origin.clone(), chunk);
        }
    }
}

pub fn update_instancing_holders(
    mut query : Query<&mut LocationInstancing>,
    station : Res<Station>,
    mut events : EventReader<InstancingUpdateEvent>
) {
    for event in events.iter() {
        match event {
            InstancingUpdateEvent::Update(e, id, key) => {
                match query.get_component_mut::<LocationInstancing>(*e) {
                    Ok(mut loc) => {
                        if let Some(chunk) = station.map.get_chunk_by_voxel(&key) {
                            // loc.locs = collect_sub_locs(chunk, *id, station.map.voxel_size);
                        }
                    },
                    Err(_) => {},
                }
            },
        }
    }
}
