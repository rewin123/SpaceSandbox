use std::{sync::Arc, collections::HashMap};

use ash::vk::{self, ImageView};
use crate::{TextureSafe, DeviceSafe, RenderPassSafe};

pub struct FramebufferSafe {
    pub franebuffer : vk::Framebuffer,
    pub images : Vec<Arc<TextureSafe>>,
    pub renderpass : Arc<RenderPassSafe>,
    pub device : Arc<DeviceSafe>
}

impl Drop for FramebufferSafe {
    fn drop(&mut self) {
        unsafe {
            self.device.destroy_framebuffer(self.franebuffer, None);
        }
    }
}

impl FramebufferSafe {
    pub fn new(renderpass : &Arc<RenderPassSafe>, attachments : &[Arc<TextureSafe>]) -> Self {
        let mut iview_vec = vec![];
        for a in attachments {
            iview_vec.push(a.imageview);
        }
        let framebuffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(renderpass.inner)
            .attachments(&iview_vec)
            .width(attachments[0].get_width())
            .height(attachments[0].get_height())
            .layers(1);
        let fb = unsafe { 
            attachments[0].device.create_framebuffer(&framebuffer_info, None) 
        }.unwrap();


        Self {
            franebuffer : fb,
            images : attachments.iter().map(|v| v.clone()).collect(),
            renderpass : renderpass.clone(),
            device : attachments[0].device.clone()
        }
    }
}

pub struct FramebufferStorage {
    storage : Vec<Arc<FramebufferSafe>>,
    renderpass : Arc<RenderPassSafe>
}


impl FramebufferStorage {

    pub fn new(renderpass : &Arc<RenderPassSafe>) -> Self {
        Self {
            storage : vec![],
            renderpass : renderpass.clone()
        }
    }

    pub fn get_framebuffer(&mut self, attachments : &[Arc<TextureSafe>]) -> Arc<FramebufferSafe> {

        for fb in &self.storage {
            let mut ok = true;
            for (idx, tex) in fb.images.iter().enumerate() {
                if tex.index != attachments[idx].index {
                    ok = false;
                    break;
                }
            }
            if ok {
                return fb.clone();
            }
        }

        let fb = Arc::new(FramebufferSafe::new(&self.renderpass, attachments));
        self.storage.push(fb.clone());
        fb
    }
}