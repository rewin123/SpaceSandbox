use std::default;

use bevy::{input::mouse::MouseMotion, window::{WindowFocused, PrimaryWindow, CursorGrabMode}, math::DVec3};
use serde::{Deserialize, Serialize};
use space_physics::{resources::RapierContext, prelude::{Velocity, point, vector, QueryFilter}};

use crate::{prelude::*, pawn_system::{CurrentPawn, Pawn, CurrentPawnMarker}, control::{Action, FPSAction}, objects::prelude::MeteorFieldCommand};

use space_physics::prelude::nalgebra;

#[derive(Component, Default, Serialize, Deserialize, Clone, PartialEq)]
pub struct FPSController {
    pub walk_speed : f64,
    pub run_speed : f64,
    pub show_develop_window : bool,
    pub capture_control : bool,
    pub speed_relax : f64,
    pub current_move : DVec3,
    pub jump_force : f64,
    pub dash_speed : f64,
    pub dash_time : f64,
    pub dash_interval : f64
}

pub struct FPSPlugin;

#[derive(States, Default, Debug, Hash, PartialEq, Eq, Clone)]
pub enum IsFPSMode {
    Yes,
    #[default]
    No
}

impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {

        app.add_state::<IsFPSMode>();
        app
            .add_systems(
                (fps_controller,
                fps_focus_control,
                fps_look_controller).after(control::remap_system).in_set(OnUpdate(IsFPSMode::Yes))
            );
        
        app.add_system(fps_setup.in_schedule(OnEnter(IsFPSMode::Yes)));

        app.add_system(fps_mod_control);
    }
}

fn fps_mod_control(
    added : Query<Entity, (Added<CurrentPawnMarker>, With<FPSController>)>,
    removed : RemovedComponents<CurrentPawnMarker>,
    mut next_state : ResMut<NextState<IsFPSMode>>,
    
) {
    let added_sum = added.iter().collect::<Vec<_>>().len();
    let sum = added_sum as i32 - removed.len() as i32;
    if sum > 0 {
        next_state.set(IsFPSMode::Yes);
        info!("FPS mode enabled");
    } else if sum < 0 {
        next_state.set(IsFPSMode::No);
        info!("FPS mode disabled");
    }
}

fn fps_setup(
    mut windows : Query<&mut Window, With<PrimaryWindow>>
) {
    let mut window = windows.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::Confined;
    window.cursor.visible = false;
}

fn fps_focus_control(
    mut window_focus : EventReader<WindowFocused>,
    mut windows : Query<&mut Window, With<PrimaryWindow>>,
    mut controllers : Query<&FPSController, With<CurrentPawnMarker>>
) {
    let mut window = windows.get_single_mut().unwrap();
    if let Ok(con) = controllers.get_single_mut() {
      if con.capture_control {
        for focus in window_focus.iter() {
            if !focus.focused {
                window.cursor.grab_mode = CursorGrabMode::None;
                window.cursor.visible = true;
            } else {
                window.cursor.grab_mode = CursorGrabMode::Confined;
                window.cursor.visible = false;
            }
        }
      } else {
        window.cursor.grab_mode = CursorGrabMode::None;
        window.cursor.visible = true;
      }  
    }
    
}

fn fps_look_controller(
    pawn : Res<CurrentPawn>,
    mut transform : Query<&mut DTransform>,
    mut pawns : Query<(&Pawn, &FPSController)>,
    mut mouse_move : EventReader<MouseMotion>
) {
    let moves = mouse_move.iter().map(|m| m.clone()).collect::<Vec<_>>();
    if let Some(e) = pawn.id {
        if let Ok((pawn, controller)) = pawns.get(e) {
            if controller.capture_control {
                let cam_id = pawn.camera_id;
                //head control
                if let Ok(mut pawn_transform) = transform.get_mut(cam_id) {
                    for mv in &moves {
                        let frw = pawn_transform.forward();
                        let up = pawn_transform.up();
                        let right = pawn_transform.right();
                        let delta = mv.delta.as_dvec2() * 0.001;
                        let mut changed_frw = (frw - delta.y * up).normalize();
                        changed_frw.y = changed_frw.y.max(-0.95);
                        changed_frw.y = changed_frw.y.min(0.95);
                        let pos = pawn_transform.translation;
                        pawn_transform.look_at(pos + changed_frw, DVec3::new(0.0, 1.0, 0.0));
                    }
                }

                //body control
                let Ok(mut pawn_transform) = transform.get_mut(e) else {
                    warn!("No pawn transform found for FPS controller");
                    return;
                };
                for mv in &moves {
                    let frw = pawn_transform.forward();
                    let up = pawn_transform.up();
                    let right = pawn_transform.right();
                    let delta = mv.delta.as_dvec2() * 0.001;
                    let mut changed_frw = (frw + delta.x * right).normalize();
                    changed_frw.y = changed_frw.y.max(-0.95);
                    changed_frw.y = changed_frw.y.min(0.95);
                    let pos = pawn_transform.translation;
                    pawn_transform.look_at(pos + changed_frw, DVec3::new(0.0, 1.0, 0.0));
                }
            }
        }
    }
}

