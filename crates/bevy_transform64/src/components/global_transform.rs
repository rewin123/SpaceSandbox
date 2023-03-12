use std::ops::Mul;

use bevy::math::*;
use bevy::prelude::*;

use super::DTransform;

#[derive(Component, Debug, PartialEq, Clone, Copy, Reflect, FromReflect, serde::Serialize, serde::Deserialize)]
#[reflect(Component, Default, PartialEq)]
pub struct DGlobalTransform (DAffine3);

impl Default for DGlobalTransform {
    fn default() -> Self {
        Self(DAffine3::IDENTITY)
    }
}

macro_rules! impl_local_axis {
    ($pos_name: ident, $neg_name: ident, $axis: ident) => {
        #[doc=std::concat!("Return the local ", std::stringify!($pos_name), " vector (", std::stringify!($axis) ,").")]
        #[inline]
        pub fn $pos_name(&self) -> DVec3 {
            (self.0.matrix3 * DVec3::$axis).normalize()
        }

        #[doc=std::concat!("Return the local ", std::stringify!($neg_name), " vector (-", std::stringify!($axis) ,").")]
        #[inline]
        pub fn $neg_name(&self) -> DVec3 {
            -self.$pos_name()
        }
    };
}


impl DGlobalTransform {
    /// An identity [`GlobalTransform`] that maps all points in space to themselves.
    pub const IDENTITY: Self = Self(DAffine3::IDENTITY);

