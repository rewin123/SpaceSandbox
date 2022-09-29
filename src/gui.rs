use std::sync::Mutex;
use crate::*;

pub struct EguiWrapper {
    pub integration : egui_winit_ash_integration::Integration<Arc<Mutex<gpu_allocator::vulkan::Allocator>>>,
    pub allocator : Arc<Mutex<gpu_allocator::vulkan::Allocator>>,
    pub device : Arc<DeviceSafe>
}

impl Drop for EguiWrapper {
    fn drop(&mut self) {
        info!("Destroying egui wrapper.....");
        unsafe {
            self.device.device_wait_idle().expect("Waiting device idle problem");
            self.integration.destroy();
        }
    }
}

impl EguiWrapper {
    pub fn new(
        graphic_base : &GraphicBase
        ) -> Self {

        let allocatorcreatedesc = gpu_allocator::vulkan::AllocatorCreateDesc {
            instance: graphic_base.instance.inner.clone(),
            device: graphic_base.device.inner.clone(),
            physical_device: graphic_base.physical_device.clone(),
            debug_settings: gpu_allocator::AllocatorDebugSettings::default(),
            buffer_device_address: false
        };

        let allocator = gpu_allocator::vulkan::Allocator::new(&allocatorcreatedesc).unwrap();
        let allocator_lock = Arc::new(Mutex::new(allocator));


        let egui_integration = egui_winit_ash_integration::Integration::new(
            graphic_base.swapchain.extent.width,
            graphic_base.swapchain.extent.height,
            graphic_base.window.scale_factor(),
            egui::FontDefinitions::default(),
            egui::Style::default(),
            graphic_base.device.inner.clone(),
            allocator_lock.clone(),
            graphic_base.swapchain.loader.clone(),
            graphic_base.swapchain.inner.clone(),
            graphic_base.swapchain.format.clone()
        );

        let mut style = egui::Style::default();
        style.visuals.widgets.noninteractive.bg_fill = egui::Color32::WHITE;
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke { width: 1.0, color: egui::Color32::BLACK };
        style.visuals.widgets.active.bg_fill = egui::Color32::WHITE;
        style.visuals.widgets.active.fg_stroke = egui::Stroke { width: 1.0, color: egui::Color32::BLACK };
        style.visuals.widgets.inactive.bg_fill = egui::Color32::LIGHT_BLUE;
        style.visuals.widgets.inactive.fg_stroke = egui::Stroke { width: 1.0, color: egui::Color32::BLACK };
        egui_integration.context().set_style(style);

        Self {
            allocator : allocator_lock,
            integration : egui_integration,
            device : graphic_base.device.clone()
        }
    }

    pub fn wait_draw(&self) {
        
    }
}