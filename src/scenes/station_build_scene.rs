use bevy::prelude::*;
use bevy_egui::*;
use iyes_loopless::prelude::*;
use bevy_rapier3d::prelude::*;

use crate::ship::*;
use crate::ship::common::{AllVoxelInstances, VoxelInstance};
use crate::*;
use crate::space_voxel::VoxelMap;
use crate::space_voxel::objected_voxel_map::*;

pub struct StationBuildMenu {}

impl Plugin for StationBuildMenu {
    fn build(&self, app: &mut App)  {

        app.add_enter_system(SceneType::ShipBuilding, setup_build_scene);

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(SceneType::ShipBuilding)
                .with_system(ship_build_menu)
                .with_system(pos_block)
                .with_system(spawn_block)
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
    pub cur_name : String
}

#[derive(Component)]
pub struct ActiveBlock;

fn ship_build_menu(
    mut cmds : Commands,
    mut asset_server : Res<AssetServer>,
    mut voxel_instances : Res<AllVoxelInstances>,
    mut ctx : ResMut<EguiContext>,
    mut block : ResMut<StationBuildBlock>
) {
    egui::SidePanel::left("Build panel").show(ctx.ctx_mut(), |ui| {
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
                    cmds.entity(e).insert(TransformBundle::from_transform(tr.clone()));
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

fn setup_build_scene(
    mut cmds : Commands
) {
    cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    });

    let ship_id = cmds.spawn(Ship::new_sized(IVec3::new(100, 100, 100))).id();

    cmds.insert_resource(StationBuildBlock {
        e: None,
        instance: None,
        mode: BuildMode::SingleOnY(0.0),
        ship : ship_id,
        cur_name : "".to_string()
    });
}