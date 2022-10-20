use std::ops::Range;
use std::sync::Arc;

use regex::Regex;
use egui::*;
use egui::text::LayoutJob;

use space_assets::*;


#[derive(Clone)]
struct Token {
    range : Range<usize>,
    tp : TokenType
}

#[derive(Clone)]
enum TokenType {
    Type,
    Variable,
    Function,
    Undefined
}

struct TokenParser {
    rg : Regex,
    tp : TokenType
}

struct WgslHighlight {
    code : String,
    cached_jobs : LayoutJob,
    type_regexes : Vec<TokenParser>,
    var_regexes : Vec<Regex>,
    tp_var_regexes : Vec<Regex>
}

impl Default for WgslHighlight {
    fn default() -> Self {

        let type_regexes = vec![
            TokenParser {
                tp : TokenType::Type,
                rg : Regex::new("f32").unwrap()
            },
            TokenParser {
                tp : TokenType::Type,
                rg : Regex::new(r"vec\d<f32>").unwrap()
            }
        ];

        let var_regexes = vec![
            Regex::new(r"(var)\s(\w+)").unwrap()
        ];

        let tp_var_regexes = vec![
            Regex::new(r"(var)\s(\w+)\s*:\s*(.+);").unwrap()
        ];

        Self {
            code : String::default(),
            cached_jobs : LayoutJob::default(),
            type_regexes,
            var_regexes,
            tp_var_regexes
        }
    }
}

impl WgslHighlight {

    fn get_tokens(&self, line : &str) -> Vec<Token> {
        let mut ranges = vec![];
        for parser in &self.type_regexes {
            for cap in parser.rg.captures_iter(line) {
                if let Some(mat) = cap.get(0) {
                    ranges.push( Token {
                        range : mat.range(),
                        tp : parser.tp.clone()
                    });
                }
            }
        }

        for parser in &self.var_regexes {
            for cap in parser.captures_iter(line) {
                if let Some(tp_mat) = cap.get(1) {
                    if let Some(var_mat) = cap.get(2) {
                        ranges.push(Token {
                            range : tp_mat.range(),
                            tp : TokenType::Type
                        });
                        ranges.push(Token {
                            range : var_mat.range(),
                            tp : TokenType::Variable
                        });
                    }
                }
            }
        }

        for parser in &self.tp_var_regexes {
            for cap in parser.captures_iter(line) {
                if let Some(var_word_mat) = cap.get(1) {
                    if let Some(var_mat) = cap.get(2) {
                        if let Some(tp_mat) = cap.get(3) {
                            ranges.push(Token {
                                range: var_word_mat.range(),
                                tp: TokenType::Type
                            });
                            ranges.push(Token {
                                range: var_mat.range(),
                                tp: TokenType::Variable
                            });
                            ranges.push(Token {
                                range: tp_mat.range(),
                                tp: TokenType::Type
                            });
                        }
                    }
                }
            }
        }

        let mut mark_vec = vec![false; ranges.len()];
        let mut marked = 1;
        while marked > 0 {
            marked = 0;

            for idx in  0..ranges.len() {
                let range = &ranges[idx];
                if mark_vec[idx] {
                    continue;
                }
                for j in 0..ranges.len() {
                    if j != idx {
                        let sub_range = &ranges[j];
                        if range.range.start <= sub_range.range.start &&
                            range.range.end >= sub_range.range.end {
                            marked += 1;
                            mark_vec[j] = true;
                        }
                    }
                }
            }

            ranges = ranges.iter().enumerate().filter(|(idx, range)| {
                !mark_vec[*idx]
            }).map(|(idx, range)| {range.clone()}).collect();
            mark_vec.fill(false);
        }

        ranges.sort_by(|a, b| {
            a.range.start.cmp(&b.range.start)
        });

        ranges
    }

    fn highlight(&mut self, ui: &Ui, code: &str, wrap_width: f32) -> LayoutJob {
        if self.code == code {
            return self.cached_jobs.clone();
        }

        let mut job = LayoutJob::default();

        let normal_format = TextFormat::default();
        let mut tp_format = normal_format.clone();
        tp_format.color =  Color32::from_rgb(156, 220, 254);
        let mut var_format = normal_format.clone();
        var_format.color = Color32::LIGHT_BLUE;

        for line in code.split('\n') {

            
            let mut idx = 0;
            while idx < line.len() {
                let mut skip = 1;
                let mut tp = TokenType::Undefined;

                let ranges = self.get_tokens(line);

                for range in &ranges {
                    if idx == range.range.start {
                        skip = range.range.len();
                        tp = range.tp.clone();
                    }
                }

                let format = match tp {
                    TokenType::Type => {&tp_format}
                    TokenType::Variable => {&var_format}
                    TokenType::Function => {&tp_format}
                    TokenType::Undefined => {&normal_format}
                };

                job.append(&line[idx..(idx + skip)], 0.0, format.clone());
                idx += skip;
            }

            job.append("\n", 0.0, normal_format.clone());
        }

        self.code = code.to_string();
        self.cached_jobs = job.clone();

        job
    }
}

pub struct WgslEditor {
    pub shader_text : String,
    pub shader_path : AssetPath,
    pub show : bool,
    pub select_file : bool,
    pub shader_paths : Vec<String>,
    parser : WgslHighlight
}

impl Default for WgslEditor {
    fn default() -> Self {
        WgslEditor {
            shader_text: "".into(),
            show: false,
            shader_path: AssetPath::Text("".into()),
            select_file: false,
            shader_paths : vec![],
            parser : WgslHighlight::default()
        }
    }
}

impl WgslEditor {

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

        let Self {
            parser, ..
        } = self;

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
                    let mut job = parser.highlight(ui, text, wrap);

                    job.wrap.max_width = wrap;
                    ui.fonts().layout_job(job)
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