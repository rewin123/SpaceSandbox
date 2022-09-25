use std::{collections::HashMap, sync::Weak, sync::Arc};

use crate::TextureSafe;


struct ServerTexture {
    pub server_index : usize,
    pub texture : Arc<TextureSafe>
}

struct TextureServer {
    pub textures : HashMap<usize, Weak<TextureSafe>>,
    default_texture : Arc<TextureSafe>
}