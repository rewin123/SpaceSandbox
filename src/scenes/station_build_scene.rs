use std::default::default;
use bevy::asset::AssetServer;
use bevy::prelude::{info_span, info};
use egui::{Context, Ui};
use space_game::{Game, GameCommands, SchedulePlugin, GlobalStageStep, EguiContext, SceneType, RonAssetPlugin, RenderApi, InputSystem, KeyCode, ScreenSize};
use space_render::{add_game_render_plugins, AutoInstancing};
use space_core::{ecs::*, app::App, nalgebra, SpaceResult};
use space_core::{serde::*, Camera};
use bevy::asset::*;
use bevy::utils::HashMap;
use winit::event::MouseButton;
use space_assets::{GltfAssetLoader, Location, Material, MeshBundle, SpaceAssetServer, GMesh, LocationInstancing, SubLocation};
use bevy::reflect::TypeUuid;
use std::string::String;

#[derive(Component)]
struct StationBuildActiveBlock {}

pub struct StationBuildMenu {}

#[derive(Default)]
pub struct BlockID(usize);

#[derive(Resource)]
pub struct StationChunk {
    pub origin : nalgebra::Point3<i32>,
    pub voxel_size : f32,
    pub floors : Vec<BlockID>,
    pub chunk_size : nalgebra::Vector3<i32>,
    pub instance_renders : HashMap<BlockID, Entity>
}

impl Default for StationChunk {
    fn default() -> Self {
        Self {
            origin : nalgebra::Point3::default(),
            voxel_size : 2.0,
            floors : vec![],
            chunk_size : nalgebra::Vector3::new(16, 16, 16),
            instance_renders : HashMap::new()
        }
    }
}

impl StationChunk {
    pub fn add_block(
        &mut self,
        pos : nalgebra::Vector3<f32>,
        id : BlockID,
        mesh : &Handle<GMesh>,
        mat : &Handle<Material>) -> SpaceResult<()> {


        Ok(())
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

        nalgebra::Point3::new(x, y, z)
    }

    pub fn get_chunked_pos(&self, pos : &nalgebra::Point3<f32>) -> nalgebra::Point3<f32> {
        let idxs = self.get_idx_3d(pos);

        nalgebra::Point3::new(
            idxs.x as f32 * self.voxel_size,
            idxs.y as f32 * self.voxel_size,
            idxs.z as f32 * self.voxel_size,
        )
    }
}

impl SchedulePlugin for StationBuildMenu {
    fn get_name(&self) -> space_game::PluginName {
        space_game::PluginName::Text("Station build menu".into())
    }

    fn add_system(&self, app : &mut App) {

        app.add_plugin(RonAssetPlugin::<RonBlockDesc>{ ext: vec!["wall"], ..default() });

        app.add_system_set(SystemSet::on_enter(SceneType::StationBuilding)
            .with_system(init_station_build));

        app.add_system_set(
            SystemSet::on_update(SceneType::StationBuilding)
                .with_system(station_menu)
                .with_system(camera_movement)
                .with_system(place_block)
                .with_system(add_block_to_station));

    }
}

fn add_block_to_station(
    mut commands : Commands,
    world : &World,
    input : Res<InputSystem>,
    mut panels : Res<StationBlocks>,
    render : Res<RenderApi>,
    chunk : Res<StationChunk>) {

    if input.get_mouse_button_state(&MouseButton::Left) {
        if let Some(e) = panels.active_entity.as_ref() {
            let e_ref = world.entity(*e);
            let bundle = MeshBundle {
                mesh: e_ref.get::<Handle<GMesh>>().unwrap().clone(),
                location: e_ref.get::<Location>().unwrap().clone(&render.device),
                material: e_ref.get::<Handle<Material>>().unwrap().clone()
            };
            commands.spawn(bundle);
        }
    }
}

fn place_block(
    mut commands : Commands,
    mut query : Query<(&mut Location), (With<StationBuildActiveBlock>)>,
    camera : Res<Camera>,
    input : Res<InputSystem>,
    mut panels : ResMut<StationBlocks>,
    screen_size : Res<ScreenSize>,
    chunk : Res<StationChunk>,
    render : Res<RenderApi>) {

    let ray = camera.screen_pos_to_ray(
        input.get_mouse_pos(),
        nalgebra::Point2::<f32>::new(screen_size.size.width as f32, screen_size.size.height as f32));

    for mut loc in query.iter_mut() {
        let ray_point = ray.interact_y(panels.build_level as f32);
        let point = chunk.get_chunked_pos(&ray_point);
        // let point = ray.pos + 10.0 * ray.dir;

        loc.pos.x = point.x;
        loc.pos.y = point.y;
        loc.pos.z = point.z;
    }
}

