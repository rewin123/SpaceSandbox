use space_assets::{GMesh, LocationInstancing, Material};
use space_core::ecs::*;
use space_core::nalgebra;
use crate::scenes::station_data::*;


pub fn setup_blocks(
    mut cmds : Commands,
    block_holder : Res<BlockHolder>,
    mut station : ResMut<Station>,
    mut events : EventReader<AddBlockEvent>,
    mut update_instance_evemts : EventWriter<InstancingUpdateEvent>) {


    for e in events.iter() {
        station.add_block_event(
            &mut cmds,
            e,
        &mut update_instance_evemts,
        &block_holder);

    }
}

pub fn update_instancing_holders(
    mut query : Query<&mut LocationInstancing>,
    station : Res<Station>,
    mut events : EventReader<InstancingUpdateEvent>
) {
    for (key, chunk) in &station.chunks {
        for event in events.iter() {
            match event {
                InstancingUpdateEvent::Update(e, id) => {
                    match query.get_component_mut::<LocationInstancing>(*e) {
                        Ok(mut loc) => {
                            loc.locs = chunk.collect_sub_locs(*id);
                        },
                        Err(_) => {},
                    }
                },
            }
        }
    }
}
