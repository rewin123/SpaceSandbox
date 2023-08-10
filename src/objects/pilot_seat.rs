


use bevy::{prelude::*, math::DVec3};
use bevy_egui::*;
use bevy_proto::prelude::{Schematic, ReflectSchematic};
use bevy_transform64::prelude::DTransform;
use bevy_xpbd_3d::prelude::{LinearVelocity};
use bevy_xpbd_3d::prelude::*;

use crate::{pawn_system::{ChangePawn, Pawn, CurrentPawn}, control::{Action, PilotingAction}, ship::Ship, scenes::{settings::settings_system, fps_mode::IsFPSMode}};

use super::ship_camera::ShipCamera;

struct PawnCache {
    pawn : Entity,
    pawn_transform : DTransform,
}

#[derive(Component, Default, Reflect, Schematic)]
#[reflect(Component, Schematic)]
pub struct PilotSeat {
    #[reflect(ignore)]
    pawn : Option<PawnCache>,
    #[reflect(ignore)]
    cameras : Vec<Entity>,
    #[reflect(ignore)]
    current_camera : Option<usize>,
}

pub struct PilotSeatPlugin;

const PILOT_POSITION : DVec3 = DVec3::new(0.0, 0.5, 0.0);

impl Plugin for PilotSeatPlugin {
    fn build(&self, app: &mut App) {

        app.add_systems(
            Update,
            seat_in_pilot_seat.run_if(in_state(IsFPSMode::Yes))
        );
        app.add_systems(
            Update,
            pilot_debug_ui.after(settings_system).run_if(in_state(IsFPSMode::Yes))
        );
        app.add_systems(
            Update,
            piloting.run_if(in_state(IsFPSMode::Yes))
        );

        app.register_type::<PilotSeat>();
    }
}

fn camera_selection(
    _cameras : Query<&mut Transform, With<ShipCamera>>,
) {

}

fn piloting(
    mut pilot_seats : Query<(&DTransform, &mut PilotSeat), Without<Pawn>>,
    mut ships : Query<(&DTransform, &mut LinearVelocity, &mut AngularVelocity, &mut ExternalForce, &mut ExternalTorque), With<Ship>>,
    input : Res<Input<Action>>,
    mut pawns : Query<(&mut DTransform, &Pawn), (Without<Ship>, Without<ShipCamera>)>,
    cameras : Query<&DTransform, (Without<Ship>, With<ShipCamera>)>
) {
    for (pilot_seat_transform, mut pilot_seat) in pilot_seats.iter_mut() {
        if pilot_seat.pawn.is_some() {
            let (ship_transform, _ship_velocity, mut ship_angular, mut ship_impulse, _ship_torgue) = ships.iter_mut().next().unwrap();
            let forward = ship_transform.forward();
            let right = ship_transform.right();
            let up = ship_transform.up();
            let mut target_linvel = DVec3::ZERO;
            let speed = 100.0;
            if input.pressed(Action::Piloting(PilotingAction::MoveForward)) {
                target_linvel += forward * speed;
            }
            if input.pressed(Action::Piloting(PilotingAction::MoveBackward)) {
                target_linvel -= forward * speed;
            }
            ship_impulse.apply_force(target_linvel);

            let mut angvel = -ship_angular.0 * 0.9;
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
            ship_angular.0 += angvel;

            if let Ok((mut pawn_tranform, _pawn)) = pawns.get_mut(pilot_seat.pawn.as_ref().unwrap().pawn) {
                if pilot_seat.current_camera.is_none() {
                    pawn_tranform.translation = PILOT_POSITION + pilot_seat_transform.translation;
                } else {
                    let camera_transform = cameras.get(pilot_seat.cameras[pilot_seat.current_camera.unwrap()]).unwrap();
                    pawn_tranform.translation = camera_transform.translation;
                }
            }

            if input.just_pressed(Action::Piloting(PilotingAction::GoToNextCamera)) {
                pilot_seat.current_camera = if pilot_seat.current_camera.is_none() {
                    Some(0)
                } else {
                    Some((pilot_seat.current_camera.unwrap() + 1) % pilot_seat.cameras.len())
                }
            }

            if input.just_pressed(Action::Piloting(PilotingAction::BackToSeat)) {
                pilot_seat.current_camera = None;
            }
        }
    }
}

fn pilot_debug_ui(
   mut pilot_seats : Query<&mut PilotSeat, Without<Pawn>>,
   mut egui_ctxs : Query<&mut EguiContext>,
   ships : Query<(&DTransform, &LinearVelocity), With<Ship>>,
   _pawns : Query<(&DTransform, &Pawn)>,
) {

    let mut ctx = egui_ctxs.single_mut();
    egui::SidePanel::left("pilot_debug_ui").show(ctx.get_mut(), |ui| {
        for mut pilot_seat in pilot_seats.iter_mut() {
            if let Some(_pawn) = &mut pilot_seat.pawn {
                let (ship_transform, ship_vel) = ships.iter().next().unwrap();
                ui.label(format!("Distance from world origin: {:.0}", ship_transform.translation.distance(DVec3::ZERO)));
                ui.label(format!("Ship velocity {:.2}", ship_vel.length()));
                // ui.label(format!("Ship rotation velocity {:.2}", ship_vel.angvel.length()));

                ui.label(format!("Camera count: {}", pilot_seat.cameras.len()));
                ui.label(format!("Current camera: {:?}", pilot_seat.current_camera));
            }
        }
    });
}

fn seat_in_pilot_seat(
    _commands : Commands,
    _change_pawn : EventWriter<ChangePawn>,
    _input : ResMut<Input<Action>>,
    _pawns : Query<(Entity, &mut DTransform), With<Pawn>>,
    _current_pawn : ResMut<CurrentPawn>,
    _pilot_seats : Query<(Entity, &DTransform, &mut PilotSeat), Without<Pawn>>,
    _cameras : Query<Entity, (Without<Ship>, With<ShipCamera>)>,
    _ships : Query<Entity, (With<Ship>, Without<Pawn>)>
) {
    // let Some(e) = current_pawn.id else {
    //     return;
    // };
    // let Ok((seat_e, seat_tr, mut seat)) = pilot_seats.get_single_mut() else {
    //     return;
    // };
    // if input.just_pressed(Action::FPS(crate::control::FPSAction::Interact)) {
    //     info!("Interact with pilot seat");
    //     if let Ok((e, mut tr)) = pawns.get_mut(e) {
    //         if let Some(cache) = &mut seat.pawn {
    //             commands.entity(cache.pawn).remove::<RigidBodyDisabled>().remove::<ColliderDisabled>().remove_parent();
    //             if let Ok((_, mut pawn_transform)) = pawns.get_mut(cache.pawn) {
    //                 pawn_transform.translation = cache.pawn_transform.translation;
    //             }
    //             seat.pawn = None;

    //         } else {
    //             let cache = PawnCache {
    //                 pawn : e,
    //                 pawn_transform : tr.clone(),
    //             };
    //             commands.entity(e).remove::<RigidBody>().remove::<Collider>();

    //             let ship_e = ships.single_mut();
    //             commands.entity(ship_e).add_child(e);
    //             tr.translation = seat_tr.translation + PILOT_POSITION.clone();
    //             seat.pawn = Some(cache);

    //             seat.cameras.clear();
    //             seat.cameras.extend(cameras.iter());
    //             seat.current_camera = None;
    //         }
    //     }
    // }
}