use legion::systems::Builder;
use space_game::{Game, PluginName, PluginType, SchedulePlugin};
use legion::*;
use legion::world::SubWorld;
use crate::light::PointLightShadow;
use crate::pipelines::PointLightShadowPipeline;

use space_assets::GMeshPtr;
use space_assets::Material;
use space_assets::Location;
use crate::light::PointLight;

use wgpu_profiler::GpuProfiler;

use super::DirLightTexture;
use super::GFramebuffer;
use super::PointLightPipeline;

#[system]
#[read_component(GMeshPtr)]
#[read_component(Material)]
#[read_component(Location)]
#[write_component(PointLight)]
fn point_light_shadow(
    #[state] shadow_fill : &mut PointLightShadowPipeline,
    world : &mut SubWorld,
    #[resource] encoder : &mut wgpu::CommandEncoder,
    #[resource] profiler : &mut GpuProfiler
) {
    profiler.begin_scope("Point light shadow", encoder, &shadow_fill.render.device);
    shadow_fill.draw(encoder, world, profiler);
    profiler.end_scope(encoder);
}

#[system]
#[read_component(PointLight)]
fn point_light_impl(
    #[state] fill : &mut PointLightPipeline,
    world : &mut SubWorld,
    #[resource] encoder : &mut wgpu::CommandEncoder,
    #[resource] profiler : &mut GpuProfiler,
    #[resource] dst : &DirLightTexture,
    #[resource] gbuffer : &GFramebuffer
) {
    profiler.begin_scope("Point light fill", encoder, &fill.render.device);
    let render = fill.render.clone();
    fill.draw(&render.device, encoder, world, &dst.tex, gbuffer);
    profiler.end_scope(encoder);
}


pub struct PointLightPlugin {

}

impl SchedulePlugin for PointLightPlugin {
    fn get_name(&self) -> PluginName {
        PluginName::Text("Point light".into())
    }

    fn get_plugin_type(&self) -> PluginType {
        PluginType::Render
    }

    fn add_system(&self, game: &mut Game, builder: &mut Builder) {
        let pipeline = PointLightShadowPipeline::new(&game.render_base);
        builder.add_system(point_light_shadow_system(pipeline));

        let pipeline = PointLightPipeline::new(&game.render_base, &game.scene.camera_buffer, wgpu::Extent3d {
            width : game.api.size.width,
            height : game.api.size.height,
            depth_or_array_layers : 1
        });

        let tex = pipeline.spawn_framebuffer(&game.render_base.device, wgpu::Extent3d {
            width : game.api.size.width,
            height : game.api.size.height,
            depth_or_array_layers : 1
        });

        builder.add_system(point_light_impl_system(pipeline));
        game.scene.resources.insert( DirLightTexture {
            tex
        });
    }
}