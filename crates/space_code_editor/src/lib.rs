use std::ops::Range;
use std::sync::Arc;

use regex::Regex;
use egui::*;
use egui::text::LayoutJob;

use space_assets::*;


#[derive(Clone)]
struct Token {
    range : Range<usize>,
    tp : TokenType,
    text : String
}

#[derive(Clone)]
enum TokenType {
    Type,
    Variable,
    Function,
    StructDefine,
    Undefined
}

#[derive(Clone)]
struct TokenParser {
    rg : Regex,
    tp : TokenType,
    match_idx : usize
}

#[derive(Clone)]
struct GroupedToken {
    tp : TokenType,
    rg : Vec<Regex>
}

#[derive(Clone)]
struct LanguageDefine {
    grouped_tokens : Vec<GroupedToken>,
    tokenizer : Regex
    // var_regexes : Vec<Regex>,
    // tp_var_regexes : Vec<Regex>,
    // struct_regexes : Vec<Regex>,
    // fn_regexes : Vec<Regex>
}

impl LanguageDefine {
    fn new_wgsl() -> Self {
        let mut type_regexes = vec![
            TokenParser {
                tp : TokenType::Type,
                rg : Regex::new("f32").unwrap(),
                match_idx : 0
            },
            TokenParser {
                tp : TokenType::Type,
                rg : Regex::new(r"vec\d<f32>").unwrap(),
                match_idx : 0
            },
            TokenParser {
                tp : TokenType::Type,
                rg : Regex::new(r"mat\dx\d<f32>").unwrap(),
                match_idx : 0
            }
        ];

        let var_regexes = vec![
            Regex::new(r"(var)\s(\w+)").unwrap()
        ];

        let tp_var_regexes = vec![
            Regex::new(r"(var)\s(\w+)\s*:\s*(.+);").unwrap(),
            Regex::new(r"(var<\w+>)\s*(\w+)\s*:\s*(\w+);").unwrap()
        ];

        let struct_regexes = vec![
            Regex::new(r"(struct)\s+(\w+)").unwrap()
        ];

        let mut grouped_tokens = vec![];

        let fn_names = vec![
            "max",
            "dot",
            "pow",
            "cross",
            "textureSample",
            "mix",
            "length",
            "normalize",
        ];

        for f in fn_names {
            grouped_tokens.push(GroupedToken {
                rg: vec![Regex::new(format!(r"{}", f).as_str()).unwrap()],
                tp: TokenType::Function,
            });
        }

        let fn_regexes = vec![
            Regex::new(r"fn\s+(\w+)\s*\(").unwrap()
        ];

        Self { 
            grouped_tokens, 
            tokenizer : Regex::new(r"\w+|:|\(|\)|{|}|\.|\,|(@\w+)|<|>|;|=|\*|\\|\+|\/").unwrap()
            // var_regexes, 
            // tp_var_regexes, 
            // struct_regexes,
            // fn_regexes
        }
    }
}

struct WgslHighlight {
    code : String,
    cached_jobs : LayoutJob,
    language : LanguageDefine
}

impl Default for WgslHighlight {
    fn default() -> Self {
        Self {
            code : String::default(),
            cached_jobs : LayoutJob::default(),
            language : LanguageDefine::new_wgsl()
        }
    }
}

impl WgslHighlight {

    fn tokenize(code : &str, lng : &LanguageDefine) -> Vec<Token> {
        lng.tokenizer.captures_iter(code).filter(|cap| {
            cap.get(0).is_some()
        }).map(|cap| {
            let mat = cap.get(0).unwrap();
            let text = code[mat.range().start..mat.range().end].to_string();
            Token { range: mat.range(), tp: TokenType::Undefined, text }
        }).collect()
    }

