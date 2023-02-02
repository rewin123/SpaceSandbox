use bevy::ecs::entity::EntityMap;
use bevy::prelude::*;
use bevy::scene::serde::SceneDeserializer;
use bevy::utils::{HashSet, HashMap};
use bevy::{tasks::IoTaskPool};
use bevy_egui::*;
use bevy_rapier3d::na::Dyn;
use iyes_loopless::prelude::*;
use bevy_rapier3d::prelude::*;
use serde::de::DeserializeSeed;

use crate::ship::*;
use crate::ship::common::{AllVoxelInstances, VoxelInstance};
use crate::*;
use crate::space_voxel::VoxelMap;
use crate::space_voxel::objected_voxel_map::*;

use std::fs::File;
use std::io::{Write, Read};

pub struct StationBuildMenu {}

impl Plugin for StationBuildMenu {
    fn build(&self, app: &mut App)  {

        app.add_enter_system(SceneType::ShipBuilding, setup_build_scene.label("setup_ship_build_scene"));

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(SceneType::ShipBuilding)
                .with_system(ship_build_menu)
                .with_system(pos_block)
                .with_system(spawn_block)
                .into()
        );

        app.add_system_set(
            ConditionSet::new()
            .run_in_state(SceneType::ShipBuilding)
            .run_if_resource_exists::<StationBuildBlock>()
            .with_system(clear_all_system)
            .with_system(quick_save)
            .with_system(quick_load)
            .into()
        );
    }
}

pub enum BuildMode {
    SingleOnY(f32)
}

#[derive(Resource)]
pub struct StationBuildBlock {
    pub e : Option<Entity>,
    pub instance : Option<VoxelInstance>,
    pub mode : BuildMode,
    pub ship : Entity,
    pub cur_name : String,
    pub cmd : StationBuildCmds
}

#[derive(PartialEq, Eq)]
pub enum StationBuildCmds {
    None,
    ClearAll,
    QuickSave,
    QuickLoad
}

#[derive(Component)]
pub struct ActiveBlock;

fn quick_load(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    ship_entity : Query<Entity, With<Ship>>,
    mut block : ResMut<StationBuildBlock>,
    type_registry : Res<AppTypeRegistry>,
    all_instances : Res<AllVoxelInstances>
) {
    if block.cmd == StationBuildCmds::QuickLoad {
        block.cmd = StationBuildCmds::None;

        for e in &ship_entity {
            cmds.entity(e).despawn_recursive();
        }

        let mut file = File::open("quick.scn.ron").unwrap();
        let mut scene_ron = vec![];
        file.read_to_end(&mut scene_ron).unwrap();
        let mut des = ron::Deserializer::from_bytes(&scene_ron).unwrap();

        let result = SceneDeserializer {
            type_registry : &type_registry.read()
        }.deserialize(&mut des).unwrap();

        let mut entity_map = EntityMap::default();
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

            block.ship = ship_id;

        }
    }
}


fn quick_save(
    world : &mut World
) {
    if world.get_resource_mut::<StationBuildBlock>().unwrap().cmd == StationBuildCmds::QuickSave {
        world.get_resource_mut::<StationBuildBlock>().unwrap().cmd = StationBuildCmds::None;

        let block = world.get_resource::<StationBuildBlock>().unwrap();
        let ship = world.entity(block.ship).get::<Ship>();


        let disk_ship = DiskShip::from_ship(block.ship, &world);
        
        let data_e = world.spawn(DiskShipBase64 {
            data: disk_ship.to_base64(),
        }).id();

        {
            let type_registry = world.resource::<AppTypeRegistry>().clone();
            let dynamic_scene = DynamicScene::from_world(&world, &type_registry);

            let ron_scene = dynamic_scene.serialize_ron(&type_registry).unwrap();

            File::create(format!("quick.scn.ron"))
                .and_then(|mut file| file.write_all(ron_scene.as_bytes())).unwrap();
        }
        world.despawn(data_e);
    }
}

fn clear_all_system(
    mut cmds : Commands,
    ship_entity : Query<Entity, With<Ship>>,
    mut block : ResMut<StationBuildBlock>,

) {
    if block.cmd == StationBuildCmds::ClearAll {
        block.cmd = StationBuildCmds::None;
        for e in &ship_entity {
            cmds.entity(e).despawn_recursive();
        }

        let ship_id = new_default_ship(&mut cmds);
        block.ship = ship_id;
    }
}

