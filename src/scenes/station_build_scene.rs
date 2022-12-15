use std::marker::PhantomData;
use std::process::id;
use bevy::asset::AssetServer;
use bevy::prelude::{info_span, info, Transform, Camera3dBundle, GlobalTransform, Mesh, SceneBundle, Scene, StandardMaterial, PbrBundle, PerspectiveProjection, DirectionalLightBundle, DirectionalLight, Color};
use bevy::window::Windows;
use space_game::{Game, GameCommands, SchedulePlugin, GlobalStageStep, SceneType, RonAssetPlugin, RenderApi, InputSystem, KeyCode, ScreenSize};
use space_render::{add_game_render_plugins, AutoInstancing};
use space_core::{ecs::*, app::App, nalgebra, SpaceResult, Pos3i, Vec3i, Vec3, Pos3};
use space_core::{serde::*, Camera};
use bevy::asset::*;
use bevy::utils::{default, HashMap};
use winit::event::MouseButton;
use space_assets::{GltfAssetLoader, Material, MeshBundle, SpaceAssetServer, GMesh, LocationInstancing, SubLocation};
use bevy::reflect::TypeUuid;
use std::string::String;
use bevy::input::Input;
use bevy::log::error;
use bevy::prelude::Projection::Perspective;
use bevy_egui::{egui, EguiContext};
use bevy_egui::egui::Key;
use space_core::app::Plugin;
use crate::scenes::station_data::*;
use crate::scenes::station_plugin::*;

#[derive(Component)]
struct StationBuildActiveBlock {
    pub voxel_pos : Pos3
}

pub struct StationBuildMenu {}

impl Plugin for StationBuildMenu {
    fn build(&self, app: &mut App)  {
        app.add_state(CommonBlockState::None);

        app.add_plugin(RonAssetPlugin::<RonBlockDesc>{ ext: vec!["wall"], phantom: PhantomData::default() });

        app.add_event::<AddBlockEvent>();
        app.add_event::<InstancingUpdateEvent>();
        app.add_event::<ChunkUpdateEvent>();

        app.add_system_set(SystemSet::on_enter(SceneType::StationBuilding)
            .with_system(init_station_build));

        app.add_system_set(
            SystemSet::on_update(SceneType::StationBuilding)
                .with_system(station_menu)
                .with_system(camera_movement)
                .with_system(place_block)
                .with_system(add_block_to_station.after(station_menu))
                .with_system(setup_blocks)
                .with_system(update_instancing_holders)
                .with_system(catch_update_events)
                .with_system(update_station_instancing));
        app.add_system_set(
            SystemSet::on_update(CommonBlockState::Waiting)
                .with_system(wait_loading_common_asset));

        app.insert_resource(StationRender::default());
        app.insert_resource(AutoInstanceHolder::default());
    }
}

fn add_block_to_station(
    mut commands : Commands,
    world : Query<(&Transform, &StationBuildActiveBlock)>,
    mut panels : Res<StationBlocks>,
    mut events : EventWriter<AddBlockEvent>,
    mut ctx : ResMut<EguiContext>,
    input : Res<Input<bevy::prelude::MouseButton>>) {

    if input.just_pressed(bevy::prelude::MouseButton::Left) {
        if ctx.ctx_mut().is_pointer_over_area() {
            info!("Mouse over egui");
            return;
        }
        if let Some(e) = panels.active_entity.as_ref() {
            events.send(AddBlockEvent {
                id : panels.active_id.clone(),
                world_pos: world.get_component::<StationBuildActiveBlock>(*e).unwrap().voxel_pos.into(),
                rot : panels.mode.clone()
            });
        }
    }
    
    if input.just_pressed(bevy::prelude::MouseButton::Right) {
        if ctx.ctx_mut().is_pointer_over_area() {
            info!("Mouse over egui");
            return;
        }
        if let Some(e) = panels.active_entity.as_ref() {
            events.send(AddBlockEvent{
                id: BuildCommand::None,
                world_pos: world.get_component::<StationBuildActiveBlock>(*e).unwrap().voxel_pos.into(),
                rot : panels.mode.clone()
            });
        }
    }
    
}

