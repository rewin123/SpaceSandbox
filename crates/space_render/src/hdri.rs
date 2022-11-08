use std::ops::DerefMut;
use legion::systems::Builder;
use legion::*;
use space_assets::Location;
use space_core::Camera;
use space_game::{Game, PluginName, PluginType, SchedulePlugin};

#[system(for_each)]
fn hdri_update(loc : &mut Location, hdri : &HDRISphere, #[resource] camera : &Camera) {
    loc.pos = [
        camera.pos.x,
        camera.pos.y,
        camera.pos.z
    ].into();
}

pub struct HDRISphere {

}

pub struct HDRISystem {
    pub path : String
}

impl SchedulePlugin for HDRISystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("HDRI".into())
    }

    fn get_plugin_type(&self) -> PluginType {
        PluginType::RenderPrepare
    }

    fn add_system(&self, game: &mut Game, builder: &mut Builder) {
        { //clearing
            let mut query = <(Entity, &HDRISphere, )>::query();
            let del_list: Vec<Entity> = query.iter(&game.scene.world).map(|(e, h)| {
                e.clone()
            }).collect();
            for e in del_list {
                game.scene.world.remove(e);
            }
        }

        let sphere = space_assets::wavefront::wgpu_load_gray_obj(
            &game.render_base.device,
            "res/base_models/sphere.obj".into()).unwrap()[0].clone();
        let mut location = Location::new(&game.render_base.device);
        location.pos.x = 10.0;
        location.scale *= -9000.0;
        let mut material = game.get_default_material();
        material.color =
            game.get_assets().deref_mut().load_color_texture(self.path.clone(), true);
        game.scene.world.push((location, sphere, material, HDRISphere {}));

        builder.add_system(hdri_update_system());
    }
}