// use std::collections::HashMap;
// use std::sync::Arc;
// use ash::prelude::VkResult;
// use ash::vk;
// use ash::vk::{CommandBuffer, DescriptorSet, Extent2D, Framebuffer};
// use log::*;
// use crate::{AllocatorSafe, DeviceSafe, GraphicBase, init_renderpass, RenderCamera, RenderModel, RenderPassSafe, SwapchainSafe, DescriptorPoolSafe, TextureServer, MaterialTexture, FramebufferStorage, InstancesDrawer, TextureSafe, RenderServer, ServerTexture, FramebufferSafe, BufferSafe, ShadowPrepare, GPUMesh};
// use ash::vk::DescriptorSetLayout;
// use crate::asset_server::AssetServer;
// use crate::light::PointLight;


// pub struct PointLightShadowPipeline {
//     light_info_buffer : BufferSafe,
//     framebuffers : FramebufferStorage,
//     pub renderpass : Arc<RenderPassSafe>,
//     device : Arc<DeviceSafe>,
//     allocator : Arc<AllocatorSafe>,
//     descriptor_pool : Arc<DescriptorPoolSafe>,
//     descriptor_sets_texture : HashMap<usize, DescriptorSet>,
    
//     pub pipeline: vk::Pipeline,
//     pub layout: vk::PipelineLayout,
//     pub descriptor_set_layouts : Vec<DescriptorSetLayout>,
//     pub size : vk::Extent2D,
// }


// impl Drop for PointLightShadowPipeline {
//     fn drop(&mut self) {
//         unsafe {
//             info!("Destroying grayscale pipeline...");
//             self.device.device_wait_idle();

//             unsafe {
//                 for dsl in &self.descriptor_set_layouts {
//                     self.device.destroy_descriptor_set_layout(*dsl, None);
//                 }
//                 self.device.destroy_pipeline(self.pipeline, None);
//                 self.device.destroy_pipeline_layout(self.layout, None);
//             }
//         }
//     }
// }


// impl PointLightShadowPipeline {

//     fn get_img_desc_set(logical_device : Arc<DeviceSafe>) -> vk::DescriptorSetLayout {
//         let descriptorset_layout_binding_descs = [vk::DescriptorSetLayoutBinding::builder()
//             .binding(0)
//             .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
//             .descriptor_count(1)
//             .stage_flags(vk::ShaderStageFlags::FRAGMENT)
//             .build()];
//         let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
//             .bindings(&descriptorset_layout_binding_descs);
//         let descriptorsetlayout = unsafe {
//             logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
//         }.unwrap();
//         descriptorsetlayout
//     }

//     fn init_base_pipeline(
//         logical_device: &Arc<DeviceSafe>,
//         swapchain: &SwapchainSafe,
//         renderpass: &RenderPassSafe) -> Result<(vk::Pipeline, vk::PipelineLayout, Vec<vk::DescriptorSetLayout>), Box<dyn std::error::Error>> {
//             let vertexshader_createinfo = vk::ShaderModuleCreateInfo::builder().code(
//                 vk_shader_macros::include_glsl!("./shaders/point_light_shadow/shader.vert", kind: vert),
//             );
//             let vertexshader_module =
//                 unsafe { logical_device.create_shader_module(&vertexshader_createinfo, None)? };
//             let fragmentshader_createinfo = vk::ShaderModuleCreateInfo::builder()
//                 .code(vk_shader_macros::include_glsl!("./shaders/point_light_shadow/shader.frag"));
//             let fragmentshader_module =
//                 unsafe { logical_device.create_shader_module(&fragmentshader_createinfo, None)? };
//             let mainfunctionname = std::ffi::CString::new("main").unwrap();
//             let vertexshader_stage = vk::PipelineShaderStageCreateInfo::builder()
//                 .stage(vk::ShaderStageFlags::VERTEX)
//                 .module(vertexshader_module)
//                 .name(&mainfunctionname);
//             let fragmentshader_stage = vk::PipelineShaderStageCreateInfo::builder()
//                 .stage(vk::ShaderStageFlags::FRAGMENT)
//                 .module(fragmentshader_module)
//                 .name(&mainfunctionname);
//             let shader_stages = vec![vertexshader_stage.build(), fragmentshader_stage.build()];
    
//             let mut vertex_attrib_descs  = GPUMesh::get_vertex_attrib_desc();
//             let vertex_binding_descs = GPUMesh::get_binding_desc();
    
