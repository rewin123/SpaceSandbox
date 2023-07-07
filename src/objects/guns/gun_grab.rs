use std::f64::consts::PI;

use bevy::{prelude::*, math::DVec3};
use bevy_transform64::prelude::DTransform;


pub struct GunGrabPlugin;

impl Plugin for GunGrabPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(gun_grab);
    }
}

#[derive(Component)]
pub struct GunGrab {
    pub cam_id : Entity,
    pub gun_id : Entity,
    pub shift : DVec3
}

fn gun_grab(
    mut query : Query<(&DTransform, &GunGrab)>,
    mut transforms : Query<&mut DTransform, (Without<GunGrab>, Without<Camera>)>,
    mut cam_transforms : Query<&DTransform, (With<Camera>, Without<GunGrab>)>,
) {
    for (transform, gun_grab) in query.iter_mut() {
        if let Ok(mut gun_transform) = transforms.get_mut(gun_grab.gun_id) {
           if let Ok(cam_transform) = cam_transforms.get_mut(gun_grab.cam_id) {
               let frw = cam_transform.forward();
               let right = cam_transform.right();
               let up = cam_transform.up();
               gun_transform.rotation = cam_transform.rotation;
               gun_transform.rotate_axis(DVec3::X, PI / 2.0);

               info!("Cam frw: {:?} right: {:?} up: {:?} ", frw, right, up);
               gun_transform.translation = right * gun_grab.shift.x + up * gun_grab.shift.y + frw * gun_grab.shift.z;
           }
        }
    }
}