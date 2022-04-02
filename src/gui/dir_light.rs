use specs::{World, WorldExt, Join};

use crate::game_object::DirectLight;


pub fn draw(
    ctx : &egui::CtxRef,
    world : &World
) {
    egui::Window::new("Sun").show(&ctx, |ui| {
        let mut dir_light_read = world.write_storage::<DirectLight>();

        for (light,) in (&mut dir_light_read,).join() {
            ui.label("Light");
            
            ui.label("Dir X");
            ui.add(egui::widgets::Slider::new(
                &mut light.dir.x, -1.0..=1.0));
                
            ui.label("Dir Y");
            ui.add(egui::widgets::Slider::new(
                &mut light.dir.y, -1.0..=1.0));
                
            ui.label("Dir Z");
            ui.add(egui::widgets::Slider::new(
                &mut light.dir.z, -1.0..=1.0));

            let mut mgn = light.dir.x * light.dir.x + light.dir.y * light.dir.y + light.dir.z * light.dir.z;
            mgn = mgn.sqrt();
            light.dir /= mgn;

            let mut rgb : [f32; 3] = light.color.into();
            egui::widgets::color_picker::color_edit_button_rgb(ui, &mut rgb);
            light.color = rgb.into();
        }
    });
}