fn place_block(
    mut commands : Commands,
    mut query : Query<(&mut Transform, &mut StationBuildActiveBlock)>,
    mut cameras : Query<(&bevy::prelude::Camera, &GlobalTransform)>,
    input : Res<Input<bevy::prelude::KeyCode>>,
    windows : Res<Windows>,
    mut panels : ResMut<StationBlocks>,
    chunk : Res<Station>,
    block_holder : Res<BlockHolder>) {

    let win = windows.get_primary().unwrap();

    for (cam, cam_t) in &mut cameras {
        let cursor_pos = {
            match win.cursor_position() {
                Some(pos) => {
                    pos
                }
                None => {
                    return;
                }
            }
        };
        let dir = {
            match cam.viewport_to_world(cam_t, cursor_pos) {
                Some(val) => {
                    val
                }
                None => {
                    return ;
                }
            }
        };

        for  (mut loc, mut active_pos) in query.iter_mut() {

            let y0 = panels.build_level as f32 * chunk.map.voxel_size;
            let t = (y0 - dir.origin.y) / dir.direction.y;

            let ray_point = dir.origin + dir.direction * t;

            let point = chunk.get_grid_pos(&[ray_point.x, ray_point.y, ray_point.z].into());
            let point = Pos3::new(
                ((point.x / chunk.map.voxel_size / 2.0) as i32 * 2) as f32 * chunk.map.voxel_size,
                point.y,
                ((point.z / chunk.map.voxel_size / 2.0) as i32 * 2) as f32 * chunk.map.voxel_size,
            );

            // let point = ray.pos + 10.0 * ray.dir;

            if let BuildCommand::Block(id) = &panels.active_id {
                if let Some(desc) = block_holder.map.get(id) {
                    let mut bbox = desc.bbox.clone();
                    let rot;
                    match panels.mode {
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
                        bbox.x as f32 * chunk.map.voxel_size / 2.0,
                        bbox.y as f32 * chunk.map.voxel_size / 2.0,
                        bbox.z as f32 * chunk.map.voxel_size / 2.0,
                    );

                    loc.translation.x = point.x;
                    loc.translation.y = point.y;
                    loc.translation.z = point.z;
                    loc.translation.x += shift.x;
                    loc.translation.y += shift.y;
                    loc.translation.z += shift.z;

                    active_pos.voxel_pos = point;
                }
            }
        }
    }



}

#[derive(Component)]
struct TopDownCamera {}

fn camera_movement(
    mut query : Query<(&TopDownCamera, &mut Transform)>,
    input : Res<Input<bevy::prelude::KeyCode>>) {

    let speed = 0.1;
    for (_td, mut transform) in &mut query {

        let mut frw = transform.forward();
        frw.y = 0.0;
        let right = transform.right();

        if input.pressed(bevy::prelude::KeyCode::W) {
            transform.translation = transform.translation + speed * frw;
        }
        if input.pressed(bevy::prelude::KeyCode::S) {
            transform.translation = transform.translation - speed * frw;
        }
        if input.pressed(bevy::prelude::KeyCode::A) {
            transform.translation = transform.translation - speed * right;
        }
        if input.pressed(bevy::prelude::KeyCode::D) {
            transform.translation = transform.translation + speed * right;
        }
        if input.pressed(bevy::prelude::KeyCode::LShift) {
            transform.translation.y = transform.translation.y + speed;
        }
        if input.pressed(bevy::prelude::KeyCode::LControl) {
            transform.translation.y = transform.translation.y - speed;
        }
    }

}

#[derive(Default, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "fce6d1f5-4317-4077-b23e-6099747b08dd"]
pub struct RonBlockDesc {
    pub name : String,
    pub model_path : String,
    pub bbox : Vec<i32>
}

#[derive(Resource, Default)]
struct StationBlocks {
    pub panels : Vec<Handle<RonBlockDesc>>,

    pub active_id : BuildCommand,
    pub active_entity : Option<Entity>,
    pub build_level : i32,

