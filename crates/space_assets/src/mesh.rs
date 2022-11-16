use std::sync::{Arc, Mutex};
use crate::asset_server::{SpaceAsset, SpaceAssetServer};
use crate::handle::SpaceHandle;
use bevy::asset::Asset;
use bevy::prelude::Handle;
use bevy::reflect::TypeUuid;
use nalgebra::*;
use wgpu::util::DeviceExt;
use wgpu::{BufferUsages, VertexFormat};
use space_core::RenderBase;
use space_core::bevy::prelude::{Component, Bundle};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GVertex {
    pub pos : [f32; 3],
    pub normal : [f32; 3],
    pub tangent : [f32; 3],
    pub uv : [f32; 2]
}

impl GVertex {
    pub fn desc<'a>() -> Vec<wgpu::VertexBufferLayout<'a>> {
        vec![
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<GVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 4 * 3,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 4 * 3 * 2,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 4 * 3 * 3,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x2,
                },
            ],
        },
            wgpu::VertexBufferLayout {
                array_stride: 16 * 4 * 2,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 0,
                        shader_location: 4
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 4 * 4,
                        shader_location: 5
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 4 * 4 * 2,
                        shader_location: 6
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 4 * 4 * 3,
                        shader_location: 7
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 4 * 4 * 4,
                        shader_location: 8
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 4 * 4 * 5,
                        shader_location: 9
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 4 * 4 * 6,
                        shader_location: 10
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset : 4 * 4 * 7,
                        shader_location: 11
                    },
                ]
            }
        ]
    }
}

#[derive(TypeUuid)]
#[uuid="e0620f20-c2a1-4154-bb27-cc73a47a808c"]
pub struct GMesh {
    pub vertex : wgpu::Buffer,
    pub index : wgpu::Buffer,
    pub index_count : u32
}


#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LocationInstant {
    model : [[f32; 4]; 4],
    normal : [[f32; 4]; 4]
}

#[derive(Component)]
pub struct Location {
    pub pos : Vector3<f32>,
    pub rotation : Vector3<f32>,
    pub scale : Vector3<f32>,
    pub buffer : Arc<wgpu::Buffer>
}
impl Location {

    pub fn clone(&self, device : &wgpu::Device) -> Self {
        Self {
            pos : self.pos.clone(),
            rotation : self.rotation.clone(),
            scale : self.scale.clone(),
            buffer : Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &[0u8; 16 * 4 * 2],
                usage: BufferUsages::MAP_WRITE | BufferUsages::VERTEX
            }))
        }
    }

    pub fn new(device : &wgpu::Device) -> Self {
        Location {
            pos : [0.0, 0.0, 0.0].into(),
            rotation : [0.0, 0.0, 0.0].into(),
            scale : [1.0, 1.0, 1.0].into(),
            buffer : Arc::new(device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: &[0u8; 16 * 4 * 2],
                usage: BufferUsages::MAP_WRITE | BufferUsages::VERTEX
            }))
        }
    }

    pub fn update_buffer(&mut self) {
        let tr : Matrix4<f32> = Matrix::new_translation(&self.pos);
        let scale : Matrix4<f32> = Matrix::new_nonuniform_scaling(&self.scale);

        let rot = Rotation::from_euler_angles(self.rotation.x, self.rotation.y, self.rotation.z);
        let rot_mat : Matrix4<f32> = rot.into();

        let res = tr * rot_mat * scale;
        let normal = rot_mat * Matrix4::identity();

        let inst = LocationInstant {
            model : res.into(),
            normal : normal.into()
        };

        let buffer = self.buffer.clone();
        self.buffer.slice(..).map_async(wgpu::MapMode::Write, move |a| {
            buffer.slice(..).get_mapped_range_mut().copy_from_slice(bytemuck::cast_slice(&[inst]));
            buffer.unmap();
        });
    }
}



pub struct TextureBundle {
    pub texture : wgpu::Texture,
    pub view : wgpu::TextureView,
    pub sampler : wgpu::Sampler
}

impl TextureBundle {
    pub fn new(device : &wgpu::Device, desc : &wgpu::TextureDescriptor, filter : wgpu::FilterMode) -> Self {
        let texture = device.create_texture(desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: filter,
            min_filter: filter,
            mipmap_filter: filter,
            lod_min_clamp: 0.0,
            lod_max_clamp: 0.0,
            compare: None,
            anisotropy_clamp: None,
            border_color: None
        });
        Self {
            texture,
            view,
            sampler
        }
    }
}

#[derive(Component)]
pub struct MaterialPtr {
    pub handle : Handle<Material>
}

#[derive(Component, TypeUuid)]
#[uuid="a2b0c1bf-f725-48ef-9e40-66090d26e844"]
pub struct Material {
    pub color : SpaceHandle<TextureBundle>,
    pub normal : SpaceHandle<TextureBundle>,
    pub metallic_roughness: SpaceHandle<TextureBundle>,
    pub version_sum : u32,
    pub gbuffer_bind : Option<wgpu::BindGroup>
}

impl Clone for Material {
    fn clone(&self) -> Self {
        Material {
            color : self.color.clone(),
            normal : self.normal.clone(),
            metallic_roughness : self.metallic_roughness.clone(),
            version_sum : 0,
            gbuffer_bind : None
        }
    }
}

impl Material {
    pub fn need_rebind(&self, assets : &SpaceAssetServer) -> bool {
        if self.gbuffer_bind.is_none() {
            return true;
        } else {
            let sum = assets.get_version(&self.color).unwrap()
                + assets.get_version(&self.normal).unwrap()
                + assets.get_version(&self.metallic_roughness).unwrap();
            if sum != self.version_sum {
                return true;
            } else {
                return false;
            }

        }
    }
}


impl SpaceAsset for TextureBundle {

}