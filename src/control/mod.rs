use std::hash::Hash;

use bevy::{prelude::*, utils::{HashMap, hashbrown::hash_map::Keys}};
use bevy_egui::*;
use bevy_inspector_egui::egui::Key;
use ron::ser::PrettyConfig;

pub trait IAction {
    fn get_area(&self) -> String;

    fn all_actions() -> Vec<Self> where Self : Sized;
}


#[derive(Resource)]
pub struct IKeyMapper<Action : IAction> {
    pub key_map : HashMap<Action, Option<KeyCode>>
}

impl<Action : IAction + Copy> IKeyMapper<Action> {
    fn into_ser(&self) -> IKeyMapperSer<Action> {
        IKeyMapperSer { 
            key_map: self.key_map.iter().map(|(a, k)| {
                if let Some(k) = k {
                    (*a, Some(*k as u32))
                } else {
                    (*a, None)
                }
            }).collect()
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct IKeyMapperSer<Action : IAction> {
    pub key_map : Vec<(Action, Option<u32>)>
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, serde::Serialize, serde::Deserialize)]
pub enum Action {
   FPS(FPSAction)
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, serde::Serialize, serde::Deserialize)]
pub enum FPSAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Interact
}

impl FPSAction {
    fn all_actions() -> Vec<FPSAction> {
        vec![
            FPSAction::MoveForward,
            FPSAction::MoveBackward,
            FPSAction::MoveLeft,
            FPSAction::MoveRight,
            FPSAction::Interact
        ]
    }
}

pub type KeyMapper = IKeyMapper<Action>;

impl Default for KeyMapper {
    fn default() -> Self {
        KeyMapper {
            key_map: HashMap::new(),
        }
    }
}

impl IAction for Action {
    fn get_area(&self) -> String {
        let res = match &self {
            Action::FPS(_) => "FPS",
        };
        res.to_string()
    }

    fn all_actions() -> Vec<Self> {
        let mut res = vec![];
        res.extend(FPSAction::all_actions().iter().map(|a| {
            Action::FPS(a.clone())
        }));
        res
    }
}

#[derive(Resource)]
pub struct KeyMapperWindow {
    pub is_shown : bool,
    pub actions : Vec<Action>,
    pub listen_action : Option<Action>
}

pub struct SpaceControlPlugin;


impl Plugin for SpaceControlPlugin {
    fn build(&self, app : &mut App) {
        app.insert_resource(Input::<Action>::default());
        app.init_resource::<KeyMapper>();

        let window = KeyMapperWindow {
            is_shown: true,
            actions: Action::all_actions(),
            listen_action : None
        };
        app.insert_resource(window);

        app.add_system_to_stage(CoreStage::PreUpdate, remap_system);
        app.add_system(key_mapper_window);
    }
}

fn get_keys() -> Vec<KeyCode> {
    vec![
        KeyCode::Q,
        KeyCode::W,
        KeyCode::E,
        KeyCode::R,
        KeyCode::T, 
        KeyCode::U,
        KeyCode::I, 
        KeyCode::O,
        KeyCode::P,
        KeyCode::A,
        KeyCode::S,
        KeyCode::D,
        KeyCode::F,
        KeyCode::G,
        KeyCode::H,
        KeyCode::J,
        KeyCode::K,
        KeyCode::L,
        KeyCode::Z,
        KeyCode::X,
        KeyCode::C,
        KeyCode::V,
        KeyCode::B,
        KeyCode::N,
        KeyCode::M,
    ]
}

fn key_mapper_window(
    mut ctx : ResMut<EguiContext>,
    mut window : ResMut<KeyMapperWindow>,
    mut key_mapper : ResMut<KeyMapper>,
    mut key_input : ResMut<Input<KeyCode>>
) {
    if window.is_shown {

        if let Some(action) = window.listen_action.clone() {
            for key in get_keys() {
                if key_input.just_pressed(key) {
                    key_mapper.key_map.insert(action, Some(key));
                    window.listen_action = None;
                }
            }
        }

        bevy_egui::egui::Window::new("Key mapper")
            .show(ctx.ctx_mut(), |ui| {
                
                egui::Grid::new("key mapper grid").show(ui, |ui| {
                    for action in &window.actions.clone() {
                        let text = format!("{:?}", action);
                        ui.label(&text);

                        let selected_text = {
                            if let Some(val) = key_mapper.key_map.get(action) {
                                format!("{:?}", val)
                            } else {
                                key_mapper.key_map.insert(*action, None);
                                "None".to_string()
                            }
                        };
                        if Some(*action) != window.listen_action {
                            if ui.button(selected_text).clicked() {
                                window.listen_action = Some(*action);
                            }
                        } else {
                            ui.button("Press any key");
                        }

                        ui.end_row();
                    }
                });
                
                ui.horizontal(|ui| {

                    if ui.button("Save").clicked() {
                        let ser = key_mapper.into_ser();
                        let data = ron::ser::to_string_pretty(&ser, PrettyConfig::default()).unwrap();
                        std::fs::write("key_mapping.ron", data);
                    }

                    if ui.button("Close").clicked() {
                        window.is_shown = false;
                    }
                });

                
            });
    }
}


fn remap_system(
    mut key_mapper : ResMut<KeyMapper>,
    mut input : ResMut<Input<Action>>,
    mut key_input : ResMut<Input<KeyCode>>
) {
    input.clear();
    for (action, key) in &key_mapper.key_map {
        if let Some(key) = key {
            if key_input.pressed(*key) {
                input.press(*action);
            }
        }
    }
}
