mod ui;

use std::f32::consts::PI;

use bevy::core_pipeline::bloom::BloomSettings;
use bevy::math::DMat4;
use bevy::math::DQuat;
use bevy::math::DVec2;
use bevy::math::DVec3;
use bevy::window::PrimaryWindow;
use bevy_egui::EguiContext;
use bevy_transform64::prelude::DTransform;
use instance_rotate::InstanceRotate;
use bevy_xpbd_3d::prelude::*;
use ui::*;

use bevy::prelude::*;

use crate::control::Action;
use crate::pawn_system::*;
use crate::ship::*;
use crate::ship::common::{AllVoxelInstances, VoxelInstance, TELEPORN_NAME};
use crate::*;
use crate::ship::save_load::*;
use crate::space_voxel::VoxelMap;
use crate::space_voxel::objected_voxel_map::*;

use super::fps_mode::IsFPSMode;

#[derive(Resource, Default)]
pub struct ActiveWindows {
    pub load_ship : bool
}

#[derive(Default, Clone, Hash, PartialEq, Eq, Debug, States)]
pub enum StationBuildState {
    #[default]
    NotLoaded,
    Loaded
}

pub struct StationBuilderPlugin {}

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
enum ShipBuildSet {
    Base
}

impl Plugin for StationBuilderPlugin {
    fn build(&self, app: &mut App)  {

        app.insert_resource(ActiveWindows::default());
        app.add_state::<StationBuildState>();
        app.add_systems(OnEnter(SceneType::ShipBuilding), setup_build_scene);

        app.configure_set(Update, ShipBuildSet::Base
            .run_if(not(in_state(IsFPSMode::Yes)))
            .run_if(in_state(SceneType::ShipBuilding)));

        app.add_systems(Update,
                (
                    ship_build_menu,
                    pos_block,
                    spawn_block.after(ship_build_menu),
                    rotate_block.after(ship_build_menu),
                    clear_all_system,
                    quick_save,
                    quick_load,
                    capture_loaded_ship,
                    go_to_fps,
                    move_camera_build,
                    z_slicing,
                    load_ship_ui.run_if(|windows : Res<ActiveWindows>| windows.load_ship)
                ).in_set(ShipBuildSet::Base));

        app.add_plugins(StationBuilderUI);
    }
}

pub enum BuildMode {
    SingleOnY(f64)
}