    #[doc(hidden)]
    #[inline]
    pub fn from_xyz(x: f64, y: f64, z: f64) -> Self {
        Self::from_translation(DVec3::new(x, y, z))
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_translation(translation: DVec3) -> Self {
        DGlobalTransform(DAffine3::from_translation(translation))
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_rotation(rotation: DQuat) -> Self {
        DGlobalTransform(DAffine3::from_rotation_translation(rotation, DVec3::ZERO))
    }

    #[doc(hidden)]
    #[inline]
    pub fn from_scale(scale: DVec3) -> Self {
        DGlobalTransform(DAffine3::from_scale(scale))
    }

    /// Returns the 3d affine transformation matrix as a [`Mat4`].
    #[inline]
    pub fn compute_matrix(&self) -> DMat4 {
        DMat4::from(self.0)
    }

    /// Returns the 3d affine transformation matrix as an [`Affine3A`].
    #[inline]
    pub fn affine(&self) -> DAffine3 {
        self.0
    }

    /// Returns the transformation as a [`Transform`].
    ///
    /// The transform is expected to be non-degenerate and without shearing, or the output
    /// will be invalid.
    #[inline]
    pub fn compute_transform(&self) -> DTransform {
        let (scale, rotation, translation) = self.0.to_scale_rotation_translation();
        DTransform {
            translation,
            rotation,
            scale,
        }
    }

    
    #[inline]
    pub fn reparented_to(&self, parent: &DGlobalTransform) -> DTransform {
        let relative_affine = parent.affine().inverse() * self.affine();
        let (scale, rotation, translation) = relative_affine.to_scale_rotation_translation();
        DTransform {
            translation,
            rotation,
            scale,
        }
    }

    /// Extracts `scale`, `rotation` and `translation` from `self`.
    ///
    /// The transform is expected to be non-degenerate and without shearing, or the output
    /// will be invalid.
    #[inline]
    pub fn to_scale_rotation_translation(&self) -> (DVec3, DQuat, DVec3) {
        self.0.to_scale_rotation_translation()
    }

    impl_local_axis!(right, left, X);
    impl_local_axis!(up, down, Y);
    impl_local_axis!(back, forward, Z);

    /// Get the translation as a [`Vec3`].
    #[inline]
    pub fn translation(&self) -> DVec3 {
        self.0.translation.into()
    }

    /// Get the translation as a [`Vec3A`].
    #[inline]
    pub fn translation_vec3a(&self) -> DVec3 {
        self.0.translation
    }

    /// Get an upper bound of the radius from the given `extents`.
    #[inline]
    pub fn radius_vec3a(&self, extents: DVec3) -> f64 {
        (self.0.matrix3 * extents).length()
    }

    /// Transforms the given `point`, applying shear, scale, rotation and translation.
    ///
    /// This moves `point` into the local space of this [`GlobalTransform`].
    #[inline]
    pub fn transform_point(&self, point: DVec3) -> DVec3 {
        self.0.transform_point3(point)
    }

    /// Multiplies `self` with `transform` component by component, returning the
    /// resulting [`GlobalTransform`]
    #[inline]
    pub fn mul_transform(&self, transform: DTransform) -> Self {
        Self(self.0 * transform.compute_affine())
    }
}

impl From<DTransform> for DGlobalTransform {
    fn from(transform: DTransform) -> Self {
        Self(transform.compute_affine())
    }
}

impl From<DAffine3> for DGlobalTransform {
    fn from(affine: DAffine3) -> Self {
        Self(affine)
    }
}

impl From<DMat4> for DGlobalTransform {
    fn from(matrix: DMat4) -> Self {
        Self(DAffine3::from_mat4(matrix))
    }
}

impl Mul<DGlobalTransform> for DGlobalTransform {
    type Output = DGlobalTransform;

    #[inline]
    fn mul(self, global_transform: DGlobalTransform) -> Self::Output {
        DGlobalTransform(self.0 * global_transform.0)
    }
}

impl Mul<DTransform> for DGlobalTransform {
    type Output = DGlobalTransform;

    #[inline]
    fn mul(self, transform: DTransform) -> Self::Output {
        self.mul_transform(transform)
    }
}

impl Mul<DVec3> for DGlobalTransform {
    type Output = DVec3;

    #[inline]
    fn mul(self, value: DVec3) -> Self::Output {
        self.transform_point(value)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use bevy::math::EulerRot::XYZ;

    fn transform_equal(left: DGlobalTransform, right: DTransform) -> bool {
        left.0.abs_diff_eq(right.compute_affine(), 0.01)
    }

    #[test]
    fn reparented_to_transform_identity() {
        fn reparent_to_same(t1: DGlobalTransform, t2: DGlobalTransform) -> DTransform {
            t2.mul_transform(t1.into()).reparented_to(&t2)
        }
        let t1 = DGlobalTransform::from(DTransform {
            translation: DVec3::new(1034.0, 34.0, -1324.34),
            rotation: DQuat::from_euler(XYZ, 1.0, 0.9, 2.1),
            scale: DVec3::new(1.0, 1.0, 1.0),
        });
        let t2 = DGlobalTransform::from(DTransform {
            translation: DVec3::new(0.0, -54.493, 324.34),
            rotation: DQuat::from_euler(XYZ, 1.9, 0.3, 3.0),
            scale: DVec3::new(1.345, 1.345, 1.345),
        });
        let retransformed = reparent_to_same(t1, t2);
        assert!(
            transform_equal(t1, retransformed),
            "t1:{:#?} retransformed:{:#?}",
            t1.compute_transform(),
            retransformed,
        );
    }
    #[test]
    fn reparented_usecase() {
        let t1 = DGlobalTransform::from(DTransform {
            translation: DVec3::new(1034.0, 34.0, -1324.34),
            rotation: DQuat::from_euler(XYZ, 0.8, 1.9, 2.1),
            scale: DVec3::new(10.9, 10.9, 10.9),
        });
        let t2 = DGlobalTransform::from(DTransform {
            translation: DVec3::new(28.0, -54.493, 324.34),
            rotation: DQuat::from_euler(XYZ, 0.0, 3.1, 0.1),
            scale: DVec3::new(0.9, 0.9, 0.9),
        });
        // goal: find `X` such as `t2 * X = t1`
        let reparented = t1.reparented_to(&t2);
        let t1_prime = t2 * reparented;
        assert!(
            transform_equal(t1, t1_prime.into()),
            "t1:{:#?} t1_prime:{:#?}",
            t1.compute_transform(),
            t1_prime.compute_transform(),
        );
    }
}