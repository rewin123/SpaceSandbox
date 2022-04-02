use specs::*;
use specs::prelude::*;
use cgmath::*;

pub struct Pos(pub cgmath::Vector3<f32>);

impl Default for Pos {
    fn default() -> Self {
        Pos(cgmath::Vector3::<f32>::new(0.0,0.0,0.0))
    }
}

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

#[repr(C)]
pub struct DirectLight {
    pub dir : Vector3<f32>,
    pub color : Vector3<f32>,
}

impl Default for DirectLight {
    fn default() -> Self {
        Self { 
            dir : [0.0, 1.0, 0.0].into(),
            color : [1.0, 1.0, 1.0].into()
        }
    }
}

impl Component for DirectLight {
    type Storage = HashMapStorage<Self>;
}