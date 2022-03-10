
use std::sync::Arc;

use egui::{ScrollArea, TextEdit, TextStyle};
use egui_winit_vulkano::Gui;
use vulkano::{
    device::{physical::PhysicalDevice, Device, Queue, DeviceExtensions, Features},
    image::{view::ImageView, ImageUsage, SwapchainImage},
    instance::{Instance, InstanceExtensions},
    swapchain,
    swapchain::{
        AcquireError, ColorSpace, FullscreenExclusive, PresentMode, Surface, SurfaceTransform,
        Swapchain, SwapchainCreationError,
    },
    sync,
    sync::{FlushError, GpuFuture},
    Version,
};
use vulkano_win::VkSurfaceBuild;
use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};
use crate::rpu::WinRpu;

pub struct SimpleGuiRenderer {
    #[allow(dead_code)]
    win_rpu : WinRpu,
    recreate_swapchain: bool,
    final_images : Vec<Arc<ImageView<SwapchainImage<Window>>>>,
    previous_frame_end: Option<Box<dyn GpuFuture>>,
}

impl SimpleGuiRenderer {
    pub fn new(
        win_rpu : WinRpu,
        window_size: [u32; 2],
        present_mode: PresentMode,
        name: &str,
    ) -> Self {
        let images =
            win_rpu.swapchain_images.clone().into_iter().map(|image| ImageView::new(image).unwrap()).collect::<Vec<_>>();
        let previous_frame_end = Some(sync::now(win_rpu.rpu.device.clone()).boxed());
        Self {
            win_rpu,
            previous_frame_end,
            final_images : images,
            recreate_swapchain: false,
        }
    }

    pub fn queue(&self) -> Arc<Queue> {
        self.win_rpu.rpu.queue.clone()
    }

    pub fn surface(&self) -> Arc<Surface<Window>> {
        self.win_rpu.surface.clone()
    }

    pub fn resize(&mut self) {
        self.recreate_swapchain = true;
    }

    pub fn render(&mut self, gui: &mut Gui) {
        // Recreate swap chain if needed (when resizing of window occurs or swapchain is outdated)
        if self.recreate_swapchain {
            self.recreate_swapchain();
        }
        // Acquire next image in the swapchain and our image num index
        let (image_num, suboptimal, acquire_future) =
            match swapchain::acquire_next_image(self.win_rpu.swapchain.clone(), None) {
                Ok(r) => r,
                Err(AcquireError::OutOfDate) => {
                    self.recreate_swapchain = true;
                    return;
                }
                Err(e) => panic!("Failed to acquire next image: {:?}", e),
            };
        if suboptimal {
            self.recreate_swapchain = true;
        }
        // Render GUI
        let future = self.previous_frame_end.take().unwrap().join(acquire_future);
        let after_future = gui.draw_on_image(future, self.final_images[image_num].clone());
        // Finish render
        self.finish(after_future, image_num);
    }

    fn recreate_swapchain(&mut self) {
        let dimensions: [u32; 2] = self.win_rpu.surface.window().inner_size().into();
        let (new_swapchain, new_images) =
            match self.win_rpu.swapchain.recreate().dimensions(dimensions).build() {
                Ok(r) => r,
                Err(SwapchainCreationError::UnsupportedDimensions) => return,
                Err(e) => panic!("Failed to recreate swapchain: {:?}", e),
            };
        self.win_rpu.swapchain = new_swapchain;
        let new_images =
            new_images.into_iter().map(|image| ImageView::new(image).unwrap()).collect::<Vec<_>>();
        self.final_images = new_images;
        self.recreate_swapchain = false;
    }

    fn finish(&mut self, after_future: Box<dyn GpuFuture>, image_num: usize) {
        let future = after_future
            .then_swapchain_present(self.win_rpu.rpu.queue.clone(), self.win_rpu.swapchain.clone(), image_num)
            .then_signal_fence_and_flush();
        match future {
            Ok(future) => {
                // A hack to prevent OutOfMemory error on Nvidia :(
                // https://github.com/vulkano-rs/vulkano/issues/627
                match future.wait(None) {
                    Ok(x) => x,
                    Err(err) => println!("err: {:?}", err),
                }
                self.previous_frame_end = Some(future.boxed());
            }
            Err(FlushError::OutOfDate) => {
                self.recreate_swapchain = true;
                self.previous_frame_end = Some(sync::now(self.win_rpu.rpu.device.clone()).boxed());
            }
            Err(e) => {
                println!("Failed to flush future: {:?}", e);
                self.previous_frame_end = Some(sync::now(self.win_rpu.rpu.device.clone()).boxed());
            }
        }
    }
}