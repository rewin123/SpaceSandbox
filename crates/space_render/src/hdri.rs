use std::ops::DerefMut;
use bevy::prelude::Assets;
use space_assets::{Location, SpaceAssetServer, Material};
use space_core::{Camera, app::App};
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

    fn add_system(&self, app: &mut App) {
        
        let render = app.world.get_resource::<RenderApi>().unwrap().base.clone();
        let size = app.world.get_resource::<ScreenSize>().unwrap().size.clone();

        { //clearing
            let mut query = app.world.query::<(Entity, &HDRISphere, )>();
            let del_list: Vec<Entity> = query.iter(&app.world).map(|(e, h)| {
                e.clone()
            }).collect();
            for e in del_list {
                app.world.despawn(e);
            }
        }

        let sphere = space_assets::wavefront::wgpu_load_gray_obj(
            &render.device,
            "res/base_models/sphere.obj".into()).unwrap()[0].clone();
        let mut location = Location::new(&render.device);
        location.pos.x = 10.0;
        location.scale *= -9000.0;
        let mut material = app.world.get_resource_mut::<SpaceAssetServer>().unwrap().get_default_material();
        material.color =
            app.world.get_resource_mut::<SpaceAssetServer>().unwrap().load_color_texture(self.path.clone(), true);

        let mut materials = app.world.get_resource_mut::<Assets<Material>>().unwrap();
        let material_handle = materials.add(material);
        app.world.spawn(()).insert((location, sphere, material_handle, HDRISphere {}));

        app.add_system_to_stage(GlobalStageStep::PreRender, hdri_update);
    }
}