use bevy::{input::mouse::MouseMotion, window::{WindowFocused, PrimaryWindow, CursorGrabMode}};

use crate::{prelude::*, pawn_system::{CurrentPawn, Pawn}, control::{Action, FPSAction}, objects::prelude::MeteorFieldCommand};


pub struct FPSPlugin;


impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {

        app
            .add_systems(
                (fps_controller,
                fps_focus_control,
                fps_look_controller).in_set(OnUpdate(Gamemode::FPS))
            );
        
        app.add_system(fps_setup.in_schedule(OnEnter(Gamemode::FPS)));
    }
}

fn fps_setup(
    mut windows : Query<&mut Window, With<PrimaryWindow>>,
    mut meteor_spawn_event : EventWriter<MeteorFieldCommand>,
) {
    let mut window = windows.get_single_mut().unwrap();
    window.cursor.grab_mode = CursorGrabMode::Confined;
    window.cursor.visible = false;

    meteor_spawn_event.send(MeteorFieldCommand::Spawn);
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
    mut characters : Query<(&mut Transform)>,
    mut keys : Res<Input<Action>>,
    mut time : Res<Time>
) {
    // if let Some(e) = pawn.id {
    //         if let Ok((mut pawn_transform, mut controller)) = characters.get_mut(e) {
    //             let frw = pawn_transform.forward();
    //             let right = pawn_transform.right();
    //             let mut move_dir = Vec3::ZERO;
    //             if keys.pressed(Action::FPS(FPSAction::MoveForward)) {
    //                 move_dir += frw;
    //             } 
    //             if keys.pressed(Action::FPS(FPSAction::MoveBackward)) {
    //                 move_dir -= frw;
    //             }
    //             if keys.pressed(Action::FPS(FPSAction::MoveRight)) {
    //                 move_dir += right;
    //             }
    //             if keys.pressed(Action::FPS(FPSAction::MoveLeft)) {
    //                 move_dir -= right;
    //             }
    //             //notmal human walk speed
    //             let speed = 5.0 * 1000.0 / 3600.0;
    //             move_dir = move_dir.normalize_or_zero();
    //             move_dir *= time.delta_seconds() * speed;
    //             controller.translation = Some(move_dir);
    //         } else {

    //         }
    // }
}