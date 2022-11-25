use std::marker::PhantomData;
use std::process::id;
use bevy::asset::AssetServer;
use bevy::prelude::{info_span, info};
use egui::{Context, Ui};
use space_game::{Game, GameCommands, SchedulePlugin, GlobalStageStep, EguiContext, SceneType, RonAssetPlugin, RenderApi, InputSystem, KeyCode, ScreenSize};
use space_render::{add_game_render_plugins, AutoInstancing};
use space_core::{ecs::*, app::App, nalgebra, SpaceResult, Pos3i, Vec3i};
use space_core::{serde::*, Camera};
use bevy::asset::*;
use bevy::utils::HashMap;
use winit::event::MouseButton;
use space_assets::{GltfAssetLoader, Location, Material, MeshBundle, SpaceAssetServer, GMesh, LocationInstancing, SubLocation};
use bevy::reflect::TypeUuid;
use std::string::String;
use crate::scenes::station_data::*;
use crate::scenes::station_plugin::*;

#[derive(Component)]
struct StationBuildActiveBlock {}

pub struct StationBuildMenu {}

impl SchedulePlugin for StationBuildMenu {
    fn get_name(&self) -> space_game::PluginName {
        space_game::PluginName::Text("Station build menu".into())
    }

    fn add_system(&self, app : &mut App) {
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
                .with_system(add_block_to_station)
                .with_system(setup_blocks)
                .with_system(update_instancing_holders)
                .with_system(catch_update_events));
        app.add_system_set(
            SystemSet::on_update(CommonBlockState::Waiting)
                .with_system(wait_loading_common_asset));

        app.insert_resource(StationRender::default());
    }
}

fn add_block_to_station(
    mut commands : Commands,
    world : Query<(&Location)>,
    input : Res<InputSystem>,
    mut panels : Res<StationBlocks>,
    mut events : EventWriter<AddBlockEvent>,
    ctx : Res<EguiContext>) {

    if input.get_mouse_button_state(&MouseButton::Left) {
        if ctx.is_pointer_over_area() {
            info!("Mouse over egui");
            return;
        }
        if let Some(e) = panels.active_entity.as_ref() {
            events.send(AddBlockEvent {
                id : panels.active_id.clone(),
                world_pos: world.get_component::<Location>(*e).unwrap().pos.into(),
            });
        }
    }
    
    if input.get_mouse_button_state(&MouseButton::Right) {
        if ctx.is_pointer_over_area() {
            info!("Mouse over egui");
            return;
        }
        if let Some(e) = panels.active_entity.as_ref() {
            events.send(AddBlockEvent{
                id: BuildCommand::None,
                world_pos: world.get_component::<Location>(*e).unwrap().pos.into(),
            });
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
    chunk : Res<Station>,
    render : Res<RenderApi>) {

    let ray = camera.screen_pos_to_ray(
        input.get_mouse_pos(),
        nalgebra::Point2::<f32>::new(screen_size.size.width as f32, screen_size.size.height as f32));

    for mut loc in query.iter_mut() {
        let ray_point = ray.interact_y(panels.build_level as f32);
        let point = chunk.get_grid_pos(&ray_point);
        // let point = ray.pos + 10.0 * ray.dir;

        loc.pos.x = point.x;
        loc.pos.y = point.y;
        loc.pos.z = point.z;
    }
}

fn camera_movement(
    mut camera : ResMut<Camera>,
    input : Res<InputSystem>) {

    let mut frw = camera.up.clone_owned();
    frw.y = 0.0;
    frw = frw.normalize();
    let speed = 0.1;
    let right = camera.get_right();
    if input.get_key_state(KeyCode::W) {
        camera.pos = camera.pos + speed * frw;
    }
    if input.get_key_state(KeyCode::S) {
        camera.pos = camera.pos - speed * frw;
    }
    if input.get_key_state(KeyCode::A) {
        camera.pos = camera.pos - speed * right;
    }
    if input.get_key_state(KeyCode::D) {
        camera.pos = camera.pos + speed * right;
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
    mut block : ResMut<CommonBlock>,
    asset_server : ResMut<AssetServer>,
    descs : ResMut<Assets<RonBlockDesc>>,
    mut state : ResMut<State<CommonBlockState>>,
    mut space_server : ResMut<SpaceAssetServer>,
    render : Res<RenderApi>,
    mut materials : ResMut<Assets<Material>>,
    mut meshes : ResMut<Assets<GMesh>>,
    mut block_holder : ResMut<BlockHolder>) {

    if asset_server.get_load_state(&block.desc) == LoadState::Loaded {
        if let Some(desc) = descs.get(&block.desc) {
            let bundles = space_server.wgpu_gltf_load_cmds(
                &render.device,
                desc.model_path.clone(),
                &mut materials,
                &mut meshes
            );

            let files = space_server.get_files_by_ext_from_folder(
                "assets/ss13/tiles".into(), "png".into());

            let mesh = bundles[0].mesh.clone();
            let base_mat = materials.get(&bundles[0].material).unwrap().clone();

            for file in &files {
                let tex = space_server.load_color_texture(file.clone(), true);
                let mat = Material {
                    color: tex,
                    normal: base_mat.normal.clone(),
                    metallic_roughness: base_mat.metallic_roughness.clone(),
                    version_sum: 0,
                    gbuffer_bind: None
                };
                let mat_handle = materials.add(mat);

                let desc = BlockDesc {
                    mesh : mesh.clone(),
                    material: mat_handle,
                    name: file.clone(),
                    bbox : Vec3i::new(desc.bbox[0], desc.bbox[1], desc.bbox[2])
                };

                let id = BlockId(block_holder.map.len());

                block_holder.map.insert(id, desc);
            }

            info!("Finished loading {} tiles", files.len());
            state.set(CommonBlockState::None);
        }
    }
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
    mut blocs_holder : ResMut<BlockHolder>
) {

    egui::SidePanel::left("Build panel").show(&ctx, |ui| {

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
        for (idx, block) in blocs_holder.map.iter() {
            if ui.button(&block.name).clicked() {
                if let Some(e) = panels.active_entity {
                    commands.entity(e).despawn();
                }

                let e = commands.spawn((block.mesh.clone(), block.material.clone()))
                    .insert(Location::new(&render.device))
                    .insert(StationBuildActiveBlock{}).id();
                panels.active_entity = Some(e.clone());
                panels.active_id = BuildCommand::Block(idx.clone());
            }
        }

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
    desc : Handle<RonBlockDesc>
}



fn init_station_build(
    mut commands : Commands,
    mut assets : Res<AssetServer>,
    mut camera : ResMut<Camera>,
    mut block_state : ResMut<State<CommonBlockState>>
) {
    let mut blocks = StationBlocks::default();
    blocks.panels.push(assets.load("ss13/walls_configs/metal_grid.wall"));

    let common_asset : Handle<RonBlockDesc> = assets.load("ss13/walls_configs/metal_floor.wall");

    commands.insert_resource(CommonBlock {desc : common_asset});
    block_state.set(CommonBlockState::Waiting).unwrap();
    
    // blocks.panels.push(assets.load("ss13/walls_configs/metal_floor.wall"));

    commands.insert_resource(blocks);
    commands.insert_resource(BlockHolder::default());

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

    commands.insert_resource(Station::default());
}

