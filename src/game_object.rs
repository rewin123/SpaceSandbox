use std::sync::Arc;

use specs::*;
use specs::prelude::*;
use cgmath::*;
use vulkano::{image::{StorageImage, AttachmentImage}, format::Format};

use crate::rpu::RPU;

pub struct Pos(pub cgmath::Vector3<f32>);

impl Default for Pos {
    fn default() -> Self {
        Pos(cgmath::Vector3::<f32>::new(0.0,0.0,0.0))
    }
}

impl Component for Pos {
    type Storage = VecStorage<Self>;
}

#[derive(Clone, Debug)]
pub struct DirectLightTextures {
    pub pos_img : Arc<StorageImage>,
    pub depth_img : Arc<AttachmentImage>,
}

#[repr(C)]
pub struct DirectLight {
    pub dir : Vector3<f32>,
    pub color : Vector3<f32>,
    pub intensity : f32, //in luks
    pub textures : Option<DirectLightTextures>
}


impl Default for DirectLight {
    fn default() -> Self {
        Self { 
            dir : [0.0, 1.0, 0.0].into(),
            color : [1.0, 1.0, 1.0].into(),
            intensity : 100000.0, 
            textures : None,
        }
    }
}

impl DirectLight {
    pub fn AllocTextures(&mut self, rpu : RPU, w : u32, h : u32) {
        let cam_pos_img = rpu.create_image(w, h, Format::R32G32B32A32_SFLOAT).unwrap();
        
        let depth_img = 
            AttachmentImage::transient(rpu.device.clone(), [w, h], Format::D16_UNORM).unwrap();

        let texs = DirectLightTextures {
            pos_img : cam_pos_img,
            depth_img
        };
        self.textures = Some(texs);
    }
}

impl Component for DirectLight {
    type Storage = HashMapStorage<Self>;
}