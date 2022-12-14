use space_core::app::App;
use space_game::{Game, GamePlugins};
use space_game::SceneType::MainMenu;
use space_render::add_game_render_plugins;
use SpaceSandbox::scenes::StationBuildMenu;

use SpaceSandbox::ui::*;

async fn run() {
    let app = App::default()
        .add_plugins(bevy::DefaultPlugins)
        .add_plugins(GamePlugins{})
        .add_plugin(MainMenuPlugin{})
        .run();
}

fn main() {
    pollster::block_on(run());
}