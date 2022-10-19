use egui::Ui;
use space_assets::*;

pub struct SelectGltfWindow {
    files : Vec<String>
}

impl SelectGltfWindow {
    pub fn new(assets : &AssetServer) -> Self {
        Self {
            files : assets.get_files_by_ext("gltf".to_string())
        }
    }

    // pub fn draw(&self, ui : &mut Ui, assets : &mut AssetServer, game : &mut Game) -> bool {
    //     for file in &self.files {
    //         if ui.button(file).clicked() {
    //             assets.load_static_gltf(game, file.to_string());
    //             return true;
    //         }
    //     }
    //     false
    // }
}