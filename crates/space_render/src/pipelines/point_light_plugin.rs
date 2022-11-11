use space_game::*;
use crate::light::PointLightShadow;
use crate::pipelines::PointLightShadowPipeline;

use space_assets::GMeshPtr;
use space_assets::Material;
use space_assets::Location;
use crate::light::PointLight;

use wgpu_profiler::GpuProfiler;
use space_core::ecs::*;

use super::DirLightTexture;
use super::GFramebuffer;
use super::PointLightPipeline;


fn point_light_shadow(
    mut shadow_fill : ResMut<PointLightShadowPipeline>,
    mesh_query : Query<(&GMeshPtr, &Material, &Location)>,
    light_query : Query<(&mut PointLight)>,
    mut encoder : ResMut<wgpu::CommandEncoder>,
    mut profiler : ResMut<GpuProfiler>
) {
    // profiler.begin_scope("Point light shadow", encoder, &shadow_fill.render.device);
    shadow_fill.draw(encoder.as_mut(), mesh_query, light_query, profiler.as_mut());
    // profiler.end_scope(encoder);
}

fn point_light_impl(
    mut fill : ResMut<PointLightPipeline>,
    query : Query<&PointLight>,
    mut encoder : ResMut<wgpu::CommandEncoder>,
    profiler : ResMut<GpuProfiler>,
    dst : Res<DirLightTexture>,
    gbuffer : Res<GFramebuffer>
) {
    // profiler.begin_scope("Point light fill", encoder, &fill.render.device);
    let render = fill.render.clone();
    fill.draw(&render.device, encoder.as_mut(), query, &dst.tex, gbuffer.as_ref());
    // profiler.end_scope(encoder);
}


pub struct PointLightPlugin {

}

impl SchedulePlugin for PointLightPlugin {
    fn get_name(&self) -> PluginName {
        PluginName::Text("Point light".into())
    }

    fn add_system(&self, game: &mut Game, builder: &mut Schedule) {
        let pipeline = PointLightShadowPipeline::new(&game.render_base);
        builder.add_system_to_stage(GlobalStageStep::Render, point_light_shadow);

        game.scene.app.insert_resource(pipeline);

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

        builder.add_system_to_stage( GlobalStageStep::Render,point_light_impl);
        game.scene.app.insert_resource(pipeline);
        game.scene.app.insert_resource( DirLightTexture {
            tex
        });
    }
}