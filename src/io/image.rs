use std::{io::Cursor, sync::Arc};

use image::{ImageBuffer, Rgba, GenericImage, RgbaImage};
use vulkano::{image::{ImageDimensions, ImmutableImage, MipmapsCount, view::ImageView}, format::{Format, ClearValuesTuple}, command_buffer::CommandBufferExecFuture, sync::*, device::Queue};
use vulkano::command_buffer::PrimaryAutoCommandBuffer;
use crate::rpu::RPU;


pub fn image_to_rpu(data : Vec<u8>, rpu : &RPU) -> (Arc<ImmutableImage>, CommandBufferExecFuture<NowFuture, PrimaryAutoCommandBuffer>) {

    let cursor = Cursor::new(data);
    let decoder = png::Decoder::new(cursor);
    let mut reader = decoder.read_info().unwrap();
    let info = reader.info();
    let dimensions = ImageDimensions::Dim2d {
        width: info.width,
        height: info.height,
        array_layers: 1,
    };
    let mut image_data = Vec::new();
    image_data.resize((info.width * info.height * 4) as usize, 0);
    reader.next_frame(&mut image_data).unwrap();

    let (image, future) = ImmutableImage::from_iter(
        image_data,
        dimensions,
        MipmapsCount::One,
        Format::R8G8B8A8_SRGB,
        rpu.queue.clone(),
    ).unwrap();

    (image, future)
}

