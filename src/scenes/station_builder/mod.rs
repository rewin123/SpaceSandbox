mod ui;

use bevy_rapier3d::prelude::*;
use ui::*;

use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::pawn_system::*;
use crate::ship::*;
use crate::ship::common::{AllVoxelInstances, VoxelInstance, TELEPORN_NAME};
use crate::*;
use crate::ship::save_load::*;
use crate::space_voxel::VoxelMap;
use crate::space_voxel::objected_voxel_map::*;

#[derive(Resource, Default)]
pub struct ActiveWindows {
    pub load_ship : bool
}

#[derive(Clone, Hash, PartialEq, Eq, Debug)]
pub enum StationBuildState {
    NotLoaded,
    Loaded
}

pub struct StationBuilderPlugin {}

impl Plugin for StationBuilderPlugin {
    fn build(&self, app: &mut App)  {

        app.insert_resource(ActiveWindows::default());

        app.add_loopless_state(StationBuildState::NotLoaded);
        app.add_enter_system(SceneType::ShipBuilding, setup_build_scene.label("setup_ship_build_scene"));

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(SceneType::ShipBuilding)
                .run_not_in_state(Gamemode::FPS)
                .with_system(ship_build_menu)
                .with_system(pos_block)
                .with_system(spawn_block)
                .into()
        );

        app.add_system_set(
            ConditionSet::new()
            .run_in_state(SceneType::ShipBuilding)
            .run_not_in_state(Gamemode::FPS)
            .run_if_resource_exists::<StationBuildBlock>()
            .after(PAWN_CHANGE_SYSTEM)
            .with_system(clear_all_system)
            .with_system(quick_save)
            .with_system(quick_load)
            .with_system(capture_loaded_ship)
            .with_system(go_to_fps)
            .into()
        );

        app.add_system_set(
            ConditionSet::new()
            .run_in_state(SceneType::ShipBuilding)
            .run_not_in_state(Gamemode::FPS)
            .run_if(|windows : Res<ActiveWindows>| windows.load_ship)
            .with_system(load_ship_ui)
            .into()
        );

        app.add_plugin(StationBuilderUI);
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
    QuickLoad,
    GoToFPS
}

#[derive(Component)]
pub struct ActiveBlock;

fn capture_loaded_ship(
    mut block : ResMut<StationBuildBlock>,
    mut cmd_load : EventReader<ShipLoaded>
) {
    for ship in cmd_load.iter() {
        block.ship = ship.0;
    }
}

fn quick_load(
    mut cmds : Commands,
    mut block : ResMut<StationBuildBlock>,
    mut cmd_load : EventWriter<CmdShipLoad>,
    ship_entity : Query<Entity, With<Ship>>,
) {
    if block.cmd == StationBuildCmds::QuickLoad {
        block.cmd = StationBuildCmds::None;

        for e in &ship_entity {
            cmds.entity(e).despawn_recursive();
        }

        cmd_load.send(CmdShipLoad("quick.scn.ron".to_string()));
    }
}


fn quick_save(
    mut block : ResMut<StationBuildBlock>,
    mut cmd_save : EventWriter<CmdShipSave>
) {
    if block.cmd == StationBuildCmds::QuickSave {
        block.cmd = StationBuildCmds::None;
        cmd_save.send(CmdShipSave(block.ship, "quick.scn.ron".to_string()));
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

fn spawn_block(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    buttons : Res<Input<MouseButton>>,
    active_blocks : Query<&mut Transform, With<ActiveBlock>>,
    block : ResMut<StationBuildBlock>,
    mut ships : Query<&mut Ship>,
    all_instances : Res<AllVoxelInstances>
) {
    if block.e.is_none() {
        return;
    }
    if buttons.pressed(MouseButton::Left) {
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
    mut active_blocks : Query<&mut Transform, With<ActiveBlock>>,
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
    mut cmds : Commands,
    load_state : Res<CurrentState<StationBuildState>>,
    mut pawn_event : EventWriter<ChangePawn>
) {
    if load_state.0 != StationBuildState::NotLoaded {
        return;
    }
    cmds.insert_resource(NextState(StationBuildState::Loaded));
    let pawn = cmds.spawn(Camera3dBundle {
        transform: Transform::from_xyz(10.0, 10.0, 10.0).looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    }).id();

    cmds.entity(pawn).insert(Pawn { camera_id: pawn });

    let ship_id = new_default_ship(&mut cmds);

    cmds.insert_resource(StationBuildBlock {
        e: None,
        instance: None,
        mode: BuildMode::SingleOnY(0.0),
        ship : ship_id,
        cur_name : "".to_string(),
        cmd : StationBuildCmds::None
    });

    pawn_event.send(ChangePawn { new_pawn: pawn, new_mode: Gamemode::Godmode, save_stack: false });
}

fn go_to_fps(
    mut cmds : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
    mut block : ResMut<StationBuildBlock>,
    mut query : Query<(&GlobalTransform, &VoxelInstance)>,
    mut ships : Query<&Ship>,
    mut all_instances : Res<AllVoxelInstances>) {

    if block.cmd == StationBuildCmds::GoToFPS {
        block.cmd = StationBuildCmds::None;

        //find teleport spot
        let mut pos = Vec3::ZERO; 
        for idx in &all_instances.configs {
            if idx.name == TELEPORN_NAME {
                let teleport_idx = idx.instance.common_id;
                for (tr, inst) in &query {
                    if inst.common_id == teleport_idx {
                        pos = tr.translation();
                        break;
                    }
                }
                break;
            }
        }


        let mut cam = Camera::default();
        cam.is_active = false;

        let pos = Vec3::new(pos.x, pos.y + 2.0, pos.z);
        let pawn = cmds.spawn(Collider::capsule(Vec3::new(0.0, -1.0, 0.0), Vec3::new(0.0, 1.0, 0.0), 0.5))
        .insert(SpatialBundle::from_transform(Transform::from_xyz(pos.x, pos.y + 2.0, pos.z)))
        .insert(KinematicCharacterController::default()).id();

        let cam_pawn = cmds.spawn(Camera3dBundle {
            transform: Transform::from_xyz(0.0, 1.0, 0.0).looking_at(Vec3::new(0.0, 1.0, -1.0), Vec3::Y),
            camera : cam,
            ..default()
        }).id();

        cmds.entity(pawn).add_child(cam_pawn);
    
        cmds.entity(pawn).insert(Pawn { camera_id: cam_pawn });
    
        pawn_event.send(ChangePawn { new_pawn: pawn, new_mode: Gamemode::FPS, save_stack: true });
    }
}