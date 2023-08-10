use bevy::prelude::*;
use bevy_transform64::prelude::*;


#[derive(Component, Clone, Reflect, Default)]
#[reflect(Component)]
pub struct InstanceRotate {
    pub rot_steps : IVec3
}

pub fn prepare_instance_rotate(
    _query : Query<(&mut DTransform, &InstanceRotate), Added<InstanceRotate>>
) {
    // for (mut transform, rot) in query.iter_mut() {
    //     let xyz_euler = rot.rot_steps.as_vec3() * 90.0;
    //     transform.rotation = Quat::from_euler(EulerRot::XYZ, xyz_euler.x, xyz_euler.y, xyz_euler.z);
    // }
}