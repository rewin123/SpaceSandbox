
#[derive(Default, Copy, Clone, Debug)]
pub struct Vec2 {
    pub position: [f32; 2]
}


vulkano::impl_vertex!(Vec2, position);