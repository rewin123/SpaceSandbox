use std::collections::HashMap;
use std::sync::Arc;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{CommandBuffer, DescriptorSet, Framebuffer};
use log::*;
use crate::{AllocatorSafe, DeviceSafe, GraphicBase, init_renderpass, RenderCamera, RenderModel, RenderPassSafe, SwapchainSafe, DescriptorPoolSafe, TextureServer, MaterialTexture, FramebufferStorage, InstancesDrawer, TextureSafe, RenderServer, ServerTexture};
use ash::vk::DescriptorSetLayout;
use crate::asset_server::AssetServer;


pub struct GBufferFillPipeline {
    descriptor_sets : Vec<DescriptorSet>,
    framebuffers : FramebufferStorage,
    renderpass : Arc<RenderPassSafe>,
    device : Arc<DeviceSafe>,
    descriptor_pool : Arc<DescriptorPoolSafe>,
    descriptor_sets_texture : HashMap<usize, DescriptorSet>,
    pub mode : MaterialTexture,
    
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptor_set_layouts : Vec<DescriptorSetLayout>,
}


impl Drop for GBufferFillPipeline {
    fn drop(&mut self) {
        unsafe {
            info!("Destroying grayscale pipeline...");
            self.device.device_wait_idle();

            unsafe {
                for dsl in &self.descriptor_set_layouts {
                    self.device.destroy_descriptor_set_layout(*dsl, None);
                }
                self.device.destroy_pipeline(self.pipeline, None);
                self.device.destroy_pipeline_layout(self.layout, None);
            }
        }
    }
}


impl GBufferFillPipeline {

    fn get_img_desc_set(logical_device : Arc<DeviceSafe>) -> vk::DescriptorSetLayout {
        let descriptorset_layout_binding_descs = [vk::DescriptorSetLayoutBinding::builder()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .build()];
        let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(&descriptorset_layout_binding_descs);
        let descriptorsetlayout = unsafe {
            logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
        }.unwrap();
        descriptorsetlayout
    }

    fn init_base_pipeline(
        logical_device: &Arc<DeviceSafe>,
        swapchain: &SwapchainSafe,
        renderpass: &RenderPassSafe) -> Result<(vk::Pipeline, vk::PipelineLayout, Vec<vk::DescriptorSetLayout>), Box<dyn std::error::Error>> {
            let vertexshader_createinfo = vk::ShaderModuleCreateInfo::builder().code(
                vk_shader_macros::include_glsl!("./shaders/gbuffer_fill/shader.vert", kind: vert),
            );
            let vertexshader_module =
                unsafe { logical_device.create_shader_module(&vertexshader_createinfo, None)? };
            let fragmentshader_createinfo = vk::ShaderModuleCreateInfo::builder()
                .code(vk_shader_macros::include_glsl!("./shaders/gbuffer_fill/shader.frag"));
            let fragmentshader_module =
                unsafe { logical_device.create_shader_module(&fragmentshader_createinfo, None)? };
            let mainfunctionname = std::ffi::CString::new("main").unwrap();
            let vertexshader_stage = vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(vertexshader_module)
                .name(&mainfunctionname);
            let fragmentshader_stage = vk::PipelineShaderStageCreateInfo::builder()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(fragmentshader_module)
                .name(&mainfunctionname);
            let shader_stages = vec![vertexshader_stage.build(), fragmentshader_stage.build()];
    
            let vertex_attrib_descs = [vk::VertexInputAttributeDescription {
                    binding: 0,
                    location: 0,
                    offset: 0,
                    format: vk::Format::R32G32B32_SFLOAT,
                },
                vk::VertexInputAttributeDescription {
                    binding: 1,
                    location: 1,
                    offset: 0,
                    format: vk::Format::R32G32B32_SFLOAT
                },
                vk::VertexInputAttributeDescription {
                    binding: 2,
                    location: 2,
                    offset: 0,
                    format: vk::Format::R32G32_SFLOAT
                },
    
                //define instance buffer
                vk::VertexInputAttributeDescription {
                    binding: 3,
                    location: 3,
                    offset: 0,
                    format: vk::Format::R32G32B32A32_SFLOAT
                },
                vk::VertexInputAttributeDescription {
                    binding: 3,
                    location: 4,
                    offset: 16,
                    format: vk::Format::R32G32B32A32_SFLOAT
                },
                vk::VertexInputAttributeDescription {
                    binding: 3,
                    location: 5,
                    offset: 32,
                    format: vk::Format::R32G32B32A32_SFLOAT
                },
                vk::VertexInputAttributeDescription {
                    binding: 3,
                    location: 6,
                    offset: 48,
                    format: vk::Format::R32G32B32A32_SFLOAT
                },];
    
            let vertex_binding_descs = [vk::VertexInputBindingDescription {
                binding: 0,
                stride: 4 * 3,
                input_rate: vk::VertexInputRate::VERTEX,
            },
                vk::VertexInputBindingDescription {
                    binding: 1,
                    stride: 4 * 3,
                    input_rate: vk::VertexInputRate::VERTEX
                },
                vk::VertexInputBindingDescription {
                    binding: 2,
                    stride: 4 * 2,
                    input_rate: vk::VertexInputRate::VERTEX
                },
                vk::VertexInputBindingDescription {
                    binding: 3,
                    stride: 4 * 16,
                    input_rate: vk::VertexInputRate::INSTANCE
                }];
    
            let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
                .vertex_attribute_descriptions(&vertex_attrib_descs)
                .vertex_binding_descriptions(&vertex_binding_descs);
    
            let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
                .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
            let viewports = [vk::Viewport {
                x: 0.,
                y: 0.,
                width: swapchain.extent.width as f32,
                height: swapchain.extent.height as f32,
                min_depth: 0.,
                max_depth: 1.,
            }];
            let scissors = [vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: swapchain.extent,
            }];
    
