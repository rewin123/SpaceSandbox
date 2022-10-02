use std::{sync::Arc, ops::Deref};

use ash::vk;
use crate::{DeviceSafe, Pool};
use log::*;

pub struct CommandBufferSafe {
    pub cmd : vk::CommandBuffer,
    device : Arc<DeviceSafe>,
    pool : Arc<Pool>,
    depends : Vec<Box<dyn std::any::Any>>
}

impl Drop for CommandBufferSafe {
    fn drop(&mut self) {
        unsafe {
           debug!("Destroy command buffer");
           self.device.free_command_buffers(self.pool.pool, &[self.cmd]); 
        }
    }
}

impl Deref for CommandBufferSafe {
    type Target = vk::CommandBuffer;

    fn deref(&self) -> &Self::Target {
        &self.cmd
    }
}

impl CommandBufferSafe {
    pub fn new(
        device : &Arc<DeviceSafe>,
        pool : &Arc<Pool>
    ) -> Arc<Self> {
        let commandbuf_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(pool.pool)
            .command_buffer_count(1);
        let copycmdbuffer = unsafe {
            device
                .allocate_command_buffers(&commandbuf_allocate_info)
        }
            .unwrap()[0];

        Arc::new(
            Self {
                cmd : copycmdbuffer,
                device : device.clone(),
                pool : pool.clone(),
                depends : vec![]
            }
        )
    }
}