//             let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
//                 .vertex_attribute_descriptions(&vertex_attrib_descs)
//                 .vertex_binding_descriptions(&vertex_binding_descs);
    
//             let input_assembly_info = vk::PipelineInputAssemblyStateCreateInfo::builder()
//                 .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
//             let viewports = [vk::Viewport {
//                 x: 0.,
//                 y: 0.,
//                 width: 1024 as f32,
//                 height: 1024 as f32,
//                 min_depth: 0.,
//                 max_depth: 1.,
//             }];
//             let scissors = [vk::Rect2D {
//                 offset: vk::Offset2D { x: 0, y: 0 },
//                 extent: Extent2D {width : 1024, height : 1024},
//             }];
    
//             let viewport_info = vk::PipelineViewportStateCreateInfo::builder()
//                 .viewports(&viewports)
//                 .scissors(&scissors);
//             let rasterizer_info = vk::PipelineRasterizationStateCreateInfo::builder()
//                 .line_width(1.0)
//                 .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
//                 .cull_mode(vk::CullModeFlags::FRONT)
//                 .polygon_mode(vk::PolygonMode::FILL);
//             let multisampler_info = vk::PipelineMultisampleStateCreateInfo::builder()
//                 .rasterization_samples(vk::SampleCountFlags::TYPE_1);
//             let colourblend_attachments = [vk::PipelineColorBlendAttachmentState::builder()
//                 .blend_enable(true)
//                 .src_color_blend_factor(vk::BlendFactor::ONE)
//                 .dst_color_blend_factor(vk::BlendFactor::ONE)
//                 .color_blend_op(vk::BlendOp::ADD)
//                 .src_alpha_blend_factor(vk::BlendFactor::ONE)
//                 .dst_alpha_blend_factor(vk::BlendFactor::ONE)
//                 .alpha_blend_op(vk::BlendOp::ADD)
//                 .color_write_mask(
//                     vk::ColorComponentFlags::R
//                         | vk::ColorComponentFlags::G
//                         | vk::ColorComponentFlags::B
//                         | vk::ColorComponentFlags::A,
//                 )
//                 .build(); 1];
//             let colourblend_info =
//                 vk::PipelineColorBlendStateCreateInfo::builder().attachments(&colourblend_attachments);
    
//             let descriptorset_layout_binding_descs = [vk::DescriptorSetLayoutBinding::builder()
//                 .binding(0)
//                 .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
//                 .descriptor_count(1)
//                 .stage_flags(vk::ShaderStageFlags::VERTEX)
//                 .build()];
//             let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
//                 .bindings(&descriptorset_layout_binding_descs);
//             let descriptorsetlayout = unsafe {
//                 logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
//             }?;

//             let light_layout = {
//                 let descriptorset_layout_binding_descs = [vk::DescriptorSetLayoutBinding::builder()
//                     .binding(0)
//                     .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
//                     .descriptor_count(1)
//                     .stage_flags(vk::ShaderStageFlags::FRAGMENT)
//                     .build()];
//                 let descriptorset_layout_info = vk::DescriptorSetLayoutCreateInfo::builder()
//                     .bindings(&descriptorset_layout_binding_descs);
//                 unsafe {
//                     logical_device.create_descriptor_set_layout(&descriptorset_layout_info, None)
//                 }.unwrap()
//             };

//             let desclayouts = vec![descriptorsetlayout, light_layout];
//             let pipelinelayout_info = vk::PipelineLayoutCreateInfo::builder().set_layouts(&desclayouts);

//         let depth_stencil_info = vk::PipelineDepthStencilStateCreateInfo::builder()
//                 .depth_test_enable(true)
//                 .depth_write_enable(true)
//                 .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL);
    
//             let pipelinelayout =
//                 unsafe { logical_device.create_pipeline_layout(&pipelinelayout_info, None) }?;
//             let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
//                 .stages(&shader_stages)
//                 .vertex_input_state(&vertex_input_info)
//                 .input_assembly_state(&input_assembly_info)
//                 .viewport_state(&viewport_info)
//                 .rasterization_state(&rasterizer_info)
//                 .multisample_state(&multisampler_info)
//                 .depth_stencil_state(&depth_stencil_info)
//                 .color_blend_state(&colourblend_info)
//                 .layout(pipelinelayout)
//                 .render_pass(renderpass.inner)
//                 .subpass(0);
//             let graphicspipeline = unsafe {
//                 logical_device
//                     .create_graphics_pipelines(
//                         vk::PipelineCache::null(),
//                         &[pipeline_info.build()],
//                         None,
//                     )
//                     .expect("A problem with the pipeline creation")
//             }[0];
//             unsafe {
//                 logical_device.destroy_shader_module(fragmentshader_module, None);
//                 logical_device.destroy_shader_module(vertexshader_module, None);
//             }
//             Ok((
//                 graphicspipeline,
//                 pipelinelayout,
//                 desclayouts
//             ))
//         }
    

