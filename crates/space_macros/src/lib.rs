extern crate proc_macro;
use proc_macro::TokenStream;
use syn::{parse_macro_input};

#[proc_macro]
pub fn make_answer(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
}


#[proc_macro]
pub fn unifrom_struct(item : TokenStream) -> TokenStream {
    let mut result = String::new();

    let mut include_name = String::new();
    let mut shader_struct = String::new();

    let plain_text = item.to_string();

    result += "#[derive(ShaderType)]\n";

    let mut first_token = true;

    let mut shader_struct_body = "".to_string();

    for token in plain_text.split(',') {
        if first_token {
            first_token = false;
            let name = token.to_string();
            result += format!("pub struct {} {{\n", token).as_str();

            //shader part
            include_name = name.trim().to_string();

            shader_struct += format!("struct {} {{", include_name).as_str();

        } else {
            let names : Vec<String> = token.split(':').map(|s| s.to_string()).collect();
            let mut line = String::new();
            line += "    pub ";
            line += names[0].trim();
            line += " : ";

            let cls = names[1].trim();

            if cls == "vec3" {
                line += "nalgebra::Vector3<f32>,";
            } else if cls == "f32" {
                line += "f32,";
            } else if cls == "mat4" {
                line += "nalgebra::Matrix4<f32>,";
            }
            line += "\n";

            result += line.as_str();

            //shader part
            let mut line = String::new();
            line += names[0].trim();
            line += " : ";

            if cls == "vec3" {
                line += "vec3<f32>";
            } else if cls == "f32" {
                line += "f32";
            } else if cls == "mat4" {
                line += "mat4x4<f32>";
            }

            if shader_struct_body.len() > 0 {
                shader_struct_body = format!("{}, {}", shader_struct_body, line);
            } else {
                shader_struct_body = line;
            }
        }
    }

    shader_struct += shader_struct_body.as_str();

    result += "}\n";

    shader_struct += "}\n";


    result += "\n";
    //implement shader uniform trait
    result += format!("impl ShaderUniform for {} {{\n", include_name).as_str();

    result += "fn get_name(&self) -> String {\n";
    result += format!("\"{}\".to_string()\n", include_name).as_str();
    result += "}\n";

    result += "fn get_struct(&self) -> String {\n";
    result += format!("\"{}\".to_string()\n", shader_struct.replace("\n", "")).as_str();
    result += "}\n";

    result += "}";

    //implement trait part

    // result = format!("fn answer() {{ log::info!(\"{}\"); }}", result
    //     .replace("{", "{{")
    //     .replace("}", "}}")
    //     .replace("\"", "\\\"")).parse().unwrap();

    result.parse().unwrap()
}