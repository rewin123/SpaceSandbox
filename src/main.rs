use std::io::Read;
use std::sync::Arc;
use ash::{vk};
use ash::vk::{BufferUsageFlags};
use byteorder::ByteOrder;
use gltf::{Semantic};
use gltf::buffer::{Source};
use gltf::json::accessor::ComponentType;


use log::*;

use SpaceSandbox::*;
use SpaceSandbox::MaterialTexture::{Diffuse, MetallicRoughness, Normal};
use SpaceSandbox::task_server::{TaskServer, TaskState};
use SpaceSandbox::ui::*;
use SpaceSandbox::game::*;

// for time measure wolfpld/tracy

fn init_rayon() {
    rayon::ThreadPoolBuilder::default()
        .num_threads(3)
        .build_global().unwrap();
}

fn main() {
    init_logger();
    init_rayon();

    let mut game = Game::default();

    let mut camera = RenderCamera::new(&game.gb.allocator);
    camera.aspect = (game.gb.swapchain.extent.width as f32) / (game.gb.swapchain.extent.height as f32);
    camera.update_projectionmatrix();

    let mut gray_draw = SingleTexturePipeline::new(&game.gb, &camera).unwrap();

    let mut scene : Vec<RenderModel> = vec![];

    // let sponza = gltf::Gltf::open("res/test_res/models/xian_spaceship/scene.gltf").unwrap();
    // let base = "res/test_res/models/xian_spaceship";

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
                images.push(game.textures.load_new_texture(path));
            }
            _ => {
                panic!("Not supported source for texture");
            }
        }
    }

    let mut meshes = vec![];

    for m in sponza.meshes() {
        let mut sub_models = vec![];
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
                    indices = vec![0; indices_acc.count()];
                    let buf = &buffers[indices_view.buffer().index()];
                    info!("indices stride: {:?}", indices_view.stride());
                    for idx in 0..indices.len() {
                        let global_idx = idx * indices_view.stride().unwrap_or(2) + indices_view.offset() + indices_acc.offset();
                        indices[idx] = byteorder::LittleEndian::read_u16(&buf[global_idx..(global_idx + 2)]) as u32;
                    }
                }
                ComponentType::U32 => {
                    indices = vec![0; indices_acc.count()];
                    let buf = &buffers[indices_view.buffer().index()];
                    info!("indices stride: {:?}", indices_view.stride());
                    for idx in 0..indices.len() {
                        let global_idx = idx * indices_view.stride().unwrap_or(4) + indices_view.offset() + indices_acc.offset();
                        indices[idx] = byteorder::LittleEndian::read_u32(&buf[global_idx..(global_idx + 4)]) as u32;
                    }
                }
                _ => {panic!("Unsupported index type!");}
            }



            for (sem, acc) in p.attributes() {
                // match  { }
                let view = acc.view().unwrap();
                let mut data = vec![0.0f32; acc.count() * acc.dimensions().multiplicity()];

                let stride = view.stride().unwrap_or(acc.data_type().size() * acc.dimensions().multiplicity());


                let buf = &buffers[view.buffer().index()];

                for c in 0..acc.count() {
                    for d in 0..acc.dimensions().multiplicity() {
                        let idx = c * acc.dimensions().multiplicity() + d;
                        let global_idx = c * stride + acc.offset() + view.offset() + d * acc.data_type().size();
                        data[idx] = byteorder::LittleEndian::read_f32(&buf[global_idx..(global_idx + 4)]);
                    }
                }

                match sem {
                    Semantic::Positions => {
                        pos.extend(data.iter());
                        info!("Pos {}", acc.dimensions().multiplicity());
                        info!("Stride: {}", stride);
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
                &game.gb.allocator,
                pos.len() as u64 * 4,
                    BufferUsageFlags::VERTEX_BUFFER,
            gpu_allocator::MemoryLocation::CpuToGpu).unwrap();
            let mut normal_buffer = BufferSafe::new(
                &game.gb.allocator,
                pos.len() as u64 * 4,
                BufferUsageFlags::VERTEX_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu).unwrap();
            let mut index_buffer = BufferSafe::new(
                &game.gb.allocator,
                indices.len() as u64 * 4,
                BufferUsageFlags::INDEX_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu
            ).unwrap();

            if uv.len() == 0 {
                uv = vec![0.0f32; pos.len() / 3 * 2];
            }

            let mut uv_buffer = BufferSafe::new(
                &game.gb.allocator,
                uv.len() as u64 * 4,
                BufferUsageFlags::VERTEX_BUFFER,
                gpu_allocator::MemoryLocation::CpuToGpu
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
                name: m.name().unwrap_or("").to_string()
            };

            let normal_tex;
            if let Some(tex) = p.material().normal_texture() {
                normal_tex = images[tex.texture().index()].clone();
            } else {
                normal_tex = game.textures.get_default_color_texture();
            }

            let metallic_roughness;
            if let Some(tex) = p.material().pbr_metallic_roughness().metallic_roughness_texture() {
                metallic_roughness = images[tex.texture().index()].clone();
            } else {
                metallic_roughness = game.textures.get_default_color_texture();
            }

            let material = {
                match p.material().pbr_specular_glossiness() {
                    Some(v) => {

                        let color;
                        if let Some(tex) = v.diffuse_texture() {
                            color = images[tex.texture().index()].clone()
                        } else {
                            color = game.textures.get_default_color_texture();
                        }

                        Material {
                            color,
                            normal : normal_tex,
                            metallic_roughness: metallic_roughness
                        }
                    }
                    None => {
                        Material {
                            color : images[p.material().pbr_metallic_roughness().base_color_texture().unwrap().texture().index()].clone(),
                            normal : normal_tex,
                            metallic_roughness: metallic_roughness
                        }
                    }
                }
            };

            let model = RenderModel::new(&game.gb.allocator,
                Arc::new(mesh),
                material);

            sub_models.push(model);
            // break;
        }
        meshes.push(sub_models);
    }

    for n in sponza.nodes() {
        let matrix = n.transform().matrix();
        if let Some(mesh) = n.mesh() {
            for rm in &mut meshes[mesh.index()] {
                rm.add_matrix(&matrix);
            }
        } else {
            for child in n.children() {
                if let Some(mesh) = child.mesh() {
                    for rm in &mut meshes[mesh.index()] {
                        rm.add_matrix(&matrix);
                    }
                }
            }
        }
    }

    scene = meshes.into_iter().flatten().collect();

    for rm in &mut scene {
        rm.update_instance_buffer().unwrap();
    }

    info!("Finish loading");

    unsafe {
        game.gb.device.device_wait_idle().unwrap();
    }

    let command_buffers = create_commandbuffers(
        &game.gb.device,
        &game.pools,
        game.gb.swapchain.imageviews.len()
    ).unwrap();

    let mut gui = EguiWrapper::new(
        &game.gb
    );

    let mut show_task_list = false;

    let mut fps_counter = FpsCounter::default();
    let mut api_window = ApiInfoWindow::new(&game.gb);


    use winit::event::{Event, WindowEvent};

    game.simple_loop(
     move |game, event, _, controlflow| {

      gui.integration.handle_event(&event);

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                unsafe {
                    info!("Finishing.....");
                    game.gb.device.device_wait_idle().expect("Waiting problem");
                }

                *controlflow = winit::event_loop::ControlFlow::Exit;
            }
            Event::MainEventsCleared => {
                // doing the work here (later)
                game.gb.window.request_redraw();
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
                let image_index = game.gb.next_frame();

                unsafe {

                    gui.integration.begin_frame();

                    egui::TopBottomPanel::top(0).show(&gui.integration.context(), |ui| {
                        ui.horizontal(|ui| {
                            if ui.button(format!("{} tasks running", game.task_server.get_task_count())).clicked() {
                                show_task_list = true;
                            }
                            if ui.button(format!("{:?}", &gray_draw.mode)).clicked() {
                                match gray_draw.mode {
                                    MaterialTexture::Diffuse => {
                                        gray_draw.mode = Normal;
                                    }
                                    MaterialTexture::Normal => {
                                        gray_draw.mode = MetallicRoughness;
                                    }
                                    MaterialTexture::MetallicRoughness => {
                                        gray_draw.mode = Diffuse;
                                    }
                                }
                            }
                            fps_counter.draw(ui);
                            api_window.draw(ui);
                        });
                    });

                    if show_task_list {
                        let win_res = egui::Window::new("Task list")
                            .show(&gui.integration.context(), |ui| {

                            if ui.button("Close").clicked() {
                                show_task_list = false;
                            }
                            let tasks = game.task_server.clone_task_list();
                            for t in tasks {
                                let state = t.get_state();
                                match state {
                                    TaskState::Created => {
                                        ui.label(t.get_name());
                                    }
                                    TaskState::Running => {
                                        ui.colored_label(egui::color::Color32::GREEN, t.get_name());
                                    }
                                    TaskState::Finished => {
                                        ui.colored_label(egui::color::Color32::RED, t.get_name());
                                    }
                                }
                            }
                        });
                    }

                    let (_, shapes) = gui.integration.end_frame(&mut game.gb.window);
                    let clipped_meshes = gui.integration.context().tessellate(shapes);

                    camera.update_viewmatrix();
                    camera.update_inner_buffer();

                    unsafe {
                        game.gb.device.begin_command_buffer(command_buffers[image_index as usize], &vk::CommandBufferBeginInfo::builder()).unwrap();
                    }

                    
                    gray_draw.update_commandbuffer(
                        command_buffers[image_index as usize],
                        &game.gb.device,
                        &game.gb.swapchain,
                        &scene,
                        &game.textures,
                        image_index as usize
                    ).unwrap();

                    gui.integration.paint(command_buffers[image_index as usize], image_index as usize, clipped_meshes);

                    unsafe {
                        game.gb.device.end_command_buffer(command_buffers[image_index as usize]).unwrap();
                    }

                    game.gb.end_frame(&command_buffers, image_index);

                    game.textures.sync_tick();

                    unsafe {
                        // info!("Wait device");
                        // game.gb.device.device_wait_idle().unwrap();
                    }
                };
            }
            _ => {}
        }
    });
}