    fn get_tokens(&self, tokens : &mut Vec<Token>, lng : &mut LanguageDefine) {

        // for (idx, token) in tokens.iter_mut().enumerate() {
        //     for group in &lng.grouped_tokens {
        //         if to
        //     }
        // }

        // let mut ranges = vec![];
        // for parser in &lng.type_regexes {
        //     for cap in parser.rg.captures_iter(line) {
        //         if let Some(mat) = cap.get(parser.match_idx) {
        //             ranges.push( Token {
        //                 range : mat.range(),
        //                 tp : parser.tp.clone(),
        //             });
        //         }
        //     }
        // }

        // for parser in &lng.var_regexes {
        //     for cap in parser.captures_iter(line) {
        //         if let Some(tp_mat) = cap.get(1) {
        //             if let Some(var_mat) = cap.get(2) {
        //                 ranges.push(Token {
        //                     range : tp_mat.range(),
        //                     tp : TokenType::Type
        //                 });
        //                 ranges.push(Token {
        //                     range : var_mat.range(),
        //                     tp : TokenType::Variable
        //                 });
        //             }
        //         }
        //     }
        // }

        // for parser in &lng.tp_var_regexes {
        //     for cap in parser.captures_iter(line) {
        //         if let Some(var_word_mat) = cap.get(1) {
        //             if let Some(var_mat) = cap.get(2) {
        //                 if let Some(tp_mat) = cap.get(3) {
        //                     ranges.push(Token {
        //                         range: var_word_mat.range(),
        //                         tp: TokenType::Type
        //                     });
        //                     ranges.push(Token {
        //                         range: var_mat.range(),
        //                         tp: TokenType::Variable
        //                     });
        //                     ranges.push(Token {
        //                         range: tp_mat.range(),
        //                         tp: TokenType::Type
        //                     });
        //                 }
        //             }
        //         }
        //     }
        // }

        // for parser in &lng.struct_regexes {
        //     for cap in parser.captures_iter(line) {
        //         if let Some(str_mat) = cap.get(1) {
        //             if let Some(struct_name_mat) = cap.get(2) {
        //                 ranges.push(Token {
        //                     range: str_mat.range(),
        //                     tp: TokenType::Type
        //                 });
        //                 ranges.push(Token {
        //                     range: struct_name_mat.range(),
        //                     tp: TokenType::StructDefine
        //                 });
        //                 lng.type_regexes.push(TokenParser {
        //                     rg: Regex::new(
        //                         &line[struct_name_mat.range().start..struct_name_mat.range().end]).unwrap(),
        //                     tp: TokenType::Type,
        //                     match_idx : 0
        //                 });
        //             }
        //         }
        //     }
        // }

        // let mut mark_vec = vec![false; ranges.len()];
        // let mut marked = 1;
        // while marked > 0 {
        //     marked = 0;

        //     for idx in  0..ranges.len() {
        //         let range = &ranges[idx];
        //         if mark_vec[idx] {
        //             continue;
        //         }
        //         for j in 0..ranges.len() {
        //             if j != idx {
        //                 let sub_range = &ranges[j];
        //                 if range.range.start <= sub_range.range.start &&
        //                     range.range.end >= sub_range.range.end {
        //                     marked += 1;
        //                     mark_vec[j] = true;
        //                 }
        //             }
        //         }
        //     }

        //     ranges = ranges.iter().enumerate().filter(|(idx, range)| {
        //         !mark_vec[*idx]
        //     }).map(|(idx, range)| {range.clone()}).collect();
        //     mark_vec.fill(false);
        // }

        // ranges.sort_by(|a, b| {
        //     a.range.start.cmp(&b.range.start)
        // });

        // ranges
    }

    fn highlight(&mut self, ui: &Ui, code: &str, wrap_width: f32) -> LayoutJob {
        
        // let mut lng = self.language.clone();
        //
        // if self.code == code {
        //     return self.cached_jobs.clone();
        // }
        //
        let mut job = LayoutJob::default();
        //
        // let normal_format = TextFormat::default();
        // let mut tp_format = normal_format.clone();
        // tp_format.color =  Color32::from_rgb(156, 220, 254);
        // let mut var_format = normal_format.clone();
        // var_format.color = Color32::LIGHT_BLUE;
        // let mut fn_format = normal_format.clone();
        // fn_format.color = Color32::from_rgb(209, 105, 209);
        // let mut struct_format = normal_format.clone();
        // struct_format.color = Color32::from_rgb(62, 214, 194);
        //
        // for line in code.split('\n') {
        //     let mut idx = 0;
        //     let ranges = self.get_tokens(line, &mut lng);
        //     while idx < line.len() {
        //         let mut skip = 1;
        //         let mut tp = TokenType::Undefined;
        //
        //
        //         for range in &ranges {
        //             if idx == range.range.start {
        //                 skip = range.range.len();
        //                 tp = range.tp.clone();
        //             }
        //         }
        //
        //         let format = match tp {
        //             TokenType::Type => {&tp_format}
        //             TokenType::Variable => {&var_format}
        //             TokenType::Function => {&fn_format}
        //             TokenType::Undefined => {&normal_format}
        //             TokenType::StructDefine => {&struct_format},
        //         };
        //
        //         job.append(&line[idx..(idx + skip)], 0.0, format.clone());
        //         idx += skip;
        //     }
        //
        //     job.append("\n", 0.0, normal_format.clone());
        // }
        //
        // self.code = code.to_string();
        // self.cached_jobs = job.clone();
        //
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

    fn popup_select_file(&mut self, ctx : &Context, assets : &SpaceAssetServer) {
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

    fn draw_main_window(&mut self, ctx : &Context, assets : &SpaceAssetServer) {

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

    pub fn draw_winow(&mut self, ctx : &Context, assets : &SpaceAssetServer) {
        if self.show {
            if self.select_file {
                self.popup_select_file(ctx, assets);
            } else {
                self.draw_main_window(ctx, assets);
            }
        }
    }
}