//     pub fn init_renderpass(
//         base : &GraphicBase
//         ) -> Result<RenderPassSafe, vk::Result> {
//             let attachments = [vk::AttachmentDescription::builder()
//             .format(vk::Format::D32_SFLOAT)
//             .load_op(vk::AttachmentLoadOp::CLEAR)
//             .store_op(vk::AttachmentStoreOp::STORE)
//             .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
//             .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
//             .initial_layout(vk::ImageLayout::UNDEFINED)
//             .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
//             .samples(vk::SampleCountFlags::TYPE_1)
//             .build(),];
//             let color_attachment_references = [];
//             let depth_attachment_reference = vk::AttachmentReference {
//                 attachment: 0,
//                 layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
//             };
//             let subpasses = [vk::SubpassDescription::builder()
//                 .color_attachments(&color_attachment_references)
//                 .depth_stencil_attachment(&depth_attachment_reference)
//                 .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
//                 .build()];
//             let subpass_dependencies = [vk::SubpassDependency::builder()
//                 .src_subpass(vk::SUBPASS_EXTERNAL)
//                 .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
//                 .dst_subpass(0)
//                 .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
//                 .dst_access_mask(
//                     vk::AccessFlags::COLOR_ATTACHMENT_READ | vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
//                 )
//                 .build()];
//             let renderpass_info = vk::RenderPassCreateInfo::builder()
//                 .attachments(&attachments)
//                 .subpasses(&subpasses)
//                 .dependencies(&subpass_dependencies);
//             let renderpass = unsafe { base.device.create_render_pass(&renderpass_info, None)? };
        
//             Ok(base.wrap_render_pass(renderpass))
//         }

//     fn generate_set(
//             gb : &GraphicBase,
//             layout : DescriptorSetLayout,
//             pool : vk::DescriptorPool,
//             buffer : vk::Buffer,
//             range : u64) -> DescriptorSet {
//         let desc_layouts =
//             vec![layout; 1];
//         let descriptor_set_allocate_info = vk::DescriptorSetAllocateInfo::builder()
//             .descriptor_pool(pool)
//             .set_layouts(&desc_layouts);
//         let descriptor_sets =
//             unsafe { gb.device.allocate_descriptor_sets(&descriptor_set_allocate_info)
//             }.unwrap();

//         for (_, descset) in descriptor_sets.iter().enumerate() {
//             let buffer_infos = [vk::DescriptorBufferInfo {
//                 buffer,
//                 offset: 0,
//                 range : range as vk::DeviceSize,
//             }];
//             let desc_sets_write = [vk::WriteDescriptorSet::builder()
//                 .dst_set(*descset)
//                 .dst_binding(0)
//                 .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
//                 .buffer_info(&buffer_infos)
//                 .build()];
//             unsafe { gb.device.update_descriptor_sets(&desc_sets_write, &[]) };
//         }

//         descriptor_sets[0]
//     }

//     pub fn new(
//         graphic_base : &GraphicBase) -> Result<Self, vk::Result> {

//         let renderpass = PointLightShadowPipeline::init_renderpass(&graphic_base).unwrap();

//         let (pipeline, pipeline_layout, descriptor_set_layouts) =
//             PointLightShadowPipeline::init_base_pipeline(
//                 &graphic_base.device,
//                 &graphic_base.swapchain,
//                 &renderpass).unwrap();

//         let pool_sizes = [
//             vk::DescriptorPoolSize {
//                 ty : vk::DescriptorType::UNIFORM_BUFFER,
//                 descriptor_count : 1024
//             },
//         ];
//         let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
//             .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
//             .max_sets(1024)
//             .pool_sizes(&pool_sizes);
//         let descriptor_pool = unsafe {
//             graphic_base.device.create_descriptor_pool(&descriptor_pool_info, None)
//         }.unwrap();

