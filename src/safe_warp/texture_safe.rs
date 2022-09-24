use std::{sync::{Arc, Mutex}, mem::ManuallyDrop, slice::SliceIndex, collections::HashMap};

use ash::vk::{self, DescriptorSet};

use crate::{DeviceSafe, AllocatorSafe, BufferSafe, Pools, GraphicBase};

use log::*;

static GLOBAL_TEXTURE_INDEXER : Mutex<usize> = Mutex::new(0);

pub struct TextureSafe {
    pub image : vk::Image,
    pub allocation : vk_mem::Allocation,
    pub allocation_info : vk_mem::AllocationInfo,
    pub imageview : vk::ImageView,
    pub sampler : vk::Sampler,
    pub allocator : Arc<AllocatorSafe>,
    pub device : Arc<DeviceSafe>,
    pub index : usize
}

impl Drop for TextureSafe {
    fn drop(&mut self) {

        unsafe {
            debug!("Destroy TextureSafe");
            self.device.destroy_sampler(self.sampler, None);
            self.device.destroy_image_view(self.imageview, None);
            self.allocator.destroy_image(self.image, &self.allocation).unwrap();
        }
    }
}

impl TextureSafe {

    pub fn from_file<P: AsRef<std::path::Path>>(
        path: P,
        gb: &GraphicBase,
        pools : &Pools) -> Result<Self, Box<dyn std::error::Error>> {
        let image = image::open(path)
            .map(|img| img.to_rgba())
            .expect("unable to open image");
        let (width, height) = image.dimensions();
        let mut res = TextureSafe::new(
            &gb.allocator,
            &gb.device,
            vk::Extent2D {
                width,
                height
            },
            vk::Format::R8G8B8A8_SRGB
        );

        let data = image.clone().into_raw();
        info!("data len {}", data.len());
        let mut buffer = ManuallyDrop::new( BufferSafe::new(
            &gb.allocator,
            data.len() as u64,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk_mem::MemoryUsage::CpuToGpu,
        ).unwrap());
        buffer.fill(&data).unwrap();

        let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pools.graphics.pool)
            .command_buffer_count(1);
        let copycmdbuffer = unsafe {
            gb
                .device
                .allocate_command_buffers(&commandbuf_allocate_info)
        }
            .unwrap()[0];

        let cmdbegininfo = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
        unsafe {
            gb
                .device
                .begin_command_buffer(copycmdbuffer, &cmdbegininfo)
        }?;


        let barrier = vk::ImageMemoryBarrier::builder()
            .image(res.image)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .old_layout(vk::ImageLayout::UNDEFINED)
            .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
        unsafe {
            gb.device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };


        //Insert commands here.
        let image_subresource = vk::ImageSubresourceLayers {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            mip_level: 0,
            base_array_layer: 0,
            layer_count: 1,
        };
        let region = vk::BufferImageCopy {
            buffer_offset: 0,
            buffer_row_length: 0,
            buffer_image_height: 0,
            image_offset: vk::Offset3D { x: 0, y: 0, z: 0 },
            image_extent: vk::Extent3D {
                width,
                height,
                depth: 1,
            },
            image_subresource,
            ..Default::default()
        };
        unsafe {
            gb.device.cmd_copy_buffer_to_image(
                copycmdbuffer,
                buffer.buffer,
                res.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &[region],
            );
        }

        let barrier = vk::ImageMemoryBarrier::builder()
            .image(res.image)
            .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
            .dst_access_mask(vk::AccessFlags::SHADER_READ)
            .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            })
            .build();
        unsafe {
            gb.device.cmd_pipeline_barrier(
                copycmdbuffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[barrier],
            )
        };


        unsafe { gb.device.end_command_buffer(copycmdbuffer) }?;
        let submit_infos = [vk::SubmitInfo::builder()
            .command_buffers(&[copycmdbuffer])
            .build()];
        let fence = unsafe {
            gb
                .device
                .create_fence(&vk::FenceCreateInfo::default(), None)
        }?;
        unsafe {
            gb
                .device
                .queue_submit(gb.queues.graphics_queue, &submit_infos, fence)
        }?;
        unsafe { gb.device.wait_for_fences(&[fence], true, std::u64::MAX) }?;


        unsafe { gb.device.destroy_fence(fence, None) };
        // gb.allocator.destroy_buffer(buffer.buffer, &buffer.allocation)?;
        unsafe {
            gb
                .device
                .free_command_buffers(pools.graphics.pool, &[copycmdbuffer])
        };

        unsafe {
            gb.device.device_wait_idle().unwrap();
            }
    

        info!("Finish copy");

        unsafe {
            ManuallyDrop::drop(&mut buffer);
        }

        Ok(res)
    }

    fn new(
        allocator : &Arc<AllocatorSafe>,
        device : &Arc<DeviceSafe>,
        extent : vk::Extent2D,
        format : vk::Format) -> Self {
        let img_create_info = vk::ImageCreateInfo::builder()
            .image_type(vk::ImageType::TYPE_2D)
            .extent(vk::Extent3D {
                width : extent.width,
                height : extent.height,
                depth : 1
            })
            .mip_levels(1)
            .array_layers(1)
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .usage(vk::ImageUsageFlags::TRANSFER_DST | vk::ImageUsageFlags::SAMPLED);
        let alloc_create_info = vk_mem::AllocationCreateInfo {
            usage : vk_mem::MemoryUsage::GpuOnly,
            ..Default::default()
        };
        let (vk_image, allocation, allocation_info) = allocator
            .create_image(&img_create_info, &alloc_create_info)
            .expect("creating vkImage for texture");
        let view_create_info = vk::ImageViewCreateInfo::builder()
            .image(vk_image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            });
        let imageview = unsafe {
            device.create_image_view(&view_create_info, None).expect("image view creaton")
        };
        let sampler_info = vk::SamplerCreateInfo::builder()
            .mag_filter(vk::Filter::LINEAR)
            .min_filter(vk::Filter::LINEAR);
        let sampler =
            unsafe { device.create_sampler(&sampler_info, None) }.expect("sampler creation");

        let mut global_index = GLOBAL_TEXTURE_INDEXER.lock().unwrap();
        let index = *global_index;
        *global_index += 1;

        Self {
            image : vk_image,
            allocation,
            allocation_info,
            imageview,
            sampler,
            allocator : allocator.clone(),
            device : device.clone(),
            index
        }
    }
}
