use std::sync::Arc;
use ash::vk;
use crate::{ApiBase, TextureSafe, TextureTransform};

pub struct TextureDemonstratePipeline {
    api : ApiBase,
    pub show_idx : usize
}

impl TextureDemonstratePipeline {
    pub fn new(api : &ApiBase) -> Self {

        Self {
            api : api.clone(),
            show_idx : 0
        }
    }
}

impl TextureTransform for TextureDemonstratePipeline {
    fn process(&mut self, cmd : vk::CommandBuffer, dst: &Vec<Arc<TextureSafe>>, input: Vec<Arc<TextureSafe>>) {
        let src = &input[self.show_idx];
        let dst = &dst[0];

        let src_state = src.get_barrier_state(0);
        let dst_state = dst.get_barrier_state(0);

        let range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: 0,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        };

        src.barrier_range(
            cmd,
            vk::AccessFlags::TRANSFER_READ,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::PipelineStageFlags::TRANSFER,
            range.clone());

        dst.barrier_range(
            cmd,
            vk::AccessFlags::TRANSFER_READ,
            vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
            vk::PipelineStageFlags::TRANSFER,
            range.clone());

        let blits = [vk::ImageBlit {
            src_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            src_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: src.get_width() as i32,
                    y: src.get_height() as i32,
                    z: 1,
                },
            ],
            dst_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: 0,
                base_array_layer: 0,
                layer_count: 1,
            },
            dst_offsets: [
                vk::Offset3D { x: 0, y: 0, z: 0 },
                vk::Offset3D {
                    x: dst.get_width() as i32,
                    y: dst.get_height() as i32,
                    z: 1,
                },
            ],
        }];

        unsafe {
            self.api.device.cmd_blit_image(
                cmd,
                src.image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                dst.image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &blits,
                vk::Filter::LINEAR,
            );
        }

        src.barrier_range(
            cmd,
            src_state.access,
            src_state.layout,
            src_state.stage,
            range.clone()
        );

        dst.barrier_range(
            cmd,
            dst_state.access,
            dst_state.layout,
            dst_state.stage,
            range.clone()
        );
    }

    fn get_output_count(&self) -> usize {
        1
    }
}