use std::ops::Deref;
use std::sync::Arc;
use ash::vk;
use gpu_allocator::vulkan::AllocationCreateDesc;
use log::*;
use crate::AllocatorSafe;

pub struct BufferSafe {
    pub buffer : vk::Buffer,
    pub allocation : Option<gpu_allocator::vulkan::Allocation>,
    pub allocator : Arc<AllocatorSafe>,
    pub size_in_bytes: u64,
    buffer_usage: vk::BufferUsageFlags,
    memory_usage: gpu_allocator::MemoryLocation
}

impl BufferSafe {
    pub fn new(
        allocator: &Arc<AllocatorSafe>,
        size_in_bytes: u64,
        usage: vk::BufferUsageFlags,
        memory_usage: gpu_allocator::MemoryLocation,
    ) -> Result<BufferSafe, Box<dyn std::error::Error>> {

        let buffer = unsafe {
            allocator.device.create_buffer(
                &ash::vk::BufferCreateInfo::builder()
                    .size(size_in_bytes)
                    .usage(usage)
                    .build(),
                None
            ).unwrap()
        };

        let allocation = unsafe {
            allocator.allocate(
                &AllocationCreateDesc {
                    name : "Buffer",
                    requirements : allocator.device.get_buffer_memory_requirements(buffer),
                    location : memory_usage,
                    linear: true
                }
            ).unwrap()
        };

        unsafe {
            allocator.device.bind_buffer_memory(
                buffer,
                allocation.memory(),
            allocation.offset()).unwrap();
        }

        Ok(BufferSafe {
            buffer,
            allocation : Some(allocation),
            allocator : allocator.clone(),
            size_in_bytes,
            buffer_usage : usage,
            memory_usage
        })
    }

    pub fn fill<T: Sized>(
        &mut self,
        data: &[T],
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let data_ptr = self.allocation.as_ref().unwrap().mapped_ptr().expect("problem with unwrapping").as_ptr() as *mut T;
        unsafe { data_ptr.copy_from_nonoverlapping(data.as_ptr(), data.len()) };

        Ok(())
    }
}

impl Drop for BufferSafe {
    fn drop(&mut self) {
        debug!("Destroy buffer");
        self.allocator.free(self.allocation.take().unwrap());
        unsafe {
            self.allocator.device.destroy_buffer(self.buffer, None);
        }
    }
}

impl Deref for BufferSafe {
    type Target = vk::Buffer;

    fn deref(&self) -> &Self::Target {
        &self.buffer
    }
}