use std::sync::Arc;

use egui::{Context, Ui, Galley, text::LayoutJob, Style, TextFormat, Color32};

use crate::{AssetPath, asset_server::AssetServer};


pub struct PipelineEditor {
    pub shader_text : String,
    pub shader_path : AssetPath,
    pub show : bool,
    pub select_file : bool,
    pub shader_paths : Vec<String>
}

impl Default for PipelineEditor {
    fn default() -> Self {
        PipelineEditor {
            shader_text: "".into(),
            show: false,
            shader_path: AssetPath::Text("".into()),
            select_file: false,
            shader_paths : vec![]
        }
    }
}

impl PipelineEditor {

    pub fn draw_button(&mut self, ui : &mut Ui) {
        if ui.button("Pipeline editor").clicked() {
            self.show = true;
        }
    }

    fn popup_select_file(&mut self, ctx : &Context, assets : &AssetServer) {
        let mut window = egui::Window::new("Select shader")
            .collapsible(false);
        
        window.show(ctx, |ui| {
            for file in &self.shader_paths {
                if ui.button(file).clicked() {
                    self.shader_path = AssetPath::GlobalPath(file.clone());
                    self.select_file = false;
                    self.shader_text = assets.get_file_text(&self.shader_path).unwrap();
                }
            }
        });
    }
    
    fn shader_highlight<'a>(ui: &Ui, string: &str, wrap_width: f32)
            -> Arc<Galley> {

        let mut job = LayoutJob::default();

        let mut text = string;

        let normal = TextFormat::default();
        let mut tp = normal.clone();
        tp.color = Color32::LIGHT_BLUE;

        while !text.is_empty() {

            let mut skip = 1;
            let mut format = &normal;

            if text.starts_with("vec3<f32>") {
                skip = 9;
                format = &tp;
            }

            

            job.append(&text[..skip], 0.0, format.clone());
            text = &text[skip..];
        }

        job.wrap.max_width = wrap_width;
        ui.fonts().layout_job(job)
    }

    fn draw_main_window(&mut self, ctx : &Context, assets : &AssetServer) {
        egui::Window::new("Pipeline editor").show(ctx, |ui| {
                    
            ui.horizontal(|ui| {
                if ui.button("Select shader").clicked() {
                    self.select_file = true;
                    self.shader_paths = assets.get_files_by_ext_from_folder("shaders".into(),"wgsl".into());
                }
                if ui.button("Close").clicked() {
                    self.show = false;
                }
            });
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut layouter = |ui : &Ui, text : &str, wrap| {
                    PipelineEditor::shader_highlight(&ui, text, wrap)
                };

                let output = egui::text_edit::TextEdit::multiline(
                    &mut self.shader_text).code_editor()
                    .layouter(&mut layouter).show(ui);
            });
            
        });
    }

    pub fn draw_winow(&mut self, ctx : &Context, assets : &AssetServer) {
        if self.show {
            if self.select_file {
                self.popup_select_file(ctx, assets);
            } else {
                self.draw_main_window(ctx, assets);
            }
        }
    }
}