use std::f32::consts::PI;
use space_assets::{GMesh, LocationInstancing, Material, SubLocation};
use space_core::ecs::*;
use space_core::nalgebra;
use space_voxel::VoxelChunk;
use crate::scenes::station_data::*;


pub fn setup_blocks(
    mut cmds : Commands,
    block_holder : Res<BlockHolder>,
    mut station : ResMut<Station>,
    mut events : EventReader<AddBlockEvent>,
    mut update_instance_evemts : EventWriter<ChunkUpdateEvent>) {

    for e in events.iter() {
        station.add_block_event(
            &mut cmds,
            e,
        &mut update_instance_evemts,
        &block_holder);

    }
}

fn collect_sub_locs(
    chunk : &VoxelChunk<WallVoxel>,
    id : BlockID,
    voxel_size : f32
) -> Vec<SubLocation> {
    let mut res = vec![];
    for z in 0..chunk.size.z {
        for y in 0..chunk.size.y {
            for x in 0..chunk.size.x {
                let voxel = chunk.get(x, y, z);

                if id == voxel.y {
                    let mut sub = SubLocation {
                        pos: [0.0, 0.0, 0.0].into(),
                        rotation: [0.0, 0.0, 0.0].into(),
                        scale: [1.0, 1.0, 1.0].into(),
                    };
                    sub.pos = nalgebra::Vector3::new(
                        (x + chunk.origin.x) as f32 * voxel_size,
                        (y + chunk.origin.y) as f32 * voxel_size,
                        (z + chunk.origin.z) as f32 * voxel_size,
                    );
                    res.push(sub);
                }
                if id == voxel.x {
                    let mut sub = SubLocation {
                        pos: [0.0, 0.0, 0.0].into(),
                        rotation: [0.0, 0.0, PI / 2.0].into(),
                        scale: [1.0, 1.0, 1.0].into(),
                    };
                    sub.pos = nalgebra::Vector3::new(
                        (x + chunk.origin.x) as f32 * voxel_size - voxel_size / 2.0,
                        (y + chunk.origin.y) as f32 * voxel_size + voxel_size / 2.0,
                        (z + chunk.origin.z) as f32 * voxel_size,
                    );
                    res.push(sub);
                }
                if id == voxel.z {
                    let mut sub = SubLocation {
                        pos: [0.0, 0.0, 0.0].into(),
                        rotation: [PI / 2.0, 0.0, 0.0].into(),
                        scale: [1.0, 1.0, 1.0].into(),
                    };
                    sub.pos = nalgebra::Vector3::new(
                        (x + chunk.origin.x) as f32 * voxel_size,
                        (y + chunk.origin.y) as f32 * voxel_size + voxel_size / 2.0,
                        (z + chunk.origin.z) as f32 * voxel_size - voxel_size / 2.0,
                    );
                    res.push(sub);
                }
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
            if let Some(inst) = chunk.instance_renders.get(&ev.id) {
                render_events.send(InstancingUpdateEvent::Update(
                    *inst, ev.id.clone(), ev.origin.clone()));

            } else {
                let desc = block_holder.map.get(&ev.id).unwrap();
                let inst = cmds.spawn((desc.mesh.clone(), desc.material.clone()))
                    .insert(LocationInstancing {
                        locs: vec![],
                        buffer: None
                    }).id();
                chunk.instance_renders.insert(ev.id.clone(), inst.clone());
                render_events.send(InstancingUpdateEvent::Update(
                    inst.clone(), ev.id.clone(), ev.origin.clone()));
            }
        } else {
            let mut chunk = AutoInstanceHolder::default();
            if let Some(inst) = chunk.instance_renders.get(&ev.id) {
                render_events.send(InstancingUpdateEvent::Update(
                    *inst, ev.id.clone(), ev.origin.clone()));

            } else {
                let desc = block_holder.map.get(&ev.id).unwrap();
                let inst = cmds.spawn((desc.mesh.clone(), desc.material.clone()))
                    .insert(LocationInstancing {
                        locs: vec![],
                        buffer: None
                    }).id();
                chunk.instance_renders.insert(ev.id.clone(), inst.clone());
                render_events.send(InstancingUpdateEvent::Update(
                    inst.clone(), ev.id.clone(), ev.origin.clone()));
            }
            station_render.instances.insert(ev.origin.clone(), chunk);
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
                            loc.locs = collect_sub_locs(chunk, *id, station.map.voxel_size);
                        }
                    },
                    Err(_) => {},
                }
            },
        }
    }
}
