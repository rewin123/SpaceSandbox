use space_game::Game;
use space_render::add_game_render_plugins;

pub fn setup_station_build_scene(game : &mut Game) {
    game.clear_plugins();
    add_game_render_plugins(game);
    game.update_scene_scheldue();
}