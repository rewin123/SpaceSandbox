use std::num::NonZeroU32;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use space_shaders::{PointLightUniform, ShaderUniform};
use nalgebra as na;
use wgpu::{TextureDimension, TextureFormat};
use wgpu::util::DeviceExt;
use winit::event::VirtualKeyCode::Mute;
use crate::RenderBase;

use space_shaders::LightCamera;



pub struct PointLight {
    inner : PointLightUniform,
    pub buffer : Arc<wgpu::Buffer>,
    pub shadow : Option<PointLightShadow>,
}

impl PointLight {
    pub fn new(
        render : &Arc<RenderBase>,
        position : na::Vector3<f32>,
        shadow : bool
    ) -> Self {

        let inner = PointLightUniform {
            pos: position,
            color: [1.0, 1.0, 1.0].into(),
            intensity: 10.0,
        };
        let buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Light buffer"),
            contents: &inner.get_bytes().unwrap(),
            usage: wgpu::BufferUsages::COPY_DST
                | wgpu::BufferUsages::UNIFORM
                | wgpu::BufferUsages::MAP_WRITE,
        });

        let mut shadow_struct = None;
        if shadow {
            shadow_struct = Some(PointLightShadow::new(render, 1024));
        }
        
        Self { 
            inner,
            buffer : Arc::new(buffer),
            shadow : shadow_struct
        }
    }

    pub fn update_buffer(&mut self, render : &RenderBase) {
        let inned_data = self.inner.get_bytes().unwrap();
        let buffer = self.buffer.clone();
        self.buffer.slice(..).map_async(wgpu::MapMode::Write, move|_|{
            buffer.slice(..).get_mapped_range_mut().copy_from_slice(&inned_data);
            buffer.unmap();
        });

        if let Some(shadow) = self.shadow.as_mut() {
            for idx in 0..6 {
                shadow.cameras_unforms[idx].pos = self.inner.pos.clone();

                let inned_data = shadow.cameras_unforms[idx].get_bytes().unwrap();
                let buffer = shadow.camera_buffers[idx].clone();

                shadow.camera_buffers[idx].slice(..)
                    .map_async(wgpu::MapMode::Write, move |_| {
                        buffer.slice(..).get_mapped_range_mut().copy_from_slice(&inned_data);
                        buffer.unmap();
                });
            }
        }
    }
}

impl Deref for PointLight {
    type Target = PointLightUniform;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for PointLight {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub struct PointLightShadow {
    pub tex : wgpu::Texture,
    pub side_views : Vec<wgpu::TextureView>,
    pub cube_view : wgpu::TextureView,
    pub pipeline_bind : Option<wgpu::BindGroup>,
    pub camera_binds : Vec<wgpu::BindGroup>,
    pub camera_buffers : Vec<Arc<wgpu::Buffer>>,
    pub cameras_unforms : Vec<LightCamera>,
    pub sampler : wgpu::Sampler
}

impl PointLightShadow {
    pub fn new(render : &Arc<RenderBase>, size : u32) -> Self {

        let tex = render.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth texture"),
            size: wgpu::Extent3d {
                width : size,
                height : size,
                depth_or_array_layers : 6
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT
        });

        let mut side_views = vec![];
        for idx in 0..6 {
            let view = tex.create_view(&wgpu::TextureViewDescriptor {
                label: Some("point light side view"),
                format: None,
                dimension: Some(wgpu::TextureViewDimension::D2),
                aspect: Default::default(),
                base_mip_level: 0,
                mip_level_count: Some(NonZeroU32::new(1).unwrap()),
                base_array_layer: idx,
                array_layer_count: Some(NonZeroU32::new(1).unwrap())
            });
            side_views.push(view)
        }
        
        let cube_view = tex.create_view(&wgpu::TextureViewDescriptor {
            label: Some("point light cube view"),
            format: None,
            dimension: Some(wgpu::TextureViewDimension::Cube),
            aspect: Default::default(),
            base_mip_level: 0,
            mip_level_count: None,
            base_array_layer: 0,
            array_layer_count: Some(NonZeroU32::new(6).unwrap())
        });

        let sampler = render.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Shadow cube sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None
        });

        let mut cameras = vec![];
        let mut camera_buffers = vec![];
        for idx in 0..6 {

            let up;
            let frw;
            if idx == 0 {
                frw = na::Vector3::<f32>::new(-1.0, 0.0, 0.0);
                up = na::Vector3::<f32>::new(0.0, 1.0, 0.0);
            } else if idx == 1 {
                frw = na::Vector3::<f32>::new(1.0, 0.0, 0.0);
                up = na::Vector3::<f32>::new(0.0, 1.0, 0.0);
            } else if idx == 2 {
                frw = na::Vector3::<f32>::new(0.0, -1.0, 0.0);
                up = na::Vector3::<f32>::new(0.0, 0.0, 1.0);
            } else if idx == 3 {
                frw = na::Vector3::<f32>::new(0.0, 1.0, 0.0);
                up = na::Vector3::<f32>::new(0.0, 0.0, -1.0);
            } else if idx == 4 {
                frw = na::Vector3::<f32>::new(0.0, 0.0, -1.0);
                up = na::Vector3::<f32>::new(0.0, 1.0, 0.0);
            } else if idx == 5 {
                frw = na::Vector3::<f32>::new(0.0, 0.0, 1.0);
                up = na::Vector3::<f32>::new(0.0, 1.0, 0.0);
            } else {
                panic!();
            }


            let cam = LightCamera {
                proj : nalgebra::Matrix4::<f32>::new_perspective(
                    1.0f32, std::f32::consts::PI / 2.0, 0.01f32, 100000.0f32),
                pos : [0.0, 0.0, 0.0].into(),
                frw,
                up,
                far : 10000.0
            };


            let buffer = render.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Light camera uniform"),
                contents: &cam.get_bytes().unwrap(),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::MAP_WRITE
            });

            cameras.push(cam);
            camera_buffers.push(Arc::new(buffer));
        }


        Self {
            tex,
            side_views,
            cube_view,
            pipeline_bind: None,
            camera_binds : vec![],
            camera_buffers,
            cameras_unforms : cameras,
            sampler
        }
    }
}