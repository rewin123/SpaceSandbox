use egui::Ui;

use crate::{ApiBase, GraphicBase};


pub struct ApiInfoWindow {
    device_name : String
}

impl ApiInfoWindow {
    pub fn new(api : &GraphicBase) -> Self {
        let prop = unsafe {
            api.instance.get_physical_device_properties(api.physical_device)
        };
        
        let name = String::from(
            unsafe { std::ffi::CStr::from_ptr(prop.device_name.as_ptr()) }
                .to_str()
                .unwrap(),
        );

        Self { 
            device_name : name
        }
    }

    pub fn draw(&mut self, ui : &mut Ui) {
        ui.label(format!("Device: {}", self.device_name.clone()));
    }
}