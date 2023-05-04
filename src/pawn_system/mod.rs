use crate::prelude::*;

pub const PAWN_CHANGE_SYSTEM : &'static str = "PAWN_CHANGE_SYSTEM";

pub struct PawnPlugin;

impl Plugin for PawnPlugin {
    fn build(&self, app: &mut App) {
        
        app.insert_resource(CurrentPawn::default());
        app.add_event::<ChangePawn>();
        
        app.add_system(change_pawn_system);
    }
}



#[derive(Component)]
pub struct Pawn {
    pub camera_id : Entity
}

#[derive(Debug)]
pub struct ChangePawn {
    pub new_pawn : Entity,
    pub new_mode : Gamemode,
    pub save_stack : bool
}


#[derive(Resource, Default)]
pub struct CurrentPawn {
    pub id : Option<Entity>,
    pub stack : Vec<Entity>
}


fn change_pawn_system(
    mut cmds : Commands,
    mut event_reader : EventReader<ChangePawn>,
    mut pawn : ResMut<CurrentPawn>,
        pawn_cam_holders : Query<&Pawn>,
    mut pawn_cams : Query<&mut Camera>,
    mut next_pawn_change : ResMut<NextState<Gamemode>>,
) {
    for pawn_change in event_reader.iter() {
        info!("Pawn changed: {:?}", pawn_change);
        if let Some(e) = pawn.id {
            if let Ok(holder) = pawn_cam_holders.get(e) {
                if let Ok(mut cam) = pawn_cams.get_mut(holder.camera_id) {
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
            if let Ok(mut cam) = pawn_cams.get_mut(holder.camera_id) {
                cam.is_active = true;
            }
        }

        next_pawn_change.set(pawn_change.new_mode.clone());

        break;
    }
    event_reader.clear();
}

