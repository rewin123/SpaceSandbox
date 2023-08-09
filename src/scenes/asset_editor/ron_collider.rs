use bevy::{prelude::*, math::{DVec3, DQuat}};
use bevy_proto::prelude::{Schematic, ReflectSchematic};
use crossbeam::epoch::Shared;
use serde::*;
use space_physics::prelude::{*, nalgebra::Vector3};

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
        let mut cols = self.colliders.iter().map(|c| c.into_shape()).collect::<Vec<(Isometry<Real>, SharedShape)>>();
        if cols.is_empty() {
            None
        } else {
            Some(ColliderBuilder::compound(cols).build())
        }
    }
}

impl RonCollider {
    pub fn into_shape(&self) -> (Isometry<Real>, SharedShape) {
        match self {
            RonCollider::Sphere(sphere) => sphere.into_shape(),
            RonCollider::Box(box_) => box_.into_shape()
        }
    }
}

impl RonSphereCollider {
    pub fn into_shape(&self) -> (Isometry<Real>, SharedShape) {
        let mut ball = SharedShape::ball(self.radius);
        (Vector3::new(self.position.x, self.position.y, self.position.z).into(), ball)
    }
}

impl RonBoxCollider {
    pub fn into_shape(&self) -> (Isometry<Real>, SharedShape) {
        let shape = SharedShape::cuboid(self.size.x as f64, self.size.y as f64, self.size.z as f64);
        let pos = Isometry::new(
            Vector3::new(self.position.x, self.position.y, self.position.z), 
            Vector3::new(self.rotation.x, self.rotation.y, self.rotation.z));
        (pos, shape)
    }
}