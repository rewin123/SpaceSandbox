use bevy::prelude::*;
use iyes_loopless::prelude::*;
use bevy_egui::*;
use crate::*;

fn main_menu(
    mut cmds : Commands,
    mut egui_context: ResMut<EguiContext>,
    mut scene : ResMut<CurrentState<SceneType>>
) {
    egui::Window::new("Space sandbox")
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .show(egui_context.ctx_mut(), |ui| {
            ui.vertical_centered(|ui| {
                if ui.button("New station").clicked() {
                    cmds.insert_resource(NextState(SceneType::ShipBuilding));
                }
                ui.button("Load station");
                ui.button("Connect to server");
                if ui.button("Exit").clicked() {
                    // cmds.push(GameCommands::Exit);
                }
            });
    });
}

// fn ui_example(mut egui_context: ResMut<EguiContext>) {
//     egui::Window::new("Hello").show(egui_context.ctx_mut(), |ui| {
//         ui.label("world");
//     });
// }

pub struct MainMenuPlugin {

}

impl Plugin for MainMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_loopless_state(SceneType::MainMenu);
        
        app.add_system_set(ConditionSet::new()
            .run_in_state(SceneType::MainMenu)
            .with_system(main_menu)
            .into());
    }
}