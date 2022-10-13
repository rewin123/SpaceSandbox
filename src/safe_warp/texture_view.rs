use std::sync::Arc;
use ash::vk;
use crate::TextureSafe;

pub struct TextureView {
    pub view : vk::ImageView,
    pub texture : Arc<TextureSafe>
}

impl Drop for TextureView {
    fn drop(&mut self) {
        unsafe {
            self.texture.device.destroy_image_view(self.view, None);
        }
    }
}