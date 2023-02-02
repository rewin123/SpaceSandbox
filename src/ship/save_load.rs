use std::fs::File;
use std::io::Read;

use bevy::scene::serde::SceneDeserializer;
use bevy::{prelude::*, utils::HashMap};
use serde::de::DeserializeSeed;

use super::common::*;
use super::*;

pub struct CmdShipSave(Entity);
pub struct CmdShipLoad(pub String);

pub struct ShipLoaded(pub Entity);

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CmdShipSave>();
        app.add_event::<CmdShipLoad>();
        app.add_event::<ShipLoaded>();

        app.add_system(loading_ship_system);
    }
}

fn loading_ship_system(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    type_registry : Res<AppTypeRegistry>,
    all_instances : Res<AllVoxelInstances>,
    mut load_ships : EventReader<CmdShipLoad>,
    mut loaded_ships : EventWriter<ShipLoaded>
) {
    for ship_path in load_ships.iter() {
        let mut file = File::open(&ship_path.0).unwrap();
        let mut scene_ron = vec![];
        file.read_to_end(&mut scene_ron).unwrap();
        let mut des = ron::Deserializer::from_bytes(&scene_ron).unwrap();

        let result = SceneDeserializer {
            type_registry : &type_registry.read()
        }.deserialize(&mut des).unwrap();

        let mut sub_world = Scene::from_dynamic_scene(&result, &type_registry).unwrap().world;

        {
            let data = sub_world.query::<(&DiskShipBase64)>().iter(&sub_world).next().unwrap();
            let disk_ship = DiskShip::from_base64(&data.data);

            let mut ship = Ship::new_sized(disk_ship.map.size.clone());
            let mut spawned : HashMap<u32, Entity> = HashMap::new();

            for z in 0..disk_ship.map.size.z {
                for y in 0..disk_ship.map.size.y {
                    for x in 0..disk_ship.map.size.x {
                        let idx = IVec3::new(x, y, z);
                        let disk_v = disk_ship.map.get_by_idx(&idx);

                        match disk_v {
                            DiskShipVoxel::None => {
                                ship.map.set_voxel_by_idx(&idx, VoxelVal::None);
                            },
                            DiskShipVoxel::Voxel(block) => {
                                ship.map.set_voxel_by_idx(&idx, VoxelVal::Voxel(block.clone()))
                            },
                            DiskShipVoxel::Instance(id) => {
                                if spawned.contains_key(&id.state_id) {
                                    ship.map.set_voxel_by_idx(&idx, VoxelVal::Object(*spawned.get(&id.state_id).unwrap()))
                                } else {
                                    let name = disk_ship.template_names.get(&id.template_id).unwrap().clone();

                                    for inst in &all_instances.configs {
                                        if inst.name == name {
                                            let spawn_e = inst.create.build(&mut cmds, &asset_server);
                                            spawned.insert(id.state_id, spawn_e);

                                            let state_e = Entity::from_raw(
                                                disk_ship.states.get(&id.state_id).unwrap().index()
                                            );

                                            //transform
                                            cmds.entity(spawn_e).insert(SpatialBundle::from_transform(
                                                sub_world.entity(state_e).get::<Transform>().unwrap().clone()
                                            ));

                                            ship.map.set_voxel_by_idx(&idx, VoxelVal::Object(spawn_e))
                                        }
                                    }
                                }
                            },
                        }
                    }
                }
            }

            let ship_id = cmds.spawn(ship).insert(
                SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)))
                .id();

            for (_, e) in &spawned {
                cmds.entity(ship_id).add_child(*e);
            }

            loaded_ships.send(ShipLoaded(ship_id));
        }
    }
}