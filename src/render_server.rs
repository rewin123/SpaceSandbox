use std::collections::HashMap;
use crate::GPUMesh;

pub struct Model {
    mesh : GPUMesh
}

pub struct ModelInstance {
    pub model : Model,
    //instance info
}

pub struct  RenderServer {
    models : HashMap<i32, ModelInstance>
}