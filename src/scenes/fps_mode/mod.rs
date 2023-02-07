use bevy::input::mouse::MouseMotion;

use crate::{prelude::*, pawn_system::CurrentPawn};


pub struct FPSPlugin;


impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {

        app.add_system_set(ConditionSet::new()
            .run_in_state(Gamemode::FPS)
            .with_system(fps_controller)
            .into());

        app.add_enter_system(Gamemode::FPS, fps_setup);
    }
}

fn fps_setup(
    mut windows : ResMut<Windows>
) {
    windows.get_primary_mut().unwrap().set_cursor_grab_mode(bevy::window::CursorGrabMode::Locked);
}

fn fps_controller(
    pawn : Res<CurrentPawn>,
    mut pawns : Query<(&mut Transform)>,
    mut mouse_move : EventReader<MouseMotion>
) {
    if let Some(e) = pawn.id {
        if let Ok(mut pawn_transform) = pawns.get_mut(e) {
            for mv in mouse_move.iter() {
                let frw = pawn_transform.forward();
                let up = pawn_transform.up();
                let right = pawn_transform.right();
                let delta = mv.delta * 0.01;
                let changed_frw = (frw + delta.x * right - delta.y * up).normalize();
                let pos = pawn_transform.translation;
                pawn_transform.look_at(pos + changed_frw, up);
            }
        }
    }
}