            let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
                .viewports(&viewports)
                .scissors(&scissors);
            let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
                .line_width(1.0)
                .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
                .cull_mode(vk::CullModeFlags::NONE)
                .polygon_mode(vk::PolygonMode::FILL);
            let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
                .rasterization_samples(vk::SampleCountFlags::TYPE_1);
            let colourblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .color_blend_op(vk::BlendOp::ADD)
                .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                .alpha_blend_op(vk::BlendOp::ADD)
                .color_write_mask(
                    vk::ColorComponentFlags::R
                        | vk::ColorComponentFlags::G
                        | vk::ColorComponentFlags::B
                        | vk::ColorComponentFlags::A,
                )
                .build(),
                vk::PipelineColorBlendAttachmentState::builder()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(
                        vk::ColorComponentFlags::R
                            | vk::ColorComponentFlags::G
                            | vk::ColorComponentFlags::B
                            | vk::ColorComponentFlags::A,
                    )
                    .build(),
                vk::PipelineColorBlendAttachmentState::builder()
                    .blend_enable(true)
                    .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .color_blend_op(vk::BlendOp::ADD)
                    .src_alpha_blend_factor(vk::BlendFactor::SRC_ALPHA)
                    .dst_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
                    .alpha_blend_op(vk::BlendOp::ADD)
                    .color_write_mask(
                        vk::ColorComponentFlags::R
                            | vk::ColorComponentFlags::G
                            | vk::ColorComponentFlags::B
                            | vk::ColorComponentFlags::A,
                    )
                    .build(),];
            let colourblend_info =
                vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);
    
            let descriptorset_layout_binding_descs = [vk::DescriptorSetLayoutBinding::builder()
                .binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .descriptor_count(1)
                .stage_flags(vk::ShaderStageFlags::VERTEX)
                .build()];
            let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
                .bindings(&descriptorset_layout_binding_descs);
            let descriptorsetlayout = unsafe {
                logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
            }?;
    
            let desc_set_color = GBufferFillPipeline::get_img_desc_set(logical_device.clone());
            let desc_set_normal = GBufferFillPipeline::get_img_desc_set(logical_device.clone());
            let desc_set_met_roug = GBufferFillPipeline::get_img_desc_set(logical_device.clone());
    
            let desclayouts = vec![descriptorsetlayout, desc_set_color, desc_set_normal, desc_set_met_roug];
            let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&desclayouts);
    
            let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
                .depth_test_enable(true)
                .depth_write_enable(true)
                .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
    
            let pipelinelayout =
                unsafe { logical_device.create_pipeline_layout(&pipelinelayout_info, None) }?;
            let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly_info)
                .viewport_state(&viewport_info)
                .rasterization_state(&rasterizer_info)
                .multisample_state(&multisampler_info)
                .depth_stencil_state(&depth_stencil_info)
                .color_blend_state(&colourblend_info)
                .layout(pipelinelayout)
                .render_pass(renderpass.inner)
                .subpass(0);
            let graphicspipeline = unsafe {
                logical_device
                    .create_graphics_pipelines(
                        vk::PipelineCache::null(),
                        &[pipeline_info.build()],
                        None,
                    )
                    .expect("A problem with the pipeline creation")
            }[0];
            unsafe {
                logical_device.destroy_shader_module(fragmentshader_module, None);
                logical_device.destroy_shader_module(vertexshader_module, None);
            }
            Ok((
                graphicspipeline,
                pipelinelayout,
                desclayouts
            ))
        }
    

    pub fn init_renderpass(
        base : &GraphicBase
        ) -> Result<RenderPassSafe, vk::Result> {
            let attachments = [vk::AttachmentDescription::builder()
                .load_op(vk::AttachmentLoadOp::CLEAR)
                .store_op(vk::AttachmentStoreOp::STORE)
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(vk::ImageLayout::UNDEFINED)
                .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                .samples(vk::SampleCountFlags::TYPE_1)
                .format(vk::Format::R32G32B32A32_SFLOAT)
                .build(),
                vk::AttachmentDescription::builder()
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .build(),
                vk::AttachmentDescription::builder()
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .format(vk::Format::R32G32B32A32_SFLOAT)
                    .build(),
                vk::AttachmentDescription::builder()
                    .format(vk::Format::D32_SFLOAT)
                    .load_op(vk::AttachmentLoadOp::CLEAR)
                    .store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .build(),];
            let color_attachment_references = [vk::AttachmentReference {
                attachment: 0,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
                vk::AttachmentReference {
                    attachment: 1,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                },
                vk::AttachmentReference {
                    attachment: 2,
                    layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                }];
            let depth_attachment_reference = vk::AttachmentReference {
                attachment: 3,
                layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            };
            let subpasses = [vk::SubpassDescription::builder()
                .color_attachments(&color_attachment_references)
                .depth_stencil_attachment(&depth_attachment_reference)
                .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
                .build()];
            let subpass_dependencies = [vk::SubpassDependency::builder()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_subpass(0)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(
                    vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                )
                .build()];
            let renderpass_info = vk::RenderPassCreateInfo::builder()
                .attachments(&attachments)
                .subpasses(&subpasses)
                .dependencies(&subpass_dependencies);
            let renderpass = unsafe { base.device.create_render_pass(&renderpass_info, None)? };
        
            Ok(base.wrap_render_pass(renderpass))
        }

    pub fn new(
        graphic_base : &GraphicBase,
        camera : &RenderCamera) -> Result<Self, vk::Result> {
        let renderpass = GBufferFillPipeline::init_renderpass(&graphic_base).unwrap();

        let (pipeline, pipeline_layout, descriptor_set_layouts) =
             GBufferFillPipeline::init_base_pipeline(
                &graphic_base.device,
                &graphic_base.swapchain,
                &renderpass).unwrap();

        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty : vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count : graphic_base.swapchain.amount_of_images
            },
            vk::DescriptorPoolSize {
                ty : vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count : 1024
            },
        ];
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(1024 + 3)
            .pool_sizes(&pool_sizes);
        let descriptor_pool = unsafe {
            graphic_base.device.create_descriptor_pool(&descriptor_pool_info, None)
        }.unwrap();

        let desc_layouts =
            vec![descriptor_set_layouts[0]; 1];
        let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(&desc_layouts);
        let descriptor_sets =
            unsafe { graphic_base.device.allocate_descriptor_sets(&descriptor_set_allocate_info)
            }?;

        for (_, descset) in descriptor_sets.iter().enumerate() {
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

        let renderpass = Arc::new(renderpass);
        let framebuffer_storage = FramebufferStorage::new(&renderpass);

        Ok(Self {
            pipeline,
            descriptor_sets,
            renderpass,
            device : graphic_base.device.clone(),
            descriptor_pool : Arc::new(DescriptorPoolSafe { pool: descriptor_pool, device: graphic_base.device.clone() }),
            descriptor_sets_texture : HashMap::new(),
            mode : MaterialTexture::Diffuse,
            framebuffers: framebuffer_storage,
            layout: pipeline_layout,
            descriptor_set_layouts,
        })
    }

    fn update_tex_desc(&mut self, tex: &ServerTexture, texture_server : &TextureServer) {
        unsafe {
            let tex = tex.get_texture(texture_server);
            if self.descriptor_sets_texture.contains_key(&tex.index) == false {
                let imageinfo = vk::DescriptorImageInfo::builder()
                    .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                    .image_view(tex.imageview)
                    .sampler(tex.sampler)
                    .build();

                info!("image layout {:?}", imageinfo.image_layout);

                let desc_layouts_texture =
                    vec![self.descriptor_set_layouts[1]; 1];
                let descriptor_set_allocate_info_texture = vk::DescriptorSetAllocateInfo::builder()
                    .descriptor_pool(self.descriptor_pool.pool)
                    .set_layouts(&desc_layouts_texture);
                self.descriptor_sets_texture.insert(tex.index, self.device.allocate_descriptor_sets(
                    &descriptor_set_allocate_info_texture).unwrap()[0]);

                let mut descriptorwrite_image = vk::WriteDescriptorSet::builder()
                    .dst_set(self.descriptor_sets_texture[&tex.index])
                    .dst_binding(0)
                    .dst_array_element(0)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .build();

                descriptorwrite_image.descriptor_count = 1;
                descriptorwrite_image.p_image_info = &imageinfo;
                self.device.update_descriptor_sets(&[descriptorwrite_image], &[]);
            }
        }
    }
}

