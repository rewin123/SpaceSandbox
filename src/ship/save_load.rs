use std::fs::File;
use std::io::{Read, Write};

use bevy::ecs::world::{EntityRef, EntityMut};
use bevy::scene::serde::SceneDeserializer;
use bevy::{prelude::*, utils::HashMap};
use egui_notify::Toast;
use serde::de::DeserializeSeed;

use crate::network::{NetworkSplitter, MessageChannel};
use crate::ui::ToastHolder;

use super::common::*;
use super::*;

#[derive(Default)]
pub struct CopyAlgorithm {
    pub steps : Vec<Box<dyn Fn(&mut EntityMut, &mut EntityRef) + Send + Sync>>
}

impl CopyAlgorithm {
    pub fn copy(&self, dst : &mut EntityMut, src : &mut EntityRef) {
        for s in &self.steps {
            (s)(dst, src);
        }
    }
}

pub struct ComponentClone;

impl ComponentClone {
    fn new<T : Component + Clone>() -> Box<dyn Fn(&mut EntityMut, &mut EntityRef) + Send + Sync> {
        Box::new(|dst : &mut EntityMut, src : &mut EntityRef| {
            if let Some(src_cmp) = src.get::<T>() {
                dst.insert(src_cmp.clone());
            }
        })
    }
}

#[derive(Resource)]
pub struct SaveLoadCfg {
    pub save : CopyAlgorithm,
    pub load : CopyAlgorithm
}

impl SaveLoadCfg {
    fn add_simple_clone<T : Component + Clone>(&mut self) {
        self.save.steps.push(ComponentClone::new::<T>());
        self.load.steps.push(ComponentClone::new::<T>());
    }
}

impl Default for SaveLoadCfg {
    fn default() -> Self {
        Self { save: Default::default(), load: Default::default() }
    }
}

#[derive(Resource, Default)]
pub struct ShipSaveQueue(Vec<(Entity, String)>);

pub struct CmdShipSave(pub Entity, pub String);
pub struct CmdShipLoad(pub String);

pub struct ShipLoaded(pub Entity);

pub struct ShipPlugin;

impl Plugin for ShipPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<CmdShipSave>();
        app.add_event::<CmdShipLoad>();
        app.add_event::<ShipLoaded>();
        app.add_event::<SpawnBlockCmd>();

        app.insert_resource(ShipSaveQueue::default());
        app.insert_resource(SaveLoadCfg::default());

        app.add_system(loading_ship_system);
        app.add_system(prepare_saving_ship_system);
        app.add_system(saving_ship_system);

        app.add_startup_system(setup_base_save_load_cfg);

        app.add_startup_system(network_setup);
    }
}

#[derive(Serialize, Deserialize)]
pub struct SpawnBlockCmd {
    pub replicate : bool
}

#[derive(Resource)]
struct SpawnCmdChannel {
    pub channel : MessageChannel<SpawnBlockCmd>
}

fn network_setup(
    mut cmds : Commands,
    mut splitters : ResMut<NetworkSplitter>
) {
    cmds.insert_resource(SpawnCmdChannel {
        channel : splitters.register_type::<SpawnBlockCmd>()
    });
}

fn setup_base_save_load_cfg(
    mut cfg : ResMut<SaveLoadCfg>
) {
    cfg.add_simple_clone::<Transform>();
}

fn saving_ship_system(
    world : &mut World
) {
    let queue = world.resource::<ShipSaveQueue>().0.clone();
    {
        world.resource_mut::<ShipSaveQueue>().0.clear();

        let cfg = world.resource::<SaveLoadCfg>();

        for (ship, path) in &queue {

            let mut sub_world = World::default();
            sub_world.insert_resource(world.resource::<AppTypeRegistry>().clone());

            let mut map = HashMap::new();
            for src_e in world.iter_entities() {
                let mut src_ref = world.entity(src_e);
                let mut dst_ref = sub_world.spawn_empty();
                cfg.save.copy(&mut dst_ref, &mut src_ref);
                map.insert(src_e, dst_ref.id());
            }


            let disk_ship = DiskShip::from_ship(*ship, &world, &map);

            sub_world.spawn(DiskShipBase64 {
                data: disk_ship.to_base64(),
            });

            {
                let type_registry = world.resource::<AppTypeRegistry>().clone();
                let dynamic_scene = DynamicScene::from_world(&sub_world, &type_registry);

                let ron_scene = dynamic_scene.serialize_ron(&type_registry).unwrap();

                File::create(&path)
                    .and_then(|mut file| file.write_all(ron_scene.as_bytes())).unwrap();
            }
        }
    }

    for (_, path) in &queue {
        world.resource_mut::<ToastHolder>().toast.add(Toast::info(format!("Saved ship to {}", &path)));
    }
}

fn prepare_saving_ship_system(
    mut cmd_save : EventReader<CmdShipSave>,
    mut queue : ResMut<ShipSaveQueue>
) {
    for cmd_ship in cmd_save.iter() {
        queue.0.push((cmd_ship.0, cmd_ship.1.clone()));
    }
}

fn loading_ship_system(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    type_registry : Res<AppTypeRegistry>,
    all_instances : Res<AllVoxelInstances>,
    mut load_ships : EventReader<CmdShipLoad>,
    mut loaded_ships : EventWriter<ShipLoaded>,
    mut toast : ResMut<ToastHolder>
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
            let data = sub_world.query::<&DiskShipBase64>().iter(&sub_world).next().unwrap();
            let disk_ship = DiskShip::from_base64(&data.data);

            let mut ship = Ship::new_sized(disk_ship.map.size.clone());
            let mut spawned : HashMap<u32, Entity> = HashMap::new();

            instances_from_disk(disk_ship, &mut ship, &mut spawned, &all_instances, &mut cmds, &asset_server, sub_world);

            let ship_id = cmds.spawn(ship).insert(
                SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)))
                .id();

            for (_, e) in &spawned {
                cmds.entity(ship_id).add_child(*e);
            }

            loaded_ships.send(ShipLoaded(ship_id));
            toast.toast.add(Toast::info(format!("Loaded ship from {}", &ship_path.0)));
        }
    }
}

fn instances_from_disk(
    disk_ship: DiskShip, 
    ship: &mut Ship, 
    spawned: &mut bevy::utils::hashbrown::HashMap<u32, Entity>, 
    all_instances: &Res<AllVoxelInstances>, 
    cmds: &mut Commands, 
    asset_server: &Res<AssetServer>, 
    sub_world: World) {

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
                                    let spawn_e = inst.create.build(cmds, asset_server);
                                    spawned.insert(id.state_id, spawn_e);

                                    let state_e = Entity::from_raw(
                                        disk_ship.states.get(&id.state_id).unwrap().index()
                                    );

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
}