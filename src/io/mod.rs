pub mod image;

pub struct AssetLoader {
    base_path : String,
}

impl AssetLoader {
    pub fn new(base_path : &str) -> Self {
        Self {
            base_path : base_path.to_owned(),
        }
    }

    pub fn get_real_path(&self, path : &str) -> String {
        let mut res = self.base_path.clone();
        res.push_str(path);
        res
    }
}