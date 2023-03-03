use bevy::{input::mouse::MouseMotion, window::WindowFocused};
use bevy_rapier3d::prelude::KinematicCharacterController;

use crate::{prelude::*, pawn_system::{CurrentPawn, Pawn}, control::{Action, FPSAction}, objects::prelude::MeteorFieldCommand};


pub struct FPSPlugin;


impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {

        app.add_system_set(ConditionSet::new()
            .run_in_state(Gamemode::FPS)
            .with_system(fps_controller)
            .with_system(fps_focus_control)
            .with_system(fps_look_controller)
            .into());

        app.add_enter_system(Gamemode::FPS, fps_setup);
    }
}

fn fps_setup(
    mut windows : ResMut<Windows>,
    mut meteor_spawn_event : EventWriter<MeteorFieldCommand>,
) {
    windows.get_primary_mut().unwrap().set_cursor_grab_mode(bevy::window::CursorGrabMode::Confined);
    windows.get_primary_mut().unwrap().set_cursor_visibility(false);

    meteor_spawn_event.send(MeteorFieldCommand::Spawn);
}

fn fps_focus_control(
    mut window_focus : EventReader<WindowFocused>,
    mut windows : ResMut<Windows>
) {
    for focus in window_focus.iter() {
        if !focus.focused {
            windows.get_primary_mut().unwrap().set_cursor_grab_mode(bevy::window::CursorGrabMode::None);
            windows.get_primary_mut().unwrap().set_cursor_visibility(true);
        } else {
            windows.get_primary_mut().unwrap().set_cursor_grab_mode(bevy::window::CursorGrabMode::Confined);
            windows.get_primary_mut().unwrap().set_cursor_visibility(false);
        }
    }
}

fn fps_look_controller(
    pawn : Res<CurrentPawn>,
    mut transform : Query<&mut Transform>,
    mut pawns : Query<(&Pawn)>,
    mut mouse_move : EventReader<MouseMotion>,
) {
    let moves = mouse_move.iter().map(|m| m.clone()).collect::<Vec<_>>();
    if let Some(e) = pawn.id {
        if let Ok(pawn) = pawns.get(e) {
            let cam_id = pawn.camera_id;
            if let Ok(mut pawn_transform) = transform.get_mut(cam_id) {
                for mv in &moves {
                    let frw = pawn_transform.forward();
                    let up = pawn_transform.up();
                    let right = pawn_transform.right();
                    let delta = mv.delta * 0.001;
                    let mut changed_frw = (frw - delta.y * up).normalize();
                    changed_frw.y = changed_frw.y.max(-0.95);
                    changed_frw.y = changed_frw.y.min(0.95);
                    let pos = pawn_transform.translation;
                    pawn_transform.look_at(pos + changed_frw, Vec3::new(0.0, 1.0, 0.0));
                }
            }

            let Ok(mut pawn_transform) = transform.get_mut(e) else {
                return;
            };
            for mv in &moves {
                let frw = pawn_transform.forward();
                let up = pawn_transform.up();
                let right = pawn_transform.right();
                let delta = mv.delta * 0.001;
                let mut changed_frw = (frw + delta.x * right).normalize();
                changed_frw.y = changed_frw.y.max(-0.95);
                changed_frw.y = changed_frw.y.min(0.95);
                let pos = pawn_transform.translation;
                pawn_transform.look_at(pos + changed_frw, Vec3::new(0.0, 1.0, 0.0));
            }
        }
    }
}

fn fps_controller(
    pawn : Res<CurrentPawn>,
    mut characters : Query<(&mut Transform, &mut KinematicCharacterController)>,
    mut keys : Res<Input<Action>>,
    mut time : Res<Time>
) {
    if let Some(e) = pawn.id {
            if let Ok((mut pawn_transform, mut controller)) = characters.get_mut(e) {
                let frw = pawn_transform.forward();
                let right = pawn_transform.right();
                let mut move_dir = Vec3::ZERO;
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
                move_dir *= time.delta_seconds() * speed;
                controller.translation = Some(move_dir);
            } else {

            }
    }
}