

struct AssetServer {
    root_path : String
}

impl Default for AssetServer {
    fn default() -> Self {
        Self {
            root_path : "res".to_string()
        }
    }
}