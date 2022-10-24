pub mod runtime_gpu_assets;
pub mod asset_server;
pub mod handle;
pub mod asset_holder;
pub mod mesh;
pub mod wavefront;
pub mod gltf_loader;
pub mod mipmap_generator;

pub use gltf_loader::*;
pub use asset_server::*;
pub use handle::*;
pub use mesh::*;

#[derive(Clone, Debug)]
pub enum AssetPath {
    GlobalPath(String),
    Binary(Vec<u8>),
    Text(String)
}