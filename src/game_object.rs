use specs::*;
use specs::prelude::*;

pub struct Pos(cgmath::Vector3<f32>);

impl Default for Pos {
    fn default() -> Self {
        Pos(cgmath::Vector3::<f32>::new(0.0,0.0,0.0))
    }
}

impl Component for Pos {
    type Storage = VecStorage<Self>;
}