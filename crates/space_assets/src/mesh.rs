use specs::{Component, VecStorage};
use crate::asset_server::{Asset, AssetServer};
use crate::handle::Handle;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct GVertex {
    pub pos : [f32; 3],
    pub normal : [f32; 3],
    pub tangent : [f32; 3],
    pub uv : [f32; 2]
}

impl GVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
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
        }
    }
}

pub struct GMesh {
    pub vertex : wgpu::Buffer,
    pub index : wgpu::Buffer,
    pub index_count : u32
}

impl Component for GMesh {
    type Storage = VecStorage<GMesh>;
}



pub struct TextureBundle {
    pub texture : wgpu::Texture,
    pub view : wgpu::TextureView,
    pub sampler : wgpu::Sampler
}

impl TextureBundle {
    pub fn new(device : &wgpu::Device, desc : &wgpu::TextureDescriptor) -> Self {
        let texture = device.create_texture(desc);
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: None,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
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

pub struct Material {
    pub color : Handle<TextureBundle>,
    pub normal : Handle<TextureBundle>,
    pub metallic_roughness: Handle<TextureBundle>,
    pub version_sum : u32,
    pub gbuffer_bind : Option<wgpu::BindGroup>
}

impl Material {
    pub fn need_rebind(&self, assets : &AssetServer) -> bool {
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

impl Component for Material {
    type Storage = VecStorage<Material>;
}

impl Asset for TextureBundle {

}