#[derive(Resource)]
pub struct StationBuildBlock {
    pub e : Option<Entity>,
    pub instance : Option<VoxelInstance>,
    pub mode : BuildMode,
    pub ship : Entity,
    pub cur_name : String,
    pub cmd : StationBuildCmds,
    pub z_slice : Option<f64>
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


fn z_slicing(
    mut query : Query<(&DTransform, &mut Visibility), (With<VoxelInstance>, Without<ActiveBlock>)>,
    block : Res<StationBuildBlock>,
) {
    let Some(y_slice) = block.z_slice else {
        return;
    };
    for (transform, mut visible) in query.iter_mut() {
        if transform.translation.y > y_slice {
            *visible = Visibility::Hidden;
        } else {
            *visible = Visibility::Visible;
        }
    }
}

fn move_camera_build(
    mut pawn_query : Query<&mut DTransform, With<Pawn>>,
    pawn : ResMut<CurrentPawn>,
    input : Res<Input<Action>>,
    time : ResMut<Time>,
) {
    let Some(pawn_id) = pawn.id else {
        return;
    };
    if let Ok(mut transform) = pawn_query.get_mut(pawn_id) {
        let frw = transform.forward();
        let frw = DVec3::new(frw.x, 0.0, frw.z).normalize();

        let right = transform.right();
        let right = DVec3::new(right.x, 0.0, right.z).normalize();

        let speed = 10.0;
        if input.pressed(Action::Build(control::BuildAction::MoveForward)) {
            transform.translation += frw * speed * time.delta_seconds() as f64;
        }
        if input.pressed(Action::Build(control::BuildAction::MoveBackward)) {
            transform.translation -= frw * speed * time.delta_seconds() as f64;
        }
        if input.pressed(Action::Build(control::BuildAction::MoveRight)) {
            transform.translation += right * speed * time.delta_seconds() as f64;
        }
        if input.pressed(Action::Build(control::BuildAction::MoveLeft)) {
            transform.translation -= right * speed * time.delta_seconds() as f64;
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
    block : ResMut<StationBuildBlock>,
    mut query : Query<(&mut DTransform, &mut InstanceRotate), With<ActiveBlock>>,
    input : Res<Input<Action>>,
) {
    if let Some(e) = block.e {
        if input.just_pressed(Action::Build(control::BuildAction::RotateCounterClockwise)) {
            if let Ok((mut transform, mut rotate)) = query.get_mut(e) {
                transform.rotate(DQuat::from_rotation_y(PI as f64 / 2.0));
                rotate.rot_steps.x += 1;
                
            }
        }
    }
   
}

fn spawn_block(
    mut cmds : Commands,
    asset_server : Res<AssetServer>,
    buttons : Res<Input<MouseButton>>,
    active_blocks : Query<(&mut DTransform, &InstanceRotate), With<ActiveBlock>>,
    block : ResMut<StationBuildBlock>,
    mut ships : Query<&mut Ship>,
    all_instances : Res<AllVoxelInstances>,
    mut ctx : Query<&mut EguiContext>
) {
    let mut ctx = ctx.single_mut();
    if block.e.is_none() {
        return;
    }

    if ctx.get_mut().is_pointer_over_area() {
        // println!("Captured event of egui");
        return;
    } else {
        // println!("Not captured event of egui");
    }

    let tr;
    let rot;
    if let Ok((ac_tr, ac_rot)) = active_blocks.get(block.e.unwrap()) {
        tr = *ac_tr; 
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
    let mut bbox = inst.bbox;
    if rot.rot_steps.x % 2 == 1 {
        std::mem::swap(&mut bbox.x, &mut bbox.z);
    }
    let hs = bbox.as_dvec3() / 2.0 * VOXEL_SIZE;
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
                        .insert(tr);
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

pub struct DRay {
    /// Starting point of the ray.
    pub origin: DVec3,
    /// Direction of the ray.
    pub direction: DVec3,
}

pub fn viewport_to_world(
    projection_matrix : DMat4,
    logical_viewport_size : DVec2,
    camera_transform: &DGlobalTransform,
    viewport_position: DVec2,
) -> Option<DRay> {
    let target_size = logical_viewport_size;
    let ndc = viewport_position * 2. / target_size - DVec2::ONE;

    let ndc_to_world =
        camera_transform.compute_matrix() * projection_matrix.inverse();
    let world_near_plane = ndc_to_world.project_point3(ndc.extend(1.));
    // Using EPSILON because an ndc with Z = 0 returns NaNs.
    let world_far_plane = ndc_to_world.project_point3(ndc.extend(f64::EPSILON));

    (!world_near_plane.is_nan() && !world_far_plane.is_nan()).then_some(DRay {
        origin: world_near_plane,
        direction: (world_far_plane - world_near_plane).normalize(),
    })
}

fn pos_block(
    cameras : Query<(&Camera, &DGlobalTransform)>,
    mut active_blocks : Query<&mut DTransform, With<ActiveBlock>>,
    block : ResMut<StationBuildBlock>,
    windows : Query<&Window, With<PrimaryWindow>>,
    mut ships : Query<&mut Ship>,
) {
    if block.e.is_none() {
        return;
    }
    let cursot_pos_option = windows.single().cursor_position();
    if cursot_pos_option.is_none() {
        return;
    }
    if !ships.contains(block.ship) {
        return;
    }

    let (cam, tr) = cameras.iter().next().unwrap();
    let cursor_pos = cursot_pos_option.unwrap();

    let mouse_ray = viewport_to_world(
        cam.projection_matrix().as_dmat4(),
        cam.logical_viewport_size().unwrap().as_dvec2(),
        tr,
        cursor_pos.as_dvec2()).unwrap();

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
            let bbox = block.instance.as_ref().unwrap().bbox;
            let hs = bbox.as_dvec3() / 2.0 * ship.map.voxel_size;
            let corner_pos = pos - hs - hs * block.instance.as_ref().unwrap().origin;
            let grid_pos = ship.map.get_grid_pos(&corner_pos);
            active_tr.translation = grid_pos + hs + hs * block.instance.as_ref().unwrap().origin;
        },
    }
}


fn setup_build_scene(
    mut cmds : Commands,
    load_state : Res<State<StationBuildState>>,
    mut next_load_state : ResMut<NextState<StationBuildState>>,
    mut pawn_event : EventWriter<ChangePawn>
) {
    if *load_state.get() != StationBuildState::NotLoaded {
        return;
    }
    next_load_state.set(StationBuildState::Loaded);
    let pawn = cmds.spawn(Camera3dBundle {
        camera_3d : Camera3d {
            clear_color : bevy::core_pipeline::clear_color::ClearColorConfig::Custom(Color::Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }),
            ..default()
        },
        ..default()
    }).insert(
        DTransformBundle::from_transform(
            DTransform::from_xyz(10.0, 10.0, 10.0).looking_at(DVec3::new(0.0, 0.0, 0.0), DVec3::Y))).id();

    cmds.entity(pawn).insert(Pawn { camera_id: pawn });

    let ship_id = new_default_ship(&mut cmds);

    cmds.insert_resource(StationBuildBlock {
        e: None,
        instance: None,
        mode: BuildMode::SingleOnY(0.0),
        ship : ship_id,
        cur_name : "".to_string(),
        cmd : StationBuildCmds::None,
        z_slice : None
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

    pawn_event.send(ChangePawn { new_pawn: pawn, save_stack: false });
}

fn go_to_fps(
    mut cmds : Commands,
    mut pawn_event : EventWriter<ChangePawn>,
    mut block : ResMut<StationBuildBlock>,
    query : Query<(Entity, &DGlobalTransform, &VoxelInstance)>,
    ships : Query<Entity, With<Ship>>,
    all_instances : Res<AllVoxelInstances>) {

    if block.cmd == StationBuildCmds::GoToFPS {
        block.cmd = StationBuildCmds::None;
        
        //find teleport spot
        let mut pos = DVec3::ZERO; 
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
        cam.hdr = false;
        cam.is_active = false;


        let pos = DVec3::new(pos.x, pos.y + 1.0, pos.z);
        let pawn = cmds.spawn(
            
            Collider::capsule(1.5, 0.25))
        .insert(DSpatialBundle::from_transform(DTransform::from_xyz(pos.x, pos.y, pos.z)))
        .insert(RigidBody::Dynamic)
        .insert(LockedAxes::new().lock_rotation_x().lock_rotation_y().lock_rotation_z())
        .insert(GravityScale(1.0)).id();

        let cam_pawn = cmds.spawn(Camera3dBundle {
            camera : cam,
            camera_3d : Camera3d {
                clear_color : bevy::core_pipeline::clear_color::ClearColorConfig::Custom(Color::Rgba { red: 0.0, green: 0.0, blue: 0.0, alpha: 1.0 }),
                ..default()
            },
            ..default()
        })
        .insert(DTransformBundle::from_transform(
            DTransform::from_xyz(0.0, 1.0, 0.0).looking_at(DVec3::new(0.0, 1.0, -1.0), DVec3::Y)
        ))
        .insert(BloomSettings::default()).id();

        cmds.entity(pawn).add_child(cam_pawn);
    
        cmds.entity(pawn).insert(Pawn { camera_id: cam_pawn });
    
        pawn_event.send(ChangePawn { new_pawn: pawn, save_stack: true });

        for ship_e in ships.iter() {
            cmds.entity(ship_e).insert(LockedAxes::default()).insert(RigidBody::Dynamic);
        }
    }
}