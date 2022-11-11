use std::time::Instant;
use space_core::ecs::*;
use egui::Ui;
use crate::{EguiContext, Game, GlobalStageStep, PluginName, SchedulePlugin};

fn fps_counter_system(
    mut ctx : Res<EguiContext>,
    mut fps : ResMut<FpsCounter>
) {
    egui::TopBottomPanel::top("FpsCounter").show(&ctx, |ui| {
        fps.draw(ui);
    });
}


pub struct FpsCounterSystem {

}

impl SchedulePlugin for FpsCounterSystem {
    fn get_name(&self) -> PluginName {
        PluginName::Text("FpsCounter".into())
    }

    fn add_system(&self, game: &mut Game, builder: &mut Schedule) {
        let fps = FpsCounter::default();
        game.scene.world.insert_resource(fps);

        builder.add_system_to_stage(GlobalStageStep::Gui, fps_counter_system);
    }
}

#[derive(Default)]
pub struct FpsCounter {
    prev : Option<std::time::Instant>,
    counter : i32,
    last_fps : i32
}

impl FpsCounter {
    pub fn draw(&mut self, ui : &mut Ui) {
        let cur_frame = std::time::Instant::now();

        self.counter += 1;

        match self.prev {
            None => {
                ui.label(format!("FPS: undefined"));
                self.prev = Some(cur_frame);
                self.counter = 1;
            }
            Some(v) => {
                let dur = cur_frame - v;
                if dur.as_secs_f32() >= 1.0 {
                    self.prev = Some(cur_frame);
                    let fps = ((self.counter as f32) / dur.as_secs_f32()) as i32;
                    ui.label(format!("FPS: {}", fps));
                    self.last_fps = fps;
                    self.counter = 1;
                } else {
                    ui.label(format!("FPS: {}", self.last_fps));
                }
            }
        }
    }
}