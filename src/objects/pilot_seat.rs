
use bevy::prelude::*;
use bevy_rapier3d::prelude::KinematicCharacterController;
use iyes_loopless::prelude::ConditionSet;

use crate::{pawn_system::{ChangePawn, Pawn, CurrentPawn}, Gamemode, control::Action};

struct PawnCache {
    pawn : Entity,
    pawn_transform : Transform,
    controller : KinematicCharacterController,
}

#[derive(Component, Default, Reflect, FromReflect)]
#[reflect(Component)]
pub struct PilotSeat {
    #[reflect(ignore)]
    pawn : Option<PawnCache>,
}

pub struct PilotSeatPlugin;

impl Plugin for PilotSeatPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(Gamemode::FPS)
                    .with_system(seat_in_pilot_seat)
                    .into()
            );
    }
}

fn seat_in_pilot_seat(
    mut commands : Commands,
    mut change_pawn : EventWriter<ChangePawn>,
    mut input : ResMut<Input<Action>>,
    mut pawns : Query<(Entity, &mut Transform), With<Pawn>>,
    mut current_pawn : ResMut<CurrentPawn>,
    mut pilot_seats : Query<(Entity, &Transform, &mut PilotSeat), (Without<Pawn>)>,
) {
    let Some(e) = current_pawn.id else {
        return;
    };
    let Ok((_, seat_tr, mut seat)) = pilot_seats.get_single_mut() else {
        return;
    };
    if input.just_pressed(Action::FPS(crate::control::FPSAction::Interact)) {
        if let Ok((e, mut tr)) = pawns.get_mut(e) {
            if let Some(cache) = &mut seat.pawn {
                commands.entity(cache.pawn).insert(cache.controller.clone());
                if let Ok((_, mut pawn_transform)) = pawns.get_mut(cache.pawn) {
                    pawn_transform.translation = cache.pawn_transform.translation;
                }
                seat.pawn = None;

            } else {
                let cache = PawnCache {
                    pawn : e,
                    pawn_transform : tr.clone(),
                    controller : KinematicCharacterController::default()
                };
                commands.entity(e).remove::<KinematicCharacterController>();
                tr.translation = seat_tr.translation + Vec3::new(0.0, 0.5, 0.0);
                seat.pawn = Some(cache);
            }
        }
    }
}