impl InstancesDrawer for GBufferFillPipeline {
    fn process(&mut self, cmd: CommandBuffer, dst: &Vec<Arc<TextureSafe>>, server: &RenderServer, assets : &AssetServer) {
        let clearvalues = [vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0],
            },
        },
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0]
            }
        },
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 0.0]
            }
        },
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            }
        },];

        let fb = self.framebuffers.get_framebuffer(dst);

        let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
            .render_pass(self.renderpass.inner)
            .framebuffer(fb.franebuffer)
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: dst[0].get_extent2d()
            })
            .clear_values(&clearvalues);

        unsafe {
            self.device.cmd_begin_render_pass(
                cmd,
                &renderpass_begininfo,
                vk::SubpassContents::INLINE,
            );
            self.device.cmd_bind_pipeline(
                cmd,
                vk::PipelineBindPoint::GRAPHICS,
                self.pipeline,
            );

            for model in &server.render_models {
                self.update_tex_desc(&model.material.color, &assets.texture_server);
                self.update_tex_desc(&model.material.normal, &assets.texture_server);
                self.update_tex_desc(&model.material.metallic_roughness, &assets.texture_server);
            }

            for model in &server.render_models {
                let color = model.material.color.get_texture(&assets.texture_server);
                let normal = model.material.normal.get_texture(&assets.texture_server);
                let metallic_roughness = model.material.metallic_roughness.get_texture(&assets.texture_server);

                self.device.cmd_bind_descriptor_sets(
                    cmd,
                    vk::PipelineBindPoint::GRAPHICS,
                    self.layout,
                    0,
                    &[self.descriptor_sets[0],
                        self.descriptor_sets_texture[&color.index],
                        self.descriptor_sets_texture[&normal.index],
                        self.descriptor_sets_texture[&metallic_roughness.index],],
                    &[]
                );

                self.device.cmd_bind_vertex_buffers(
                    cmd,
                    0,
                    &[model.mesh.pos_data.buffer,
                        model.mesh.normal_data.buffer,
                        model.mesh.uv_data.buffer,
                        model.instances.buffer],
                    &[0, 0, 0, 0]);
                self.device.cmd_bind_index_buffer(cmd, model.mesh.index_data.buffer, 0, vk::IndexType::UINT32);
                self.device.cmd_draw_indexed(cmd, model.mesh.vertex_count, model.model_count as u32, 0, 0, 0);
            }

            self.device.cmd_end_render_pass(cmd);
        }
    }

    fn get_output_count(&self) -> usize {
        4
    }
}