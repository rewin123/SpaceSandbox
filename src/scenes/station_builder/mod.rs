mod ui;

use std::f32::consts::PI;

use bevy_rapier3d::prelude::*;
use instance_rotate::InstanceRotate;
use ui::*;

use bevy::prelude::*;
use iyes_loopless::prelude::*;

use crate::control::Action;
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
                .label("ship_build_menu")
                .with_system(ship_build_menu)
                .with_system(pos_block)
                .into()
        );

        app.add_system_set(
            ConditionSet::new()
                .run_in_state(SceneType::ShipBuilding)
                .run_if_resource_exists::<StationBuildBlock>()
                .after("ship_build_menu")
                .with_system(spawn_block)
                .with_system(rotate_block)
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
            .with_system(move_camera_build)
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

fn move_camera_build(
    mut pawn_query : Query<&mut Transform, With<Pawn>>,
    mut pawn : ResMut<CurrentPawn>,
    mut input : Res<Input<Action>>,
    mut time : ResMut<Time>,
) {
    let Some(pawn_id) = pawn.id else {
        return;
    };
    if let Ok(mut transform) = pawn_query.get_mut(pawn_id) {
        let frw = transform.forward();
        let frw = Vec3::new(frw.x, 0.0, frw.z).normalize();

        let right = transform.right();
        let right = Vec3::new(right.x, 0.0, right.z).normalize();

        let speed = 10.0;
        if input.pressed(Action::Build(control::BuildAction::MoveForward)) {
            transform.translation += frw * speed * time.delta_seconds();
        }
        if input.pressed(Action::Build(control::BuildAction::MoveBackward)) {
            transform.translation -= frw * speed * time.delta_seconds();
        }
        if input.pressed(Action::Build(control::BuildAction::MoveRight)) {
            transform.translation += right * speed * time.delta_seconds();
        }
        if input.pressed(Action::Build(control::BuildAction::MoveLeft)) {
            transform.translation -= right * speed * time.delta_seconds();
        }
    }
}

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

fn rotate_block(
    mut block : ResMut<StationBuildBlock>,
    mut query : Query<(&mut Transform, &mut InstanceRotate), With<ActiveBlock>>,
    input : Res<Input<Action>>,
) {
    if let Some(e) = block.e {
        if input.just_pressed(Action::Build(control::BuildAction::RotateCounterClockwise)) {
            if let Ok((mut transform, mut rotate)) = query.get_mut(e) {
                transform.rotate(Quat::from_rotation_y(PI / 2.0));
                rotate.rot_steps.x += 1;
                
            }
        }
    }
   
}

fn spawn_block(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    buttons : Res<Input<MouseButton>>,
    active_blocks : Query<(&mut Transform, &InstanceRotate), With<ActiveBlock>>,
    block : ResMut<StationBuildBlock>,
    mut ships : Query<&mut Ship>,
    all_instances : Res<AllVoxelInstances>,
    mut ctx : ResMut<bevy_egui::EguiContext>
) {
    if block.e.is_none() {
        return;
    }

    if ctx.ctx_mut().is_pointer_over_area() {
        // println!("Captured event of egui");
        return;
    } else {
        // println!("Not captured event of egui");
    }

    let tr;
    let rot;
    if let Ok((ac_tr, ac_rot)) = active_blocks.get(block.e.unwrap()) {
        tr = ac_tr.clone(); 
        rot = ac_rot.clone();
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
    let mut bbox = inst.bbox.clone();
    if rot.rot_steps.x % 2 == 1 {
        std::mem::swap(&mut bbox.x, &mut bbox.z);
    }
    let hs = bbox.as_vec3() / 2.0 * VOXEL_SIZE;
    let grid_idx = ship.get_grid_idx_by_center(&(tr.translation - hs * inst.origin), &bbox);
    let id = ship.map.get_by_idx(&grid_idx).clone();

    if buttons.pressed(MouseButton::Left) {
        if ship.map.can_place_object(&grid_idx, &bbox) {
            // ship.map.set_object_by_idx(e, pos, bbox)
            for inst_cfg in &all_instances.configs {
                if inst_cfg.name == block.cur_name {
                    let e = inst_cfg.create.build(&mut cmds, &asset_server);
                    ship.map.set_object_by_idx(e, &grid_idx, &bbox);
                    println!("{:#?}", &tr);
                    cmds.entity(e)
                        .insert(tr.clone());
                    cmds.entity(block.ship).add_child(e);
                }
            }
        }
    } else if buttons.pressed(MouseButton::Right) {
        
        ship.map.erase_object(&grid_idx, &IVec3::new(50,50,50));
        match id {
            VoxelVal::None => {},
            VoxelVal::Voxel(_) => todo!(),
            VoxelVal::Object(e) => {
                cmds.entity(e).despawn_recursive();
            },
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
            let corner_pos = pos - hs - hs * block.instance.as_ref().unwrap().origin;
            let grid_pos = ship.map.get_grid_pos(&corner_pos);
            active_tr.translation = grid_pos + hs + hs * block.instance.as_ref().unwrap().origin;
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
        camera_3d : Camera3d {
            clear_color : bevy::core_pipeline::clear_color::ClearColorConfig::Custom(Color::Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }),
            ..default()
        },
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

    // ambient light
    cmds.insert_resource(AmbientLight {
        color: Color::WHITE,
        brightness: 0.2,
    });

    const HALF_SIZE: f32 = 100.0;
    cmds.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            // Configure the projection to better fit the scene
            shadow_projection: OrthographicProjection {
                left: -HALF_SIZE,
                right: HALF_SIZE,
                bottom: -HALF_SIZE,
                top: HALF_SIZE,
                near: -100.0 * HALF_SIZE,
                far: 100.0 * HALF_SIZE,
                ..default()
            },
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-2.5),
            ..default()
        },
        ..default()
    });

    pawn_event.send(ChangePawn { new_pawn: pawn, new_mode: Gamemode::Godmode, save_stack: false });
}

fn go_to_fps(
    mut cmds : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
    mut block : ResMut<StationBuildBlock>,
    mut query : Query<(Entity, &GlobalTransform, &VoxelInstance)>,
    mut ships : Query<&Ship>,
    mut all_instances : Res<AllVoxelInstances>) {

    if block.cmd == StationBuildCmds::GoToFPS {
        block.cmd = StationBuildCmds::None;
        
        //find teleport spot
        let mut pos = Vec3::ZERO; 
        for idx in &all_instances.configs {
            if idx.name == TELEPORN_NAME {
                let teleport_idx = idx.instance.common_id;
                for (e, tr, inst) in &query {
                    if inst.common_id == teleport_idx && Some(e) != block.e {
                        pos = tr.translation();
                        break;
                    }
                }
                break;
            }
        }


        let mut cam = Camera::default();
        cam.is_active = false;

        let kinematic_controller = KinematicCharacterController::default();

        let pos = Vec3::new(pos.x, pos.y + 1.0, pos.z);
        let pawn = cmds.spawn(Collider::capsule(Vec3::new(0.0, -0.75, 0.0), Vec3::new(0.0, 0.75, 0.0), 0.25))
        .insert(SpatialBundle::from_transform(Transform::from_xyz(pos.x, pos.y, pos.z)))
        .insert(kinematic_controller)
        .insert(RigidBody::Dynamic)
        .insert(GravityScale(1.0)).id();

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