fn fps_controller(
    pawn : Res<CurrentPawn>,
    mut characters : Query<(&mut DTransform, &mut FPSController)>,
    mut keys : Res<Input<Action>>,
    mut time : Res<Time>,
    mut physics : ResMut<RapierContext>,
    mut bodies : Query<&mut Velocity>,
    mut twice_click : ResMut<control::TwiceClick>
) {
    if let Some(e) = pawn.id {
            if let Ok((mut pawn_transform, mut controller)) = characters.get_mut(e) {
                let frw = pawn_transform.forward();
                let right = pawn_transform.right();
                let mut move_dir = DVec3::ZERO;
                if keys.pressed(Action::FPS(FPSAction::MoveForward)) {
                    move_dir += frw;
                } 
                if keys.pressed(Action::FPS(FPSAction::MoveBackward)) {
                    move_dir -= frw;
                }
                if keys.pressed(Action::FPS(FPSAction::MoveRight)) {
                    move_dir += right;
                }
                if keys.pressed(Action::FPS(FPSAction::MoveLeft)) {
                    move_dir -= right;
                }
                //notmal human walk speed
                let target_speed = if keys.pressed(Action::FPS(FPSAction::Sprint)) { 
                    controller.run_speed 
                } else if move_dir.length() < 0.1 {
                    0.0
                } else {
                    controller.walk_speed 
                };
                let mut target_move = move_dir.normalize_or_zero() * target_speed;

                let mut dash_move_dir = DVec3::ZERO;
                let mut is_dash = false;
                if twice_click.is_twice(&Action::FPS(FPSAction::MoveForward)) {
                    dash_move_dir += frw;
                    is_dash = true;
                }
                if twice_click.is_twice(&Action::FPS(FPSAction::MoveBackward)) {
                    dash_move_dir -= frw;
                    is_dash = true;
                }
                if twice_click.is_twice(&Action::FPS(FPSAction::MoveRight)) {
                    dash_move_dir += right;
                    is_dash = true;
                }
                if twice_click.is_twice(&Action::FPS(FPSAction::MoveLeft)) {
                    dash_move_dir -= right;
                    is_dash = true;
                }
                dash_move_dir = dash_move_dir.normalize_or_zero() * controller.dash_speed;
                if is_dash && time.elapsed_seconds_f64() - controller.dash_time < controller.dash_interval {
                    info!("Dash timeout");
                    is_dash = false;
                }

                if is_dash {
                    controller.dash_time = time.delta_seconds_f64();
                    target_move = dash_move_dir;
                }

                controller.current_move = 
                target_move  + 
                        (controller.current_move - target_move) * (-controller.speed_relax * time.delta_seconds_f64()).exp();

                

                // move_dir = move_dir.normalize_or_zero();
                // move_dir *= time.delta_seconds_f64() * controller.current_speed;
                pawn_transform.translation += controller.current_move * time.delta_seconds_f64();

                let up = pawn_transform.up();
                let start_pos = pawn_transform.translation - up * 1.1;
                let floor_ray = space_physics::prelude::Ray::new(point![start_pos.x, start_pos.y, start_pos.z],
                    vector![-up.x, -up.y, -up.z]);

                let physics = physics.as_mut();

                if let Some((handle, toi)) = physics.query_pipeline.cast_ray(
                    &physics.rigid_body_set,
                    &physics.collider_set,
                    &floor_ray,
                    0.1,
                    false,
                    QueryFilter::default()
                ) {
                    let intersection_point = floor_ray.point_at(toi);
                    // info!("toi: {:?} point: {:?}", toi, intersection_point);

                    if keys.just_pressed(Action::FPS(FPSAction::Jump)) {
                        // info!("jump");
                        if let Ok(mut vel) = bodies.get_mut(e) {
                            vel.linvel.y += controller.jump_force;
                            // info!("change vel to {:?}", vel.linvel);
                        }
                    }
                }

                
            } else {

            }
    }
}