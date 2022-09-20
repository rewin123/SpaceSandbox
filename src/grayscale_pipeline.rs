use std::sync::Arc;
use ash::vk;
use ash::vk::{DescriptorSet, Framebuffer};
use log::info;
use crate::{AllocatorSafe, DeviceSafe, ExamplePipeline, GPUMesh, GraphicBase, init_renderpass, RenderCamera, RenderPassSafe, SwapchainSafe};

pub struct GrayscalePipeline {
    pipeline : ExamplePipeline,
    descriptor_sets : Vec<DescriptorSet>,
    framebuffers : Vec<Framebuffer>,
    renderpass : RenderPassSafe,
    device : Arc<DeviceSafe>,
    allocator : Arc<AllocatorSafe>,
    descriptor_pool : vk::DescriptorPool
}

impl Drop for GrayscalePipeline {
    fn drop(&mut self) {
        unsafe {
            info!("Destroying grayscale pipeline...");
            self.device.device_wait_idle();
            for fb in &self.framebuffers {
                self.device.destroy_framebuffer(*fb, None);
            }
            self.device.destroy_descriptor_pool(self.descriptor_pool, None);
        }
    }
}

impl GrayscalePipeline {
    pub fn new(
        graphic_base : &GraphicBase,
        camera : &RenderCamera) -> Result<Self, vk::Result> {
        let mut renderpass = init_renderpass(&graphic_base).unwrap();
        let framebuffers = GrayscalePipeline::create_framebuffers(
            &graphic_base.device,
            renderpass.inner,
            &graphic_base.swapchain
        )?;

        let pipeline = ExamplePipeline::init(
            &graphic_base.device,
            &graphic_base.swapchain,
            &renderpass).unwrap();

        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty : vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count : graphic_base.swapchain.amount_of_images
            }
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .max_sets(graphic_base.swapchain.amount_of_images)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe {
            graphic_base.device.create_descriptor_pool(&descriptor_pool_info, None)
        }.unwrap();

        let desc_layouts =
            vec![pipeline.descriptor_set_layouts[0]; graphic_base.swapchain.amount_of_images as usize];
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts);
        let descriptor_sets =
            unsafe { graphic_base.device.allocate_descriptor_sets(&descriptor_set_allocate_info)
            }?;

        for (i, descset) in descriptor_sets.iter().enumerate() {
            let buffer_infos = [vk::DescriptorBufferInfo {
                buffer: camera.uniformbuffer.buffer,
                offset: 0,
                range: 128,
            }];
            let desc_sets_write = [vk::WriteDescriptorSet::builder()
                .dst_set(*descset)
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(&buffer_infos)
                .build()];
            unsafe { graphic_base.device.update_descriptor_sets(&desc_sets_write, &[]) };
        }

        Ok(Self {
            pipeline,
            descriptor_sets,
            framebuffers,
            renderpass,
            device : graphic_base.device.clone(),
            allocator : graphic_base.allocator.clone(),
            descriptor_pool
        })
    }

    fn create_framebuffers(
        logical_device: &ash::Device,
        renderpass: vk::RenderPass,
        swapchain : &SwapchainSafe
    ) -> Result<Vec<Framebuffer>, vk::Result> {
        let mut res = vec![];
        for iv in &swapchain.imageviews {
            let iview = [*iv, swapchain.depth_imageview];
            let framebuffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(renderpass)
                .attachments(&iview)
                .width(swapchain.extent.width)
                .height(swapchain.extent.height)
                .layers(1);
            let fb = unsafe { logical_device.create_framebuffer(&framebuffer_info, None) }?;
            res.push(fb);
        }
        Ok(res)
    }

    pub fn update_commandbuffer(
        &mut self,
        commandbuffer : vk::CommandBuffer,
        logical_device: &ash::Device,
        swapchain: &SwapchainSafe,
        meshes : &Vec<GPUMesh>,
        i : usize
    ) -> Result<(), vk::Result> {

        let clearvalues = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.08, 1.0],
            },
        },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.0,
                    stencil: 0,
                }
            }];
        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass.inner)
            .framebuffer(self.framebuffers[i])
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            })
            .clear_values(&clearvalues);
        unsafe {
            logical_device.cmd_begin_render_pass(
                commandbuffer,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
            logical_device.cmd_bind_pipeline(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.pipeline,
            );
            logical_device.cmd_bind_descriptor_sets(
                commandbuffer,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline.layout,
                0,
                &[self.descriptor_sets[i]],
                &[]
            );
            for mesh in meshes {
                logical_device.cmd_bind_vertex_buffers(commandbuffer, 0, &[mesh.pos_data.buffer, mesh.normal_data.buffer], &[0, 0]);
                logical_device.cmd_bind_index_buffer(commandbuffer, mesh.index_data.buffer, 0, vk::IndexType::UINT32);
                logical_device.cmd_draw_indexed(commandbuffer, mesh.vertex_count, 1, 0, 0, 0);
            }

            logical_device.cmd_end_render_pass(commandbuffer);
            // logical_device.end_command_buffer(commandbuffer)?;
        }

        Ok(())
    }

}