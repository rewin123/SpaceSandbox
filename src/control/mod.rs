use std::hash::Hash;

use bevy::{prelude::*, utils::{HashMap, HashSet}};
use bevy_egui::*;
use ron::ser::PrettyConfig;


pub trait IAction {
    fn get_area(&self) -> String;

    fn all_actions() -> Vec<Self> where Self : Sized;
}


#[derive(Resource, serde::Deserialize, serde::Serialize)]
pub struct KeyMapper {
    pub key_map : HashMap<Action, Option<KeyCode>>
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, serde::Serialize, serde::Deserialize)]
pub enum Action {
   FPS(FPSAction),
   Build(BuildAction),
   Piloting(PilotingAction)
}


#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, serde::Serialize, serde::Deserialize)]
pub enum PilotingAction {
    MoveForward,
    MoveBackward,
    RollLeft,
    RollRight,
    TurnUp,
    TurnDown,
    TurnLeft,
    TurnRight,
    GoToNextCamera,
    BackToSeat
}

impl PilotingAction {
    fn all_actions() -> Vec<Self> {
        vec![
            PilotingAction::MoveForward,
            PilotingAction::MoveBackward,
            PilotingAction::RollLeft,
            PilotingAction::RollRight,
            PilotingAction::TurnUp,
            PilotingAction::TurnDown,
            PilotingAction::TurnLeft,
            PilotingAction::TurnRight,
            PilotingAction::GoToNextCamera,
            PilotingAction::BackToSeat
        ]
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, serde::Serialize, serde::Deserialize)]
pub enum FPSAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    Interact,
    Jump,
    Crouch,
    Sprint,
    Dash
}

impl FPSAction {
    fn all_actions() -> Vec<FPSAction> {
        vec![
            FPSAction::MoveForward,
            FPSAction::MoveBackward,
            FPSAction::MoveLeft,
            FPSAction::MoveRight,
            FPSAction::Interact,
            FPSAction::Jump,
            FPSAction::Crouch,
            FPSAction::Sprint,
            FPSAction::Dash
        ]
    }
}




#[derive(Debug, Clone, Hash, PartialEq, Eq, Copy, serde::Serialize, serde::Deserialize)]
pub enum BuildAction {
    MoveForward,
    MoveBackward,
    MoveRight,
    MoveLeft,
    LevelUp,
    LevelDown,
    RotateClockwise,
    RotateCounterClockwise,
}



impl BuildAction {
    fn all_actions() -> Vec<BuildAction> {
        vec![
            BuildAction::MoveForward,
            BuildAction::MoveBackward,
            BuildAction::MoveRight,
            BuildAction::MoveLeft,
            BuildAction::LevelUp,
            BuildAction::LevelDown,
            BuildAction::RotateClockwise,
            BuildAction::RotateCounterClockwise,
        ]
    }
}

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
            Action::Build(_) => "Build",
            Action::Piloting(_) => "Piloting",
        };
        res.to_string()
    }

    fn all_actions() -> Vec<Self> {
        let mut res = vec![];
        res.extend(FPSAction::all_actions().iter().map(|a| {
            Action::FPS(*a)
        }));
        res.extend(BuildAction::all_actions().iter().map(|a| {
            Action::Build(*a)
        }));
        res.extend(PilotingAction::all_actions().iter().map(|a| {
            Action::Piloting(*a)
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

#[derive(Resource)]
pub struct TwiceClick {
    action_times : HashMap<Action, f64>,
    twice_threhold : f64,
    is_twice : HashSet<Action>
}

impl TwiceClick {
    pub fn is_twice(&self, action : &Action) -> bool {
        self.is_twice.contains(action)
    }

    pub fn get_time(&self, action : &Action) -> f64 {
        *self.action_times.get(action).unwrap_or(&0.0)
    }
}

impl Default for TwiceClick {
    fn default() -> Self {
        TwiceClick {
            action_times : HashMap::new(),
            twice_threhold : 0.5,
            is_twice : HashSet::new(),
        }
    }
}

pub struct SpaceControlPlugin;


impl Plugin for SpaceControlPlugin {
    fn build(&self, app : &mut App) {
        app.insert_resource(Input::<Action>::default());

        let mut key_mapper = KeyMapper::default();

        if let Ok(data) = std::fs::read_to_string("key_mapping.ron") {
            if let Ok(ser) = ron::from_str::<KeyMapper>(&data) {
                key_mapper = ser;
            }
        }

        app.insert_resource(key_mapper);
        app.insert_resource(TwiceClick::default());

        let window = KeyMapperWindow {
            is_shown: false,
            actions: Action::all_actions(),
            listen_action : None
        };
        app.insert_resource(window);

        app.add_system(remap_system);
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
        KeyCode::Space,
        // KeyCode::LShift,
        // KeyCode::RShift,
        // KeyCode::LControl,
        // KeyCode::RControl,
        // KeyCode::LAlt,
        // KeyCode::RAlt,
        // KeyCode::LWin,
        // KeyCode::RWin,
    ]
}

fn key_mapper_window(
    mut ctx : Query<&mut EguiContext>,
    mut window : ResMut<KeyMapperWindow>,
    mut key_mapper : ResMut<KeyMapper>,
    key_input : ResMut<Input<KeyCode>>,
    input : Res<Input<Action>>
) {
    let mut ctx = ctx.single_mut();
    if window.is_shown {

        if let Some(action) = window.listen_action {
            for key in get_keys() {
                if key_input.just_pressed(key) {
                    key_mapper.key_map.insert(action, Some(key));
                    window.listen_action = None;
                }
            }
        }

        bevy_egui::egui::Window::new("Key mapper")
            .show(ctx.get_mut(), |ui| {
                
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
                            if input.pressed(*action) {
                                if ui.add(egui::Button::new(selected_text).fill(egui::Color32::GREEN)).clicked() {
                                    window.listen_action = Some(*action);
                                }
                            } else if ui.button(selected_text).clicked() {
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
                        let data = ron::ser::to_string_pretty(key_mapper.as_ref(), PrettyConfig::default()).unwrap();
                        std::fs::write("key_mapping.ron", data);
                    }

                    if ui.button("Close").clicked() {
                        window.is_shown = false;
                    }
                });

                
            });
    }
}


pub fn remap_system(
    key_mapper : ResMut<KeyMapper>,
    mut input : ResMut<Input<Action>>,
    key_input : ResMut<Input<KeyCode>>,
    mut twice_click : ResMut<TwiceClick>,
    time : Res<Time>
) {
    input.clear();
    let time_now = time.elapsed_seconds_f64();
    twice_click.is_twice.clear();
    for (action, key) in &key_mapper.key_map {
        if let Some(key) = key {
            if key_input.pressed(*key) {
                input.press(*action);
            } else {
                input.release(*action);
            }

            if key_input.just_pressed(*key) {
                let dt = time_now - twice_click.get_time(action);
                info!("dt: {} threhold: {} time_now: {}", dt, twice_click.twice_threhold, time_now);
                if dt < twice_click.twice_threhold {
                    twice_click.is_twice.insert(*action);
                }
                twice_click.action_times.insert(*action, time_now);
            }
        }
    }
}