//         let renderpass = Arc::new(renderpass);
//         let framebuffer_storage = FramebufferStorage::new(&renderpass);

//         let mut light_info_buffer = BufferSafe::new(
//             &graphic_base.allocator,
//             4 * 2,
//             vk::BufferUsageFlags::UNIFORM_BUFFER,
//             gpu_allocator::MemoryLocation::CpuToGpu).unwrap();

//         light_info_buffer.fill(
//             &[graphic_base.swapchain.extent.width as f32,
//                 graphic_base.swapchain.extent.height as f32]);

//         Ok(Self {
//             pipeline,
//             renderpass,
//             light_info_buffer,
//             device : graphic_base.device.clone(),
//             descriptor_pool : Arc::new(DescriptorPoolSafe { pool: descriptor_pool, device: graphic_base.device.clone() }),
//             descriptor_sets_texture : HashMap::new(),
//             framebuffers: framebuffer_storage,
//             layout: pipeline_layout,
//             descriptor_set_layouts,
//             allocator : graphic_base.allocator.clone(),
//             size : Extent2D {width : 1024, height : 1024}
//         })
//     }

//     fn update_tex_desc(&mut self, tex: &Arc<TextureSafe>) {
//         unsafe {
//             if self.descriptor_sets_texture.contains_key(&tex.index) == false {
//                 let imageinfo = vk::DescriptorImageInfo::builder()
//                     .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
//                     .image_view(tex.imageview)
//                     .sampler(tex.sampler)
//                     .build();

//                 info!("image layout {:?}", imageinfo.image_layout);

//                 let desc_layouts_texture =
//                     vec![self.descriptor_set_layouts[2]; 1];
//                 let descriptor_set_allocate_info_texture = vk::DescriptorSetAllocateInfo::builder()
//                     .descriptor_pool(self.descriptor_pool.pool)
//                     .set_layouts(&desc_layouts_texture);
//                 self.descriptor_sets_texture.insert(tex.index, self.device.allocate_descriptor_sets(
//                     &descriptor_set_allocate_info_texture).unwrap()[0]);

//                 let mut descriptorwrite_image = vk::WriteDescriptorSet::builder()
//                     .dst_set(self.descriptor_sets_texture[&tex.index])
//                     .dst_binding(0)
//                     .dst_array_element(0)
//                     .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
//                     .build();

//                 descriptorwrite_image.descriptor_count = 1;
//                 descriptorwrite_image.p_image_info = &imageinfo;
//                 self.device.update_descriptor_sets(&[descriptorwrite_image], &[]);
//             }
//         }
//     }
// }

// impl ShadowPrepare for PointLightShadowPipeline {
//     fn process(
//         &mut self,
//         cmd: CommandBuffer,
//         server: &mut RenderServer,
//         assets: &AssetServer) {
//         let clearvalues = [vk::ClearValue {
//             depth_stencil: vk::ClearDepthStencilValue {
//                 depth: 1.0,
//                 stencil: 0
//             },
//         }, ];

//         for light in &mut server.point_lights {
//             if light.shadow_enabled == false {
//                 continue;
//             }

//             if light.shadow_map.is_none() {
//                 let mut shadow = PointLight::get_shadow_map(
//                     &self.allocator,
//                     &self.device,
//                     &self.renderpass
//                 );

//                 for i in 0..6 {
//                     unsafe {
//                         let buffer_infos = [vk::DescriptorBufferInfo {
//                             buffer: shadow.cameras[i].uniformbuffer.buffer,
//                             offset: 0,
//                             range: 128,
//                         }];

//                         let desc_layouts =
//                             vec![self.descriptor_set_layouts[0]; 1];
//                         let desc_set_info = vk::DescriptorSetAllocateInfo::builder()
//                             .descriptor_pool(self.descriptor_pool.pool)
//                             .set_layouts(&desc_layouts);
//                         let desc_set =
//                             self.device.allocate_descriptor_sets(
//                                 &desc_set_info).unwrap()[0];

//                         let desc_sets_write = [
//                             vk::WriteDescriptorSet::builder()
//                                 .dst_set(desc_set)
//                                 .dst_binding(0)
//                                 .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
//                                 .buffer_info(&buffer_infos)
//                                 .build()];

//                         self.device.update_descriptor_sets(
//                             &desc_sets_write,
//                             &[]);

//                         shadow.sets.push(
//                             desc_set);
//                     }
//                 }

