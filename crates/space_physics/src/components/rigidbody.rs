use bevy::{prelude::*, math::DVec3};
use bevy_transform64::prelude::DTransform;
use rapier3d_f64::prelude::{LockedAxes as RapierLockedAxes, RigidBodyHandle};

#[derive(Component)]
pub struct RapierRigidBodyHandle(pub RigidBodyHandle);


#[derive(Component, Debug)]
pub enum SpaceRigidBodyType {
    Dynamic,
    Fixed
}

impl SpaceRigidBodyType {
    pub fn to_rapier(&self) -> rapier3d_f64::prelude::RigidBodyType {
        match self {
            SpaceRigidBodyType::Dynamic => rapier3d_f64::prelude::RigidBodyType::Dynamic,
            SpaceRigidBodyType::Fixed => rapier3d_f64::prelude::RigidBodyType::Fixed
        }
    }
}

#[derive(Component, Debug, Default)]
pub struct Velocity {
    pub linvel : DVec3,
    pub angvel : DVec3
}

bitflags::bitflags! {
    #[derive(Default, Component, Reflect, FromReflect)]
    #[reflect(Component, PartialEq)]
    /// Flags affecting the behavior of the constraints solver for a given contact manifold.
    pub struct SpaceLockedAxes: u8 {
        /// Flag indicating that the rigid-body cannot translate along the `X` axis.
        const TRANSLATION_LOCKED_X = 1 << 0;
        /// Flag indicating that the rigid-body cannot translate along the `Y` axis.
        const TRANSLATION_LOCKED_Y = 1 << 1;
        /// Flag indicating that the rigid-body cannot translate along the `Z` axis.
        const TRANSLATION_LOCKED_Z = 1 << 2;
        /// Flag indicating that the rigid-body cannot translate along any direction.
        const TRANSLATION_LOCKED = Self::TRANSLATION_LOCKED_X.bits | Self::TRANSLATION_LOCKED_Y.bits | Self::TRANSLATION_LOCKED_Z.bits;
        /// Flag indicating that the rigid-body cannot rotate along the `X` axis.
        const ROTATION_LOCKED_X = 1 << 3;
        /// Flag indicating that the rigid-body cannot rotate along the `Y` axis.
        const ROTATION_LOCKED_Y = 1 << 4;
        /// Flag indicating that the rigid-body cannot rotate along the `Z` axis.
        const ROTATION_LOCKED_Z = 1 << 5;
        /// Combination of flags indicating that the rigid-body cannot rotate along any axis.
        const ROTATION_LOCKED = Self::ROTATION_LOCKED_X.bits | Self::ROTATION_LOCKED_Y.bits | Self::ROTATION_LOCKED_Z.bits;
    }
}

impl From<SpaceLockedAxes> for RapierLockedAxes {
    fn from(locked_axes: SpaceLockedAxes) -> RapierLockedAxes {
        RapierLockedAxes::from_bits(locked_axes.bits).expect("Internal conversion error.")
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Component, Reflect, FromReflect)]
#[reflect(Component, PartialEq)]
pub struct ExternalImpulse {
    /// The linear force applied to the rigid-body.
    pub impulse: DVec3,
    pub torque_impulse: DVec3,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Component, Reflect, FromReflect)]
#[reflect(Component, PartialEq)]
pub struct RigidBodyDisabled;

#[derive(Copy, Clone, Debug, Default, PartialEq, Component, Reflect, FromReflect)]
#[reflect(Component, PartialEq)]
pub struct ColliderDisabled;

#[derive(Copy, Clone, Debug, Default, PartialEq, Component, Reflect, FromReflect)]
#[reflect(Component, PartialEq)]
pub struct GravityScale(pub f64);

#[derive(Copy, Clone, Debug, Default, PartialEq, Component, Reflect, FromReflect)]
#[reflect(Component, PartialEq)]
pub struct SpaceDominance(pub i8);