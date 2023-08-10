use bevy::{prelude::*, math::{DVec3, DQuat}};
use bevy_proto::prelude::{Schematic, ReflectSchematic};
use bevy_xpbd_3d::{parry::{shape::SharedShape}, prelude::Collider};



#[derive(Component, Reflect, Schematic)]
#[reflect(Schematic)]
pub struct RonColliderCompound {
    pub colliders : Vec<RonCollider>,
}

#[derive(Reflect)]
pub enum RonCollider {
    Sphere(RonSphereCollider),
    Box(RonBoxCollider),
}

#[derive(Reflect)]
pub struct RonSphereCollider {
    pub position : DVec3,
    pub radius : f64
}

#[derive(Reflect)]
pub struct RonBoxCollider {
    pub position : DVec3,
    pub rotation : DVec3,
    pub size : DVec3
}

impl RonColliderCompound {
    pub fn into_collider(&self) -> Option<Collider> {
        let cols = self.colliders.iter().map(|c| c.into_shape()).collect::<Vec<_>>();
        if cols.is_empty() {
            None
        } else {
            Some(Collider::compound(cols))
        }
    }
}

impl RonCollider {
    pub fn into_shape(&self) -> (DVec3, DQuat, SharedShape) {
        match self {
            RonCollider::Sphere(sphere) => sphere.into_shape(),
            RonCollider::Box(box_) => box_.into_shape()
        }
    }
}

impl RonSphereCollider {
    pub fn into_shape(&self) -> (DVec3, DQuat, SharedShape) {
        let ball = SharedShape::ball(self.radius);
        (DVec3::new(self.position.x, self.position.y, self.position.z), DQuat::default(), ball)
    }
}

impl RonBoxCollider {
    pub fn into_shape(&self) -> (DVec3, DQuat, SharedShape) {
        let shape = SharedShape::cuboid(self.size.x, self.size.y, self.size.z);
        (self.position, DQuat::from_euler(EulerRot::XYZ, self.rotation.x, self.rotation.y, self.rotation.z), shape)
    }
}