use std::ops::DerefMut;
use space_assets::Location;
use space_core::Camera;
use space_game::*;
use space_core::ecs::*;

fn hdri_update(
    mut query : Query<(&mut Location, &HDRISphere)>,
    camera : Res<Camera>) {

    for (mut loc, hdri) in &mut query {
        loc.pos = [
            camera.pos.x,
            camera.pos.y,
            camera.pos.z
        ].into();
    }
}

#[derive(Component)]
pub struct HDRISphere {

}

pub struct HDRISystem {
    pub path : String
}

impl SchedulePlugin for HDRISystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("HDRI".into())
    }

    fn add_system(&self, game: &mut Game, builder: &mut Schedule) {
        { //clearing
            let mut query = game.scene.app.world.query::<(Entity, &HDRISphere, )>();
            let del_list: Vec<Entity> = query.iter(&game.scene.app.world).map(|(e, h)| {
                e.clone()
            }).collect();
            for e in del_list {
                game.scene.app.world.despawn(e);
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
        game.scene.app.world.spawn().insert_bundle((location, sphere, material, HDRISphere {}));

        builder.add_system_to_stage(GlobalStageStep::RenderPrepare, hdri_update);
    }
}