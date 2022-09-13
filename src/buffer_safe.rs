use std::ops::Deref;
use std::sync::Arc;
use ash::vk;
use log::info;
use crate::AllocatorSafe;

pub struct BufferSafe {
    pub buffer : vk::Buffer,
    pub allocation : vk_mem::Allocation,
    pub allocation_info : vk_mem::AllocationInfo,
    pub allocator : Arc<AllocatorSafe>
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
            allocator : allocator.clone()
        })
    }

    pub fn fill<T: Sized>(
        &self,
        data: &[T],
    ) -> Result<(), vk_mem::error::Error> {
        let data_ptr = self.allocator.map_memory(&self.allocation)? as *mut T;
        unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };
        self.allocator.unmap_memory(&self.allocation);
        Ok(())
    }
}

impl Drop for BufferSafe {
    fn drop(&mut self) {
        info!("Destroy buffer");
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