fn camera_movement(
    mut camera : ResMut<Camera>,
    input : Res<InputSystem>) {
    
    let right = camera.get_right();
    if input.get_key_state(KeyCode::W) {
        camera.pos = camera.pos + 0.01 * camera.up;
    }
    if input.get_key_state(KeyCode::S) {
        camera.pos = camera.pos - 0.01 * camera.up;
    }
    if input.get_key_state(KeyCode::A) {
        camera.pos = camera.pos - 0.01 * right;
    }
    if input.get_key_state(KeyCode::D) {
        camera.pos = camera.pos + 0.01 * right;
    }
}

#[derive(Default, Deserialize, TypeUuid, Debug, Clone)]
#[uuid = "fce6d1f5-4317-4077-b23e-6099747b08dd"]
struct RonBlockDesc {
    pub name : String,
    pub model_path : String
}

#[derive(Resource, Default)]
struct StationBlocks {
    pub panels : Vec<Handle<RonBlockDesc>>,

    pub active_block : Option<RonBlockDesc>,
    pub active_id : BlockID,
    pub active_entity : Option<Entity>,
    pub build_level : i32,
}


fn station_menu(
    mut commands : Commands,
    ctx : Res<EguiContext>,
    mut panels : ResMut<StationBlocks>,
    blocks : Res<Assets<RonBlockDesc>>,
    mut asset_server : ResMut<SpaceAssetServer>,
    render : Res<RenderApi>,
    mut materials : ResMut<Assets<Material>>,
    mut meshes : ResMut<Assets<GMesh>>,
    mut chunk : ResMut<StationChunk>
) {
    egui::SidePanel::left("Build panel").show(&ctx, |ui| {
        if let Some(block) = panels.active_block.as_ref() {
            ui.label(format!("Selected block: {}", block.name));
        } else {
            ui.label(format!("Selected block: None"));
        }
        ui.separator();


        ui.label("Blocks:");
        let mut panel_list = panels.panels.clone();
        for (idx, h) in panel_list.iter().enumerate() {
            if let Some(block) = blocks.get(h) {
                if ui.button(&block.name).clicked() {

                    if let Some(e) = panels.active_entity {
                        commands.entity(e).despawn();
                    }


                    panels.active_block = Some(block.clone());
                    let bundles = asset_server.wgpu_gltf_load_cmds(
                        &render.device,
                        block.model_path.clone(),
                        &mut materials,
                        &mut meshes
                    );
                    for b in bundles {
                        let e = commands.spawn(b)
                            .insert(StationBuildActiveBlock{}).id();
                        panels.active_entity = Some(e);
                    }
                    panels.active_id = BlockID(idx);
                }
            }
        }

        ui.separator();

        if ui.button("Stress test").clicked() {
            if let Some(block) = panels.active_block.as_ref() {
                let mut bundles = asset_server.wgpu_gltf_load_cmds(
                    &render.device,
                    block.model_path.clone(),
                    &mut materials,
                    &mut meshes
                );
                let mut instant_location = LocationInstancing::default();
                for y in -100..100 {
                    for x in -100..100 {
                        let sub = SubLocation {
                            pos: nalgebra::Vector3::new(x as f32, 0.0, y as f32),
                            rotation: nalgebra::Vector3::new(0.0, 0.0, 0.0),
                            scale: nalgebra::Vector3::new(1.0, 1.0, 1.0)
                        };
                        instant_location.locs.push(sub);
                    }
                }
                commands.spawn((
                    bundles[0].mesh.clone(),
                    bundles[0].material.clone(),
                    instant_location));
            }
        }
    });
}

fn init_station_build(
    mut commands : Commands,
    mut assets : Res<AssetServer>,
    mut camera : ResMut<Camera>
) {
    let mut blocks = StationBlocks::default();
    blocks.panels.push(assets.load("ss13/walls_configs/metal_grid.wall"));
    blocks.panels.push(assets.load("ss13/walls_configs/metal_floor.wall"));
    commands.insert_resource(blocks);

    camera.pos.x = 0.0;
    camera.pos.y = 10.0;
    camera.pos.z = 0.0;

    camera.up.y = 0.0;
    camera.up.z = 1.0;
    camera.up.x = 0.0;

    camera.frw.x = 0.0;
    camera.frw.y = -1.0;
    camera.frw.z = 1.0;
    camera.frw = camera.frw.normalize();

    camera.up =  camera.get_right().cross(&camera.frw).normalize();

    commands.insert_resource(StationChunk::default());
}

