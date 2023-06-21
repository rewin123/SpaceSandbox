use std::default;

use bevy::{input::mouse::MouseMotion, window::{WindowFocused, PrimaryWindow, CursorGrabMode}, math::DVec3};

use crate::{prelude::*, pawn_system::{CurrentPawn, Pawn, CurrentPawnMarker}, control::{Action, FPSAction}, objects::prelude::MeteorFieldCommand};


#[derive(Component)]
pub struct FPSController;

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
                fps_look_controller).in_set(OnUpdate(IsFPSMode::Yes))
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
    mut windows : Query<&mut Window, With<PrimaryWindow>>
) {
    let mut window = windows.get_single_mut().unwrap();
    for focus in window_focus.iter() {
        if !focus.focused {
            window.cursor.grab_mode = CursorGrabMode::None;
            window.cursor.visible = true;
        } else {
            window.cursor.grab_mode = CursorGrabMode::Confined;
            window.cursor.visible = false;
        }
    }
}

fn fps_look_controller(
    pawn : Res<CurrentPawn>,
    mut transform : Query<&mut DTransform>,
    mut pawns : Query<(&Pawn)>,
    mut mouse_move : EventReader<MouseMotion>,
) {
    let moves = mouse_move.iter().map(|m| m.clone()).collect::<Vec<_>>();
    if let Some(e) = pawn.id {
        if let Ok(pawn) = pawns.get(e) {
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

fn fps_controller(
    pawn : Res<CurrentPawn>,
    mut characters : Query<(&mut DTransform)>,
    mut keys : Res<Input<Action>>,
    mut time : Res<Time>
) {
    if let Some(e) = pawn.id {
            if let Ok((mut pawn_transform)) = characters.get_mut(e) {
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
                let speed = 5.0 * 1000.0 / 3600.0;
                move_dir = move_dir.normalize_or_zero();
                move_dir *= time.delta_seconds_f64() * speed;
                pawn_transform.translation += move_dir;
            } else {

            }
    }
}