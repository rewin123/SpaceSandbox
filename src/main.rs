use std::cell::RefCell;
use std::fs::File;
use std::io::Read;
use std::ops::Deref;
use std::os::raw::c_char;
use std::sync::Arc;
use ash::{Device, Entry, Instance, vk};
use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::extensions::khr::Swapchain;
use ash::vk::{BufferUsageFlags, CommandBuffer, DeviceQueueCreateInfo, Handle, PhysicalDevice, PhysicalDeviceProperties, SurfaceKHR, SwapchainKHR};
use byteorder::ByteOrder;
use egui::panel::TopBottomSide;
use gltf::{Attribute, Semantic};
use gltf::accessor::DataType;
use gltf::buffer::{Source, Target};
use gltf::json::accessor::ComponentType;


use log::*;
use nalgebra::inf;
use simplelog::*;
use tobj::LoadError;
use vk_mem::MemoryUsage;
use winit::platform::unix::WindowExtUnix;
use winit::window::Window;

use SpaceSandbox::*;
use SpaceSandbox::example_pipeline::ExamplePipeline;
use SpaceSandbox::render_server::Model;

// for time measure wolfpld/tracy


fn main() {
    init_logger();

    let eventloop = winit::event_loop::EventLoop::new();
    let window = winit::window::Window::new(&eventloop).unwrap();
    info!("Created window");

    let mut graphic_base = GraphicBase::init(window);

    let mut camera = RenderCamera::new(&graphic_base.allocator);
    camera.aspect = (graphic_base.swapchain.extent.width as f32) / (graphic_base.swapchain.extent.height as f32);
    camera.update_projectionmatrix();

    let mut gray_draw = SingleTexturePipeline::new(&graphic_base, &camera).unwrap();

    let pools = Pools::init(
        &graphic_base.device,
        &graphic_base.queue_families
    ).unwrap();

    let mut scene : Vec<RenderModel> = vec![];

    let sponza = gltf::Gltf::open("res/test_res/models/sponza/glTF/Sponza.gltf").unwrap();
    let base = "res/test_res/models/sponza/glTF";

    let mut buffers = vec![];
    for buf in sponza.buffers() {
        match buf.source() {
            Source::Bin => {
                error!("Bin buffer not supported!");
            }
            Source::Uri(uri) => {
                info!("Loading buffer {} ...", uri);
                let mut f = std::fs::File::open(format!("{}/{}", &base, uri)).unwrap();
                let metadata = std::fs::metadata(&format!("{}/{}", &base, uri)).unwrap();
                let mut byte_buffer = vec![0; metadata.len() as usize];
                f.read(&mut byte_buffer).unwrap();
                buffers.push(byte_buffer);
            }
        }
    }

    let mut images = vec![];

    for img_meta in sponza.images() {
        match img_meta.source() {
            gltf::image::Source::Uri {uri, mime_type} => {
                let path = format!("{}/{}", base, uri);
                info!("Loading texture {} ...", path);
                images.push(
                  Arc::new(
                      RefCell::new(
                      TextureSafe::from_file(path, &graphic_base, &pools).unwrap()
                      )
                  )
                );
            }
            _ => {
                panic!("Not supported source for texture");
            }
        }
    }

    for m in sponza.meshes() {
        for p in m.primitives() {
            let mut pos : Vec<f32> = vec![];
            let mut normals : Vec<f32> = vec![];
            let mut uv : Vec<f32> = vec![];

            let indices_acc = p.indices().unwrap();
            let indices_view = indices_acc.view().unwrap();
            let mut indices;

            info!("ind: {:?}", indices_acc.data_type());

            match indices_acc.data_type() {
                ComponentType::U16 => {
                    indices = vec![0; indices_view.length() / 2];
                    let buf = &buffers[indices_view.buffer().index()];
                    for idx in 0..indices.len() {
                        let global_idx = idx * indices_view.stride().unwrap_or(2) + indices_view.offset();
                        indices[idx] = byteorder::LittleEndian::read_u16(&buf[global_idx..(global_idx + 2)]) as u32;
                    }
                }
                _ => {panic!("Unsupported index type!");}
            }



            for (sem, acc) in p.attributes() {
                // match  { }
                let view = acc.view().unwrap();
                let mut data = vec![0.0f32; view.length() / 4];

                let buf = &buffers[view.buffer().index()];

                for idx in 0..data.len() {
                    let global_idx = idx * view.stride().unwrap_or(4) + view.offset();
                    data[idx] = byteorder::LittleEndian::read_f32(&buf[global_idx..(global_idx+4)]);
                }

                match sem {
                    Semantic::Positions => {
                        pos.extend(data.iter());

                    }
                    Semantic::Normals => {
                        normals.extend(data.iter());
                    }
                    Semantic::Tangents => {}
                    Semantic::Colors(_) => {}
                    Semantic::TexCoords(_) => {
                        uv.extend(data.iter());
                    }
                    Semantic::Joints(_) => {}
                    Semantic::Weights(_) => {}
                    _ => {}
                }
            }
            info!("Loaded mesh with {} positions and {} normals", pos.len(), normals.len());

            //load diffuse texture


            let mut pos_buffer = BufferSafe::new(
                &graphic_base.allocator,
                pos.len() as u64 * 4,
                    BufferUsageFlags::VERTEX_BUFFER,
            MemoryUsage::CpuToGpu).unwrap();
            let mut normal_buffer = BufferSafe::new(
                &graphic_base.allocator,
                pos.len() as u64 * 4,
                BufferUsageFlags::VERTEX_BUFFER,
                MemoryUsage::CpuToGpu).unwrap();
            let mut index_buffer = BufferSafe::new(
                &graphic_base.allocator,
                indices.len() as u64 * 4,
                BufferUsageFlags::INDEX_BUFFER,
                MemoryUsage::CpuToGpu
            ).unwrap();

            let mut uv_buffer = BufferSafe::new(
                &graphic_base.allocator,
                uv.len() as u64 * 4,
                BufferUsageFlags::VERTEX_BUFFER,
                MemoryUsage::CpuToGpu
            ).unwrap();

            pos_buffer.fill(&pos).unwrap();
            normal_buffer.fill(&normals).unwrap();
            index_buffer.fill(&indices).unwrap();
            uv_buffer.fill(&uv).unwrap();

            let mesh = GPUMesh {
                pos_data: pos_buffer,
                normal_data: normal_buffer,
                index_data: index_buffer,
                uv_data : uv_buffer,
                vertex_count: indices.len() as u32,
                name: "".to_string()
            };

            let material = Material {

            };

            let model = RenderModel {
                mesh,
                material
            };

            scene.push(model);
            // break;
        }
    }

    info!("Finish loading");

    unsafe {
        graphic_base.device.device_wait_idle().unwrap();
    }

    let command_buffers = create_commandbuffers(
        &graphic_base.device,
        &pools,
        graphic_base.swapchain.imageviews.len()
    ).unwrap();

    let mut gui = EguiWrapper::new(
        &graphic_base
    );

    use winit::event::{Event, WindowEvent};
    eventloop.run(move |event, _, controlflow| {

      gui.integration.handle_event(&event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                unsafe {
                    info!("Finishing.....");
                    graphic_base.device.device_wait_idle().expect("Waiting problem");
                }

                *controlflow = winit::event_loop::ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // doing the work here (later)
                graphic_base.window.request_redraw();
            }
            Event::WindowEvent {
                event: WindowEvent::KeyboardInput { input, .. },
                ..
            } => {
                if let winit::event::KeyboardInput {
                    state: winit::event::ElementState::Pressed,
                    virtual_keycode: Some(keycode),
                    ..
                } = input
                {
                    let mut move_vector = nalgebra::Vector3::new(0.0, 0.0 ,0.0);
                    match keycode {
                        winit::event::VirtualKeyCode::Right => {
                            move_vector.x = 1.0;
                        }
                        winit::event::VirtualKeyCode::Left => {
                            move_vector.x = -1.0;
                        }
                        winit::event::VirtualKeyCode::Up => {
                            move_vector.y = 1.0;
                        }
                        winit::event::VirtualKeyCode::Down => {
                            move_vector.y = -1.0;
                        }
                        winit::event::VirtualKeyCode::PageUp => {
                            move_vector.z = 1.0;
                        }
                        winit::event::VirtualKeyCode::PageDown => {
                            move_vector.z = -1.0;
                        }
                        _ => {}
                    }

                    move_vector *= 0.1f32;
                    let frw = camera.view_direction;
                    let up = -camera.down_direction;
                    let right = camera.get_right_vector();
                    let mut dist = camera.position.magnitude();
                    dist += move_vector.z * dist;

                    let dp : nalgebra::Vector3<f32> = move_vector.x * right + up.scale( move_vector.y) + frw.scale(move_vector.z);
                    camera.position = camera.position + dp * dist;
                    camera.position = camera.position.normalize().scale(dist);
                    camera.view_direction = -camera.position.normalize();
                    camera.down_direction = camera.view_direction.cross(&right);

                    info!("{:#?}", camera.position);
                }
            }
            Event::RedrawRequested(_) => {
                //render here (later)
                // info!("Start frame!");
                let image_index = graphic_base.next_frame();

                unsafe {

                    gui.integration.begin_frame();


                    egui::Window::new("Loaded meshes")
                        .hscroll(true)
                        .resizable(true)
                        .show(&gui.integration.context(), |ui| {

                            let mut del_mesh = None;
                            for (idx, m) in scene.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{} : {} verts", m.mesh.name.clone(), m.mesh.vertex_count));
                                    if ui.button("Delete").clicked() {
                                        del_mesh = Some(idx);
                                    }
                                });
                            }
                            match del_mesh {
                                Some(idx) => {
                                    scene.remove(idx);
                                }
                                None => {}
                            }


                        });
                    let (_, shapes) = gui.integration.end_frame(&mut graphic_base.window);
                    let clipped_meshes = gui.integration.context().tessellate(shapes);

                    camera.update_viewmatrix();
                    camera.update_inner_buffer();

                    unsafe {
                        graphic_base.device.begin_command_buffer(command_buffers[image_index as usize], &vk::CommandBufferBeginInfo::builder());
                    }

                    gray_draw.update_commandbuffer(
                        command_buffers[image_index as usize],
                        &graphic_base.device,
                        &graphic_base.swapchain,
                        &scene,
                        image_index as usize
                    );


                    gui.integration.paint(command_buffers[image_index as usize], image_index as usize, clipped_meshes);

                    unsafe {
                        graphic_base.device.end_command_buffer(command_buffers[image_index as usize]).unwrap();
                    }

                    graphic_base.end_frame(&command_buffers, image_index);

                    unsafe {
                        // info!("Wait device");
                        // graphic_base.device.device_wait_idle().unwrap();
                    }
                };
            }
            _ => {}
        }
    });
}

