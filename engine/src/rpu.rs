use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage, PrimaryCommandBuffer};
use vulkano::instance::{Instance, InstanceExtensions};
use vulkano::Version;
use vulkano::device::physical::{PhysicalDevice, QueueFamily};
use vulkano::device::{Device, DeviceExtensions, Features, QueuesIter, Queue};
use std::sync::Arc;
use vulkano::image::*;

pub struct RPU {
    pub instance: Arc<Instance>,
    pub device: Arc<Device>,
    pub queue : Arc<Queue>
}

impl RPU {
    pub fn create_image(&self, width : u32, height : u32, format : vulkano::format::Format) -> Result<Arc<StorageImage>, ImageCreationError> {
        StorageImage::new(
            self.device.clone(),
            ImageDimensions::Dim2d {
                width,
                height,
                array_layers : 1
            },
            format,
            Some(self.queue.family())
        )
    }

    
}

impl Default for RPU {
    fn default() -> Self {
        let instance = 
        Instance::new(None, Version::V1_1, &InstanceExtensions::none(), None).expect("Failed to create Instance");
        let physical = PhysicalDevice::enumerate(&instance).next().expect("Physical device not found");

        // println!("Selected physical device: {}", physical.properties().device_name);

        // for family in physical.queue_families() {
        //     println!("Found family with {:?} queue(s)", family.queues_count());
        //     println!("Support compute: {}", family.supports_compute());
        //     println!("Support graphics: {}", family.supports_graphics());
        //     println!("Support sparce bindings: {}", family.supports_sparse_binding());
        // }

        let queues_family = physical.queue_families()
            .find(|&q| q.supports_graphics())
            .expect("couldn't find a graphical queue family");

        let (device, mut queues) = {
            Device::new(
                physical, 
                &Features::none(),
                &DeviceExtensions::none(), 
                            [(queues_family, 0.5)].iter().cloned()).expect("Failed to create device")
        };

        let queue = queues.next().expect("Failed to grab first queue");
        
        Self {
            instance,
            device,
            queue
        }

        }
}