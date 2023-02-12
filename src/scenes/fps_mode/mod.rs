use bevy::{input::mouse::MouseMotion, window::WindowFocused};

use crate::{prelude::*, pawn_system::{CurrentPawn, Pawn}};


pub struct FPSPlugin;


impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {

        app.add_system_set(ConditionSet::new()
            .run_in_state(Gamemode::FPS)
            .with_system(fps_controller)
            .with_system(fps_focus_control)
            .into());

        app.add_enter_system(Gamemode::FPS, fps_setup);
    }
}

fn fps_setup(
    mut windows : ResMut<Windows>
) {
    windows.get_primary_mut().unwrap().set_cursor_grab_mode(bevy::window::CursorGrabMode::Confined);
    windows.get_primary_mut().unwrap().set_cursor_visibility(false);
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

fn fps_controller(
    pawn : Res<CurrentPawn>,
    mut transform : Query<(&mut Transform)>,
    mut pawns : Query<(&Pawn)>,
    mut mouse_move : EventReader<MouseMotion>
) {
    if let Some(e) = pawn.id {
        if let Ok(pawn) = pawns.get(e) {
            let cam_id = pawn.camera_id;
            if let Ok(mut pawn_transform) = transform.get_mut(cam_id) {
                for mv in mouse_move.iter() {
                    let frw = pawn_transform.forward();
                    let up = pawn_transform.up();
                    let right = pawn_transform.right();
                    let delta = mv.delta * 0.001;
                    let changed_frw = (frw + delta.x * right - delta.y * up).normalize();
                    let pos = pawn_transform.translation;
                    pawn_transform.look_at(pos + changed_frw, Vec3::new(0.0, 1.0, 0.0));
                }
            }
        }
    }
}