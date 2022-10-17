pub mod wgpu_gbuffer_fill;
pub mod wgpu_light_fill;
pub mod wgpu_texture_present;
pub mod wgpu_light_shadow;


// pub trait InstancesDrawer {
//     fn process(
//         &mut self,
//         cmd : CommandBuffer,
//         input : &[Arc<TextureSafe>],
//         fb : &Arc<FramebufferSafe>,
//         server : &RenderServer,
//         assets : &AssetServer);
//     fn create_framebuffer(&mut self) -> Arc<FramebufferSafe>;
//     fn set_camera(&mut self, camera : &RenderCamera);
// }

// pub trait ShadowPrepare {
//     fn process(
//         &mut self,
//         cmd : CommandBuffer,
//         server : &mut RenderServer,
//         assets : &AssetServer);
//     fn create_framebuffer(&mut self) -> Arc<FramebufferSafe>;
// }

// pub trait TextureTransform {
//     fn process(&mut self, cmd : CommandBuffer, dst : &Arc<FramebufferSafe>, input : Vec<Arc<TextureSafe>>);
//     fn create_framebuffer(&mut self) -> Arc<FramebufferSafe>;
// }