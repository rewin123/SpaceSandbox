use bevy::prelude::Assets;
use bevy::prelude::Handle;
use space_assets::{GMesh, TransformBuffer};
use space_core::app::App;
use space_game::*;
use crate::light::PointLightShadow;
use crate::pipelines::PointLightShadowPipeline;

use space_assets::Material;
use crate::light::PointLight;

use space_core::ecs::*;

use super::DirLightTexture;
use super::GFramebuffer;
use super::PointLightPipeline;


fn point_light_shadow(
    mut shadow_fill : ResMut<PointLightShadowPipeline>,
    mesh_query : Query<(&Handle<GMesh>, &Material, &TransformBuffer)>,
    light_query : Query<(&mut PointLight)>,
    mut encoder : ResMut<RenderCommands>,
    mut meshes : ResMut<Assets<GMesh>>
) {
    // profiler.begin_scope("Point light shadow", encoder, &shadow_fill.render.device);
    shadow_fill.draw(encoder.as_mut(), mesh_query, light_query, meshes);
    // profiler.end_scope(encoder);
}

fn point_light_impl(
    mut fill : ResMut<PointLightPipeline>,
    query : Query<&PointLight>,
    mut encoder : ResMut<RenderCommands>,
    dst : Res<DirLightTexture>,
    gbuffer : Res<GFramebuffer>,
    mut meshes : ResMut<Assets<GMesh>>
) {
    // profiler.begin_scope("Point light fill", encoder, &fill.render.device);
    let render = fill.render.clone();
    fill.draw(&render.device, encoder.as_mut(), query, &dst.tex, gbuffer.as_ref(), meshes.as_mut());
    // profiler.end_scope(encoder);
}


pub struct PointLightPlugin {

}

impl SchedulePlugin for PointLightPlugin {
    fn get_name(&self) -> PluginName {
        PluginName::Text("Point light".into())
    }

    fn add_system(&self, app: &mut App) {

        let render = app.world.get_resource_mut::<RenderApi>().unwrap().base.clone();
        let size = app.world.get_resource::<ScreenSize>().unwrap().size.clone();

        let pipeline = PointLightShadowPipeline::new(
            &render);
        app.add_system_to_stage(GlobalStageStep::Render, point_light_shadow);

        app.insert_resource(pipeline);

        let pipeline = PointLightPipeline::new(
            &render, 
            wgpu::Extent3d {
                width : size.width,
                height : size.height,
                depth_or_array_layers : 1
            },
            app);

        let tex = pipeline.spawn_framebuffer(&render.device, wgpu::Extent3d {
            width : size.width,
            height : size.height,
            depth_or_array_layers : 1
        });

        app.add_system_to_stage( GlobalStageStep::Render,point_light_impl);
        app.insert_resource(pipeline);
        app.insert_resource( DirLightTexture {
            tex
        });
    }
}