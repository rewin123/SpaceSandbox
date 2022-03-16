
#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Vec2 {
    pub data: [f32; 2]
}
vulkano::impl_vertex!(Vec2, data);

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Vec3 {
    pub data: [f32; 3]
}
vulkano::impl_vertex!(Vec3, data);

#[repr(C)]
#[derive(Default, Copy, Clone, Debug)]
pub struct Vec4 {
    pub data: [f32; 4]
}
vulkano::impl_vertex!(Vec4, data);

impl Vec3 {
    pub fn new(x : f32, y : f32, z : f32) -> Self {
        Self { 
            data : [x, y, z]
        }
    }
}