use crate::prelude::*;

pub const PAWN_CHANGE_SYSTEM : &'static str = "PAWN_CHANGE_SYSTEM";

pub struct FPSPlugin;

#[derive(Resource, Default)]
pub struct CurrentPawn {
    pub id : Option<Entity>,
    pub stack : Vec<Entity>
}

#[derive(Component)]
pub struct PawnCamera {
    pub id : Entity
}

pub struct ChangePawn {
    pub new_pawn : Entity,
    pub new_mode : Gamemode,
    pub save_stack : bool
}

impl Plugin for FPSPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(CurrentPawn::default());
        app.add_event::<ChangePawn>();

        app.add_system_set(ConditionSet::new()
            .run_in_state(Gamemode::FPS)
            .with_system(fps_controller)
            .into());

        app.add_system(change_pawn_system.label(PAWN_CHANGE_SYSTEM));
    }
}

fn change_pawn_system(
    mut cmds : Commands,
    mut event_reader : EventReader<ChangePawn>,
    mut pawn : ResMut<CurrentPawn>,
        pawn_cam_holders : Query<&PawnCamera>,
    mut pawn_cams : Query<&mut Camera>
) {
    for pawn_change in event_reader.iter() {

        if let Some(e) = pawn.id {
            if let Ok(holder) = pawn_cam_holders.get(e) {
                if let Ok(mut cam) = pawn_cams.get_mut(holder.id) {
                    cam.is_active = false;
                }
            }
            if pawn_change.save_stack {
                pawn.stack.push(e);
            }
        }

        if !pawn_change.save_stack {
            pawn.stack.clear();
        }

        pawn.id = Some(pawn_change.new_pawn);
        if let Ok(holder) = pawn_cam_holders.get(pawn_change.new_pawn) {
            if let Ok(mut cam) = pawn_cams.get_mut(holder.id) {
                cam.is_active = true;
            }
        }

        cmds.insert_resource(NextState(pawn_change.new_mode.clone()));

        break;
    }
    event_reader.clear();
}


fn fps_controller(
    pawn : Res<CurrentPawn>
) {
    if let Some(e) = pawn.id {

    }
}