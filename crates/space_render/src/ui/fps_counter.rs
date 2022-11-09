use std::time::Instant;
use egui::Ui;

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