fn ship_build_menu(
    mut cmds : Commands,
    mut asset_server : Res<AssetServer>,
    mut voxel_instances : Res<AllVoxelInstances>,
    mut ctx : ResMut<EguiContext>,
    mut block : ResMut<StationBuildBlock>
) {
    egui::SidePanel::left("Build panel").show(ctx.ctx_mut(), |ui| {

        if ui.button("Clear level").clicked() {
            block.cmd = StationBuildCmds::ClearAll;
        }
        ui.separator();
        if ui.button("Quick load").clicked() {
            block.cmd = StationBuildCmds::QuickLoad;
        }
        if ui.button("Quick save").clicked() {
            block.cmd = StationBuildCmds::QuickSave;
        }

        ui.separator();
        for inst in &voxel_instances.configs {
            if ui.button(&inst.name).clicked() {

                let e = inst.create.build(&mut cmds, &asset_server);
                cmds.entity(e).insert(ActiveBlock);

                if let Some(prev_e) = block.e {
                    cmds.entity(prev_e).despawn_recursive();
                }

                block.e = Some(e);
                block.instance = Some(inst.instance.clone());
                block.cur_name = inst.name.clone();
            }
        }
    });
}

fn spawn_block(
    mut cmds : Commands,
    mut asset_server : Res<AssetServer>,
    buttons : Res<Input<MouseButton>>,
    mut active_blocks : Query<(&mut Transform), With<ActiveBlock>>,
    mut block : ResMut<StationBuildBlock>,
    mut ships : Query<&mut Ship>,
    all_instances : Res<AllVoxelInstances>
) {
    if block.e.is_none() {
        return;
    }
    if buttons.just_pressed(MouseButton::Left) {
        let tr;
        if let Ok(ac_tr) = active_blocks.get(block.e.unwrap()) {
            tr = ac_tr;
        } else {
            return;
        }

        let mut ship;
        if let Ok(cur_ship) = ships.get_mut(block.ship) {
            ship = cur_ship;
        } else {
            return;
        }

        let inst = block.instance.as_ref().unwrap();

        let grid_idx = ship.get_grid_idx_by_center(&tr.translation, &inst.bbox);
        if ship.map.can_place_object(&grid_idx, &inst.bbox) {
            // ship.map.set_object_by_idx(e, pos, bbox)
            for inst_cfg in &all_instances.configs {
                if inst_cfg.name == block.cur_name {
                    let e = inst_cfg.create.build(&mut cmds, &asset_server);
                    ship.map.set_object_by_idx(e, &grid_idx, &inst.bbox);
                    let inst_e = cmds.entity(e)
                        .insert(TransformBundle::from_transform(tr.clone())).id();
                    cmds.entity(block.ship).add_child(inst_e);
                }
            }
        }
    }
}

fn pos_block(
    cameras : Query<(&Camera, &GlobalTransform)>,
    mut active_blocks : Query<(&mut Transform), With<ActiveBlock>>,
    block : ResMut<StationBuildBlock>,
    windows : Res<Windows>,
    mut ships : Query<&mut Ship>,
) {
    if block.e.is_none() {
        return;
    }
    let cursot_pos_option = windows.get_primary().unwrap().cursor_position();
    if cursot_pos_option.is_none() {
        return;
    }
    if !ships.contains(block.ship) {
        return;
    }

    let (cam, tr) = cameras.iter().next().unwrap();
    let cursor_pos = cursot_pos_option.unwrap();

    let mouse_ray = cam.viewport_to_world(tr, cursor_pos).unwrap();

    let e = block.e.unwrap();
    let mut active_tr;
    if let Ok(tr) = active_blocks.get_mut(e) {
        (active_tr) = tr;
    } else {
        return;
    }

    match block.mode {
        BuildMode::SingleOnY(lvl) => {
            let ship = ships.get_mut(block.ship).unwrap();

            let t = (lvl - mouse_ray.origin.y) / mouse_ray.direction.y;
            let pos = mouse_ray.origin + t * mouse_ray.direction;
            let bbox = block.instance.as_ref().unwrap().bbox.clone();
            let hs = bbox.as_vec3() / 2.0 * ship.map.voxel_size;
            let corner_pos = pos - hs;
            let grid_pos = ship.map.get_grid_pos(&corner_pos);
            active_tr.translation = grid_pos + hs;
        },
    }
}


fn new_default_ship(cmds : &mut Commands) -> Entity {
    cmds.spawn(Ship::new_sized(IVec3::new(100, 100, 100)))
        .insert(SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)))
        .id()
}

fn setup_build_scene(
    mut cmds : Commands
) {
    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    let ship_id = new_default_ship(&mut cmds);

    cmds.insert_resource(StationBuildBlock {
        e: None,
        instance: None,
        mode: BuildMode::SingleOnY(0.0),
        ship : ship_id,
        cur_name : "".to_string(),
        cmd : StationBuildCmds::None
    });
}