//                 unsafe {
//                     let buffer_infos = [vk::DescriptorBufferInfo {
//                         buffer: shadow.shadow_uniform.buffer,
//                         offset: 0,
//                         range: 3 * 4,
//                     }];

//                     let desc_layouts =
//                         vec![self.descriptor_set_layouts[1]; 1];
//                     let desc_set_info = vk::DescriptorSetAllocateInfo::builder()
//                         .descriptor_pool(self.descriptor_pool.pool)
//                         .set_layouts(&desc_layouts);
//                     let desc_set =
//                         self.device.allocate_descriptor_sets(
//                             &desc_set_info).unwrap()[0];

//                     shadow.shadow_set = desc_set;

//                     let desc_sets_write = [
//                         vk::WriteDescriptorSet::builder()
//                             .dst_set(desc_set)
//                             .dst_binding(0)
//                             .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
//                             .buffer_info(&buffer_infos)
//                             .build()];

//                     self.device.update_descriptor_sets(
//                         &desc_sets_write,
//                         &[]);
//                 }

//                 light.shadow_map = Some(shadow);
//             }

//             let mut shadow_map = light.shadow_map.as_mut().unwrap();


//             for i in 0..6 {
//                 shadow_map.texture.barrier_range(
//                     cmd,
//                     vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
//                     vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
//                     vk::PipelineStageFlags::ALL_GRAPHICS,
//                     vk::ImageSubresourceRange::builder()
//                         .layer_count(1)
//                         .base_array_layer(i)
//                         .base_mip_level(0)
//                         .level_count(1)
//                         .aspect_mask(vk::ImageAspectFlags::DEPTH)
//                         .build()
//                 );
//             }

//             for cam_idx in 0..6 {

//                 shadow_map.cameras[cam_idx].position = light.position.into();
//                 shadow_map.cameras[cam_idx].update_projectionmatrix();
//                 shadow_map.cameras[cam_idx].update_viewmatrix();
//                 shadow_map.cameras[cam_idx].update_inner_buffer();

//                 let renderpass_begininfo = vk::RenderPassBeginInfo::builder()
//                     .render_pass(self.renderpass.inner)
//                     .framebuffer(shadow_map.framebuffer[cam_idx].framebuffer)
//                     .render_area(vk::Rect2D {
//                         offset: vk::Offset2D { x: 0, y: 0 },
//                         extent: self.size
//                     })
//                     .clear_values(&clearvalues);

//                 unsafe {

//                     self.device.cmd_bind_pipeline(
//                         cmd,
//                         vk::PipelineBindPoint::GRAPHICS,
//                         self.pipeline,
//                     );
//                     self.device.cmd_begin_render_pass(
//                         cmd,
//                         &renderpass_begininfo,
//                         vk::SubpassContents::INLINE,
//                     );

//                     shadow_map.shadow_uniform.fill(&light.position).unwrap();

//                     for model in &server.render_models {

//                         self.device.cmd_bind_descriptor_sets(
//                             cmd,
//                             vk::PipelineBindPoint::GRAPHICS,
//                             self.layout,
//                             0,
//                             &[shadow_map.sets[cam_idx], shadow_map.shadow_set],
//                             &[]);

//                         self.device.cmd_bind_vertex_buffers(
//                             cmd,
//                             0,
//                             &[model.mesh.pos_data.buffer,
//                                 model.mesh.normal_data.buffer,
//                                 model.mesh.tangent_data.buffer,
//                                 model.mesh.uv_data.buffer,
//                                 model.instances.buffer],
//                             &[0, 0, 0, 0, 0]);
//                         self.device.cmd_bind_index_buffer(cmd, model.mesh.index_data.buffer, 0, vk::IndexType::UINT32);
//                         self.device.cmd_draw_indexed(cmd, model.mesh.vertex_count, model.model_count as u32, 0, 0, 0);
//                     }

//                     self.device.cmd_end_render_pass(cmd);
//                 }
//             }
//         }
//     }


//     fn create_framebuffer(&mut self) -> Arc<FramebufferSafe> {
//         let mut gbuffer_buf = vec![];
//         let tex = Arc::new(TextureSafe::new(
//             &self.allocator,
//             &self.device,
//             vk::Extent2D {
//                 width : 1024,
//                 height : 1024
//             },
//             vk::Format::R32G32B32A32_SFLOAT,
//             false));
//         gbuffer_buf.push(tex);
//         self.framebuffers.get_framebuffer(&gbuffer_buf)
//     }
// }