    pub mode : BlockAxis
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
enum CommonBlockState {
    None,
    Waiting
}

fn wait_loading_common_asset(
    mut cmds : Commands,
    mut block : ResMut<CommonBlock>,
    asset_server : ResMut<AssetServer>,
    descs : ResMut<Assets<RonBlockDesc>>,
    mut state : ResMut<State<CommonBlockState>>,
    mut block_holder : ResMut<BlockHolder>,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<bevy::prelude::StandardMaterial>>) {

    if asset_server.get_load_state(&block.desc) == LoadState::Loaded {
        //wait all to load
        for h in &block.all_blocks {
            if asset_server.get_load_state(h) != LoadState::Loaded {
                return;
            }
        }

        if let Some(desc) = descs.get(&block.desc) {

            let gltf : Handle<Scene> = asset_server.load(format!("{}#Scene0", desc.model_path));

            cmds.spawn(SceneBundle {
                scene : gltf,
                transform: Default::default(),
                global_transform: Default::default(),
                visibility: Default::default(),
                computed_visibility: Default::default()
            });


            // let bundles = space_server.wgpu_gltf_load_cmds(
            //     &render.device,
            //     desc.model_path.clone(),
            //     &mut materials,
            //     &mut meshes
            // );
            //
            // let files = space_server.get_files_by_ext_from_folder(
            //     "assets/ss13/tiles".into(), "png".into());
            //
            // let mesh = bundles[0].mesh.clone();
            // let base_mat = materials.get(&bundles[0].material).unwrap().clone();

            // for file in &files {
            //     let tex = space_server.load_color_texture(file.clone(), true);
            //     let mat = Material {
            //         color: tex,
            //         normal: base_mat.normal.clone(),
            //         metallic_roughness: base_mat.metallic_roughness.clone(),
            //         version_sum: 0,
            //         gbuffer_bind: None
            //     };
            //     let mat_handle = materials.add(mat);
            //
            //     let desc = BlockDesc {
            //         mesh : mesh.clone(),
            //         material: mat_handle,
            //         name: file.clone(),
            //         bbox : Vec3i::new(desc.bbox[0], desc.bbox[1], desc.bbox[2])
            //     };
            //
            //     let id = BlockId(block_holder.map.len());
            //
            //     block_holder.map.insert(id, desc);
            // }

            // info!("Finished loading {} tiles", files.len());
            state.set(CommonBlockState::None);
        }

        for h in &block.all_blocks {
            if let Some(desc) = descs.get(h) {
                // let bundles = space_server.wgpu_gltf_load_cmds(
                //     &render.device,
                //     desc.model_path.clone(),
                //     &mut materials,
                //     &mut meshes
                // );

                // let mesh = bundles[0].mesh.clone();
                // let mat = bundles[0].material.clone();
                //
                // let desc = BlockDesc {
                //     mesh,
                //     material: mat,
                //     name: desc.name.clone(),
                //     bbox : Vec3i::new(desc.bbox[0], desc.bbox[1], desc.bbox[2])
                // };
                //
                // let id = BlockId(block_holder.map.len());
                //
                // block_holder.map.insert(id, desc);
            }
        }
    }
}

fn station_menu(
    mut commands : Commands,
    mut ctx : ResMut<EguiContext>,
    mut panels : ResMut<StationBlocks>,
    mut blocs_holder : ResMut<BlockHolder>,
    mut block_events : EventWriter<AddBlockEvent>,
) {

    egui::SidePanel::left("Build panel").show(ctx.ctx_mut(), |ui| {

        if ui.button("Test load ss13 map").clicked() {
            let map_res = std::fs::read_to_string("assets/ss13/ss_map.txt");
            if let Ok(map) = map_res {
                let mut state = true;
                let mut first = true;

                let mut tile_count = 0;
                let mut load_idx = 0;
                let mut tile_names = vec![];

                let mut reidx = vec![];

                for line in map.split('\n') {
                    if first {
                        first = false;
                        tile_count = line.parse::<usize>().unwrap();
                    } else if state {
                        tile_names.push(line.to_string());
                        load_idx += 1;

                        if load_idx >= tile_count {
                            state = false;

                            for file_tile in &tile_names {
                                for (id, name) in blocs_holder.map.iter() {
                                    if name.name.contains(file_tile) {
                                        reidx.push(id.clone());
                                        info!("Tile {} with id {:?}", file_tile, id);
                                        break;
                                    }
                                }
                            }
                        }
                    } else {
                        if line.len() == 0 {
                            continue;
                        }
                        let parts : Vec<&str> = line.split(' ').collect();
                        let x = parts[0].parse::<usize>().unwrap();
                        let y = parts[1].parse::<usize>().unwrap();
                        let idx = parts[2].parse::<usize>().unwrap();
                        let mut z = 0.0;

                        if idx != tile_names.len() {
                            if tile_names[idx].contains("wall") {
                                z = 0.5;
                            }
                            let pos = Pos3::new(
                                (x as f32) - 255.0 / 2.0 ,
                                z,
                                (y as f32) - 255.0 / 2.0
                            );

                            block_events.send(AddBlockEvent {
                                id: BuildCommand::Block(reidx[idx].clone()),
                                world_pos: pos,
                                rot: BlockAxis::Y
                            });
                        }
                    }
                }
            } else {
                error!("Cannot find ss13 map");
            }

        }

        if ui.input().key_pressed(Key::Q) {
            panels.build_level -= 1;
        }
        if ui.input().key_pressed(Key::E) {
            panels.build_level += 1;
        }

        ui.add(egui::DragValue::new(&mut panels.build_level).prefix("Build level "));

        match &panels.active_id {
            BuildCommand::None => {
                ui.label(format!("Selected block: None"));
            }
            BuildCommand::Voxel(id) => {

            }
            BuildCommand::Block(id) => {
                ui.label(format!("Selected block: {}", id.0));
            }
        }

        ui.separator();

        egui::ComboBox::new("Axis", "Axis:")
            .selected_text(format!("{:?}", &panels.mode))
            .show_ui(ui, |ui| {
            ui.selectable_value(&mut panels.mode, BlockAxis::X, "X");
            ui.selectable_value(&mut panels.mode, BlockAxis::Y, "Y");
            ui.selectable_value(&mut panels.mode, BlockAxis::Z, "Z");
        });

        ui.label("Blocks:");
        let mut panel_list = panels.panels.clone();
        egui::ScrollArea::vertical().show(ui, |ui| {
            for (idx, block) in blocs_holder.map.iter() {
                if ui.button(&block.name).clicked() {
                    if let Some(e) = panels.active_entity {
                        commands.entity(e).despawn();
                    }
    
                    let e = commands.spawn((block.mesh.clone(), block.material.clone()))
                        .insert(bevy::prelude::Transform::default())
                        .insert(StationBuildActiveBlock{ voxel_pos : Pos3::default()}).id();
                    panels.active_entity = Some(e.clone());
                    panels.active_id = BuildCommand::Block(idx.clone());
                }
            }
        });
        

        ui.separator();

        // if ui.button("Stress test").clicked() {
        //     if panels.active_id != BlockID::None {
        //         let block = &blocs_holder.map[&panels.active_id];
        //
        //         let mut instant_location = LocationInstancing::default();
        //         for y in -100..100 {
        //             for x in -100..100 {
        //                 let sub = SubLocation {
        //                     pos: nalgebra::Vector3::new(x as f32, 0.0, y as f32),
        //                     rotation: nalgebra::Vector3::new(0.0, 0.0, 0.0),
        //                     scale: nalgebra::Vector3::new(1.0, 1.0, 1.0)
        //                 };
        //                 instant_location.locs.push(sub);
        //             }
        //         }
        //         commands.spawn((
        //             block.mesh.clone(),
        //             block.material.clone(),
        //             instant_location));
        //     }
        // }
    });
}

#[derive(Resource)]
struct CommonBlock {
    desc : Handle<RonBlockDesc>,
    all_blocks : Vec<Handle<RonBlockDesc>>
}



fn init_station_build(
    mut commands : Commands,
    mut assets : Res<AssetServer>,
    mut block_state : ResMut<State<CommonBlockState>>,
    mut meshes : ResMut<Assets<Mesh>>,
    mut materials : ResMut<Assets<StandardMaterial>>
) {
    let mut blocks = StationBlocks::default();
    blocks.panels.push(assets.load("ss13/walls_configs/metal_grid.wall"));
    blocks.panels.push(assets.load("ss13/walls_configs/metal_wall.wall"));
    blocks.panels.push(assets.load("ss13/walls_configs/glass_wall.wall"));
    blocks.panels.push(assets.load("ss13/walls_configs/door.wall"));

    let common_asset : Handle<RonBlockDesc> = assets.load("ss13/walls_configs/metal_floor.wall");

    commands.insert_resource(CommonBlock {desc : common_asset, all_blocks : blocks.panels.clone() });
    block_state.set(CommonBlockState::Waiting).unwrap();
    
    // blocks.panels.push(assets.load("ss13/walls_configs/metal_floor.wall"));

    commands.insert_resource(blocks);
    commands.insert_resource(BlockHolder::default());

    let mut camera = Camera3dBundle::default();

    camera.transform.translation.x = -10.0;
    camera.transform.translation.y = 10.0;
    camera.transform.translation.z = 0.0;

    camera.projection = Perspective(PerspectiveProjection::default());

    camera.transform.look_at([0.0, 0.0, 0.0].into(), [0.0, 1.0, 0.0].into());

    commands.spawn(camera)
        .insert(TopDownCamera{});
    
    // for z in -10..10 {
    //     for x in -10..10 {
    //         commands.spawn(PbrBundle {
    //             mesh: meshes.add(Mesh::from(bevy::prelude::shape::Cube::new(1.0))),
    //             material: materials.add(StandardMaterial::default()),
    //             transform: Transform {
    //                 translation : [x as f32 * 2.0, 0.0, z as f32 * 2.0].into(),
    //                 ..default()
    //             },
    //             ..default()
    //         });
    //     }
    // }

    let mut tr = Transform::from_xyz(10.0, 10.0, 10.0);
    tr.look_at([0.0, 0.0, 0.0].into(), [0.0, 1.0, 0.0].into());
    commands.spawn(DirectionalLightBundle {
        directional_light : DirectionalLight {
            color: Color::rgb(1.0,1.0,1.0),
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        transform : tr,
        ..default()
    });

    commands.insert_resource(Station::default());
}

