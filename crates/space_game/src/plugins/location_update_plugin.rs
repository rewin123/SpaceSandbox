use space_assets::{TransformBuffer};
use crate::*;
use space_core::ecs::*;
use bevy::prelude::*;
use wgpu::util::DeviceExt;

fn create_global_transform(
    mut cmds : Commands,
    mut query : Query<(Entity), Added<Transform>>
) {
    for e in &query {
        cmds.entity(e).insert(GlobalTransform::default());
    }
}

fn create_buffer(
        mut cmds : Commands,
        mut query : Query<(Entity, &GlobalTransform), Added<GlobalTransform>>,
        api : Res<RenderApi>) {
    for (e, loc) in &query {
        let transform_buffer = TransformBuffer {
            buffer : Arc::new(api.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: None,
                contents: bytemuck::cast_slice(&[0.0f32; 4 * 4 * 2]),
                usage: wgpu::BufferUsages::MAP_WRITE | wgpu::BufferUsages::VERTEX
            }))
        };
        transform_buffer.update_buffer(loc);
        cmds.entity(e).insert(transform_buffer);
        println!("Create transform buffer!");
    }
}

fn update_loc_buffer(mut query: Query<(&mut TransformBuffer, &GlobalTransform), Changed<GlobalTransform>>) {
    for (mut buf, loc) in &mut query {
        buf.update_buffer(&loc);
    }
}


pub struct LocUpdateSystem {

}

impl SchedulePlugin for LocUpdateSystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("LocUpdateSystem".into())
    }

    fn add_system(&self, app:  &mut space_core::app::App) {
        app.add_system_to_stage(CoreStage::Update,update_loc_buffer);
        app.add_system_to_stage(CoreStage::PreUpdate, create_global_transform);
        app.add_system_to_stage(CoreStage::PreUpdate, create_buffer);
    }
}

