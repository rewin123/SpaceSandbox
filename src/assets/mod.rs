pub mod runtime_gpu_assets;
pub mod asset_server;
pub mod texture_server;
pub mod wavefront;
pub mod handle;
pub mod asset_holder;

pub use texture_server::*;

#[derive(Clone, Debug)]
pub enum AssetPath {
    Uri(String)
}