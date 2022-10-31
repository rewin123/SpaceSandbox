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

#[system]
#[read_component(GMeshPtr)]
#[read_component(Material)]
#[read_component(Location)]
#[write_component(PointLight)]
fn point_light_shadow(
    #[state] shadow_fill : &mut PointLightShadowPipeline,
    world : &mut SubWorld,
    #[resource] encoder : &mut wgpu::CommandEncoder,
) {
    shadow_fill.draw(encoder, world);
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
    }
}