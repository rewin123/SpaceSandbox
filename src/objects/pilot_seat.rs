
use std::sync::Arc;

use bevy::prelude::*;
use bevy_egui::*;
use bevy_rapier3d::prelude::*;

use crate::{pawn_system::{ChangePawn, Pawn, CurrentPawn}, Gamemode, control::{Action, FPSAction, PilotingAction}, ship::Ship};

struct PawnCache {
    pawn : Entity,
    pawn_transform : Transform,
}

#[derive(Component, Default, Reflect, FromReflect)]
#[reflect(Component)]
pub struct PilotSeat {
    #[reflect(ignore)]
    pawn : Option<PawnCache>,
}

pub struct PilotSeatPlugin;

const PILOT_POSITION : Vec3 = Vec3::new(0.0, 0.5, 0.0);

impl Plugin for PilotSeatPlugin {
    fn build(&self, app: &mut App) {

        app.add_system(
            seat_in_pilot_seat.in_set(OnUpdate(Gamemode::FPS))
        );
        app.add_system(
            pilot_debug_ui.in_set(OnUpdate(Gamemode::FPS))
        );
        app.add_system(
            piloting.in_set(OnUpdate(Gamemode::FPS))
        );
    }
}

fn piloting(
    mut pilot_seats : Query<(&mut PilotSeat), (Without<Pawn>)>,
    mut ships : Query<(&Transform, &mut Velocity, &mut ExternalImpulse), With<Ship>>,
    input : Res<Input<Action>>,
    mut pawns : Query<(&mut Transform, &Pawn), (Without<Ship>, Without<Camera>)>,
    mut cameras : Query<&GlobalTransform, (Without<Ship>, With<Camera>)>
) {
    for (mut pilot_seat) in pilot_seats.iter_mut() {
        if pilot_seat.pawn.is_some() {
            let (ship_transform, mut ship_velocity, mut ship_impulse) = ships.iter_mut().next().unwrap();
            let forward = ship_transform.forward();
            let right = ship_transform.right();
            let up = ship_transform.up();
            let mut target_linvel = Vec3::ZERO;
            let speed = 100.0;
            if input.pressed(Action::Piloting(PilotingAction::MoveForward)) {
                target_linvel += forward * speed;
            }
            if input.pressed(Action::Piloting(PilotingAction::MoveBackward)) {
                target_linvel -= forward * speed;
            }
            ship_impulse.impulse = target_linvel;

            let mut angvel = -ship_velocity.angvel * 0.9;
            if input.pressed(Action::Piloting(PilotingAction::TurnUp)) {
                angvel += right;
            }
            if input.pressed(Action::Piloting(PilotingAction::TurnDown)) {
                angvel -= right;
            }
            if input.pressed(Action::Piloting(PilotingAction::TurnLeft)) {
                angvel += up;
            }
            if input.pressed(Action::Piloting(PilotingAction::TurnRight)) {
                angvel -= up;
            }
            if input.pressed(Action::Piloting(PilotingAction::RollLeft)) {
                angvel += forward;
            }
            if input.pressed(Action::Piloting(PilotingAction::RollRight)) {
                angvel -= forward;
            }
            ship_impulse.torque_impulse = angvel;

            if let Ok((mut pawn_tranform, pawn)) = pawns.get_mut(pilot_seat.pawn.as_ref().unwrap().pawn) {
                pawn_tranform.translation = PILOT_POSITION;
            }
        }
    }
}

fn pilot_debug_ui(
   mut pilot_seats : Query<(&mut PilotSeat), (Without<Pawn>)>,
   mut egui_ctxs : Query<&mut EguiContext>,
   mut ships : Query<(&Transform, &Velocity), With<Ship>>,
   mut pawns : Query<(&Transform, &Pawn)>,
) {

    let mut ctx = egui_ctxs.single_mut();
    egui::SidePanel::left("pilot_debug_ui").show(ctx.get_mut(), |ui| {
        for (mut pilot_seat) in pilot_seats.iter_mut() {
            if let Some(pawn) = &mut pilot_seat.pawn {
                let (ship_transform, ship_vel) = ships.iter().next().unwrap();
                ui.label(format!("Distance from world origin: {:.0}", ship_transform.translation.distance(Vec3::ZERO)));
                ui.label(format!("Ship velocity {:.2}", ship_vel.linvel.length()));
                ui.label(format!("Ship rotation velocity {:.2}", ship_vel.angvel.length()));
            }
        }
    });
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
    let Ok((seat_e, seat_tr, mut seat)) = pilot_seats.get_single_mut() else {
        return;
    };
    if input.just_pressed(Action::FPS(crate::control::FPSAction::Interact)) {
        if let Ok((e, mut tr)) = pawns.get_mut(e) {
            if let Some(cache) = &mut seat.pawn {
                commands.entity(cache.pawn).remove::<RigidBodyDisabled>().remove::<ColliderDisabled>().remove_parent();
                if let Ok((_, mut pawn_transform)) = pawns.get_mut(cache.pawn) {
                    pawn_transform.translation = cache.pawn_transform.translation;
                }
                seat.pawn = None;

            } else {
                let cache = PawnCache {
                    pawn : e,
                    pawn_transform : tr.clone(),
                };
                commands.entity(e).insert(RigidBodyDisabled).insert(ColliderDisabled);
                commands.entity(seat_e).add_child(e);
                tr.translation = PILOT_POSITION.clone();
                seat.pawn = Some(cache);
            }
        }
    }
}