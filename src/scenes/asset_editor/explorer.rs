use std::collections::HashMap;

use std::path::{PathBuf};
use bevy_egui::egui::{Ui};
use bevy::prelude::*;


#[derive(Component)]
pub struct AssetExplorer {
    current_dir: PathBuf,
    entry_cache: HashMap<PathBuf, Vec<PathBuf>>,
    selected_file: Option<PathBuf>,
}

impl AssetExplorer {
    pub fn new(initial_dir: PathBuf) -> Self {
        Self {
            current_dir: initial_dir,
            entry_cache: HashMap::new(),
            selected_file: None,
        }
    }

    pub fn title(&self) -> String {
        format!("Asset Explorer - {:?}", self.current_dir)
    }

    pub fn show(&mut self, _ui: &mut Ui) {
        
    }
}