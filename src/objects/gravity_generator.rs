use bevy::{prelude::*, math::DVec3};
use bevy_transform64::prelude::DGlobalTransform;
use serde::{Serialize, Deserialize};


pub struct GravityGeneratorPlugin;

impl Plugin for GravityGeneratorPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(gravity_sensetive_fill);
    }
}

#[derive(Component, Debug, Serialize, Deserialize, Default)]
pub struct GravityGenerator {
    pub gravity_force : DVec3,
    pub radius : f64,
}

#[derive(Component, Debug, Serialize, Deserialize, Default)]
pub struct GravitySenitive {
    pub is_senitive : bool,
    pub g : DVec3,
}

fn gravity_sensetive_fill(
    mut gravity_generators : Query<(Entity, &DGlobalTransform, &mut GravityGenerator)>,
    mut gravity_senitives : Query<(Entity, &DGlobalTransform, &mut GravitySenitive)>
) {
    for (e_s, t_s, mut s) in gravity_senitives.iter_mut() {
        s.is_senitive = false;
        s.g = DVec3::ZERO;
        // info!("Sensitive: {:?}", e_s);
        for (e_g, t_g, mut g) in gravity_generators.iter_mut() {
            let dist = (t_g.translation() - t_s.translation()).length();
            let dir = (t_g.translation() - t_s.translation()).dot(t_g.up());
            
            if dist < g.radius {
                let force = t_g.transform_point(g.gravity_force) - t_g.translation();
                let gravity_force = -dir * force / (9.0 * dist * dist / g.radius / g.radius + 1.0);
                s.is_senitive = true;
                s.g += gravity_force;
                trace!("Sensitive: {:?} with gravity generator: {:?}", e_s, e_g);
            }
        }
    }
}