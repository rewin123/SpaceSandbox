

pub mod max_image_fragment {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/render/shaders/image_max.glsl",
    }
}

pub mod min_image_fragment {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/render/shaders/image_min.glsl",
    }
}

pub mod eye_fragment {
    vulkano_shaders::shader!{
        ty: "fragment",
        path : "src/render/shaders/image_eye.glsl",
    }
}