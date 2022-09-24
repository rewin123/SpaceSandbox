use std::ops::Deref;
use std::sync::Arc;
use ash::vk;
use log::{debug, info};
use crate::AllocatorSafe;

pub struct BufferSafe {
    pub buffer : vk::Buffer,
    pub allocation : vk_mem::Allocation,
    pub allocation_info : vk_mem::AllocationInfo,
    pub allocator : Arc<AllocatorSafe>,
    pub size_in_bytes: u64,
    buffer_usage: vk::BufferUsageFlags,
    memory_usage: vk_mem::MemoryUsage
}

impl BufferSafe {
    pub fn new(
        allocator: &Arc<AllocatorSafe>,
        size_in_bytes: u64,
        usage: vk::BufferUsageFlags,
        memory_usage: vk_mem::MemoryUsage,
    ) -> Result<BufferSafe, vk_mem::error::Error> {
        let allocation_create_info = vk_mem::AllocationCreateInfo {
            usage: memory_usage,
            ..Default::default()
        };
        let (buffer, allocation, allocation_info) = allocator.create_buffer(
            &ash::vk::BufferCreateInfo::builder()
                .size(size_in_bytes)
                .usage(usage)
                .build(),
            &allocation_create_info,
        )?;
        Ok(BufferSafe {
            buffer,
            allocation,
            allocation_info,
            allocator : allocator.clone(),
            size_in_bytes,
            buffer_usage : usage,
            memory_usage
        })
    }

    pub fn fill<T: Sized>(
        &mut self,
        data: &[T],
    ) -> Result<(), vk_mem::error::Error> {
        let bytes_to_write = (data.len() * std::mem::size_of::<T>()) as u64;
        if bytes_to_write > self.size_in_bytes {
            // self.allocator.destroy_buffer(self.buffer, &self.allocation);
            let newbuffer = BufferSafe::new(
                &self.allocator,
                bytes_to_write,
                self.buffer_usage,
                self.memory_usage,
            )?;
            *self = newbuffer;
        }
        let data_ptr = self.allocator.map_memory(&self.allocation)? as *mut T;
        unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
        self.allocator.unmap_memory(&self.allocation);
        Ok(())
    }
}

impl Drop for BufferSafe {
    fn drop(&mut self) {
        debug!("Destroy buffer");
        unsafe {
            self.allocator.destroy_buffer(self.buffer, &self.allocation);
        }
    }
}

impl Deref for BufferSafe {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}