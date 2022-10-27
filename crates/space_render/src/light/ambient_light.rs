
use encase::*;
use crate::pipelines::TextureTransformUniform;

#[derive(ShaderType, Default)]
pub struct AmbientLightUniform {
    pub color : nalgebra::Vector3<f32>,
    pub cam_pos : nalgebra::Vector3<f32>
}

impl TextureTransformUniform for AmbientLightUniform {
    fn get_bytes(&self) -> Vec<u8> {
        let mut uniform = encase::UniformBuffer::new(vec![]);
        uniform.write(&self).unwrap();
        uniform.into_inner()
    }
}

pub struct AmbientLight {
    pub color : nalgebra::Vector3<f32>,
}

