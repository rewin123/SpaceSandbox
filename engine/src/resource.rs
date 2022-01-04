use std::{fs, str::FromStr};
use serde_json;

pub trait ResourceEngine {
    fn default() -> Self;
    fn init(&mut self, path : &String);
    fn get_json(&self, name : &String) -> Option<String>;
    fn get_data_string(&self, name : &String) -> Option<String>;

    fn get_wgsl_shader(&self, name : &String) -> Result<wgpu::ShaderSource, String> {

        match self.get_data_string(name) {
            Some(data) => {
                let shader = wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(data));
                return Ok(shader);
            }
            None => {
                return Err(String::from("Shader source not found"));
            }
        }
    }

    fn get_wgsl_shader_templated(&self, name : &String) -> Result<wgpu::ShaderSource, String> {

        match self.get_data_string(name) {
            Some(data) => {
                let shader = wgpu::ShaderSource::Wgsl(std::borrow::Cow::Owned(data));
                return Ok(shader);
            }
            None => {
                return Err(String::from("Shader source not found"));
            }
        }
    }
}

pub trait ResourceBase {
    fn default() -> Self;
    fn get_name(&self) -> String;
    fn get_json(&self) -> String;
}

#[derive(Debug)]
struct FileResourceBase {
    name : String,
    json : String,
    binary_local_path : String,
    binary_path : String
}

impl ResourceBase for FileResourceBase {
    fn default() -> Self {
        Self {
            name : String::default(),
            json : String::default(),
            binary_local_path : String::default(),
            binary_path : String::default()
        }
    }
    
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn get_json(&self) -> String {
        self.json.clone()
    }
}

pub struct FileResourceEngine {
    resources : Vec<FileResourceBase>
}

struct FileOps;

impl FileOps {
    pub fn get_extension_from_filename(filename: &str) -> Option<&str> {
        std::path::Path::new(filename)
            .extension()
            .and_then(std::ffi::OsStr::to_str)
    }
}

impl FileResourceEngine {
    fn get_res_base(&self, name : &String) -> Option<&FileResourceBase> {
        for asset in &self.resources {
            if &asset.name == name {
                return Some(asset);
            }
        }
        None
    }
}

impl ResourceEngine for FileResourceEngine {
    fn default() -> Self {
        Self {
            resources : vec![]
        }
    }

    fn init(&mut self, path : &String) {
        let mut paths_vec: Vec<String> = vec![path.clone()];

        let mut i = 0;
        while i < paths_vec.len() {
            let root_path = paths_vec.get(i).unwrap().clone();
            let paths = fs::read_dir(root_path.clone()).unwrap();
            for path in paths {
                let entry = path.unwrap();
                let metadata = entry.metadata().unwrap();
                let entry_path = entry.path();
                let str_path = String::from(entry_path.to_str().unwrap());

                if metadata.is_dir() && root_path.as_str() != str_path.as_str() {
                    paths_vec.push(str_path);
                }

                if metadata.is_file() {
                    if FileOps::get_extension_from_filename(entry_path.to_str().unwrap()) == Some("json") {
                        let mut res = FileResourceBase::default();
                        let json_str = std::fs::read_to_string(entry_path.to_str().unwrap()).unwrap();
                        let res_set: serde_json::Value = serde_json::from_str(json_str.as_str()).unwrap();

                        res.json = json_str.clone();

                        match res_set["name"].clone() {
                            serde_json::Value::String(v) => {
                                res.name = v;
                            }
                            _ => {}
                        }
                        match res_set["local_path"].clone() {
                            serde_json::Value::String(v) => {
                                res.binary_local_path = v;
                            }
                            _ => {}
                        }

                        res.binary_path = root_path.clone() + "/" + res.binary_local_path.clone().as_str();
                        println!("New res: {:?}", res);

                        self.resources.push(res);
                    }
                }
            }

            i += 1;
        }
    }

    

    
    fn get_json(&self, name : &String) -> Option<String> {
        for asset in &self.resources {
            if &asset.name == name {
                return Some(asset.json.clone());
            }
        }
        None
    }

    fn get_data_string(&self, name : &String) -> Option<String> {
        match self.get_res_base(name) {
            Some(res_base) => {
                let string_data = std::fs::read_to_string(res_base.binary_path.as_str()).unwrap();
                return Some(string_data);
            }
            None => {
                return None;
            }
        } 
    }
}


