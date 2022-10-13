use std::sync::Arc;
use ash::vk;
use crate::{AllocatorSafe, BufferSafe, DeviceSafe, FramebufferPartial, RenderCamera, RenderPassSafe, TextureSafe, TextureView};

pub struct PointLightShadowMap {
    pub texture : Arc<TextureSafe>,
    pub cameras : Vec<RenderCamera>,
    pub framebuffer : Vec<FramebufferPartial>
}

pub struct PointLight {
    pub intensity : f32,
    pub position : [f32;3],
    pub color : [f32;3],
    pub instance : BufferSafe,
    pub shadow_map : Option<PointLightShadowMap>,
    pub shadow_enabled : bool
}

impl PointLight {


    pub fn default(allocator : &Arc<AllocatorSafe>,
                   device : &Arc<DeviceSafe>) -> Self {

        Self {
            intensity: 0.0,
            position: [0.0, 0.0, 0.0],
            color: [1.0, 1.0, 1.0],
            instance: BufferSafe::new(
                allocator,
                PointLight::get_instance_stride() as u64,
                vk::BufferUsageFlags::VERTEX_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu
            ).unwrap(),
            shadow_map: None,
            shadow_enabled : true
        }
    }


    pub fn get_shadow_map(
            allocator : &Arc<AllocatorSafe>,
            device : &Arc<DeviceSafe>,
            render_pass : &Arc<RenderPassSafe>) -> PointLightShadowMap {

        let texture = Arc::new(
            TextureSafe::new_depth_cubemap(
                    allocator,
                    device,
                    vk::Extent2D { width: 1024, height: 1024 },
                    false));

        //allocate cameras
        let mut cams = vec![];
        let mut fbs = vec![];
        for i in 0..6 {
            cams.push(RenderCamera::new(allocator));

            let fb = unsafe {

                let create_view_info = vk::ImageViewCreateInfo::builder()
                    .image(texture.image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(vk::Format::D32_SFLOAT)
                    .subresource_range(vk::ImageSubresourceRange::builder()
                        .base_array_layer(0)
                        .aspect_mask(vk::ImageAspectFlags::DEPTH)
                        .base_mip_level(0)
                        .level_count(1)
                        .base_array_layer(i as u32)
                        .layer_count(1)
                        .build());

                let view = device.create_image_view(
                    &create_view_info,
                    None).unwrap();

                let view_safe = TextureView {
                    view,
                    texture : texture.clone()
                };

                FramebufferPartial {
                    framebuffer: Default::default(),
                    renderpass: render_pass.clone(),
                    device: device.clone(),
                    views: vec![Arc::new(view_safe)]
                }
            };
            fbs.push(fb);
        }

        PointLightShadowMap {
            texture,
            cameras : cams,
            framebuffer: fbs
        }
    }



    pub fn get_instance_stride() -> u32 {
        (1 + 3 + 3) * 4
    }

    pub fn get_instance_vertex_attribs() ->
                                         Vec<vk::VertexInputAttributeDescription> {
        vec![
            vk::VertexInputAttributeDescription {
                binding : 3,
                location : 3,
                offset : 0,
                format: vk::Format::R32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding : 3,
                location : 4,
                offset : 4,
                format: vk::Format::R32G32B32_SFLOAT
            },
            vk::VertexInputAttributeDescription {
                binding : 3,
                location : 5,
                offset : 4 + 4 * 3,
                format: vk::Format::R32G32B32_SFLOAT
            },
        ]
    }

    pub fn fill_instanse(&mut self) {
        let mut data = vec![];
        data.push(self.intensity);
        data.extend(self.color);
        data.extend(self.position);
        self.instance.fill(&data);
    }
}