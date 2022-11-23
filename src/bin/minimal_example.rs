use space_game::Game;
use space_render::add_game_render_plugins;

async fn run() {
    rayon::ThreadPoolBuilder::default()
        .num_threads(3)
        .build_global().unwrap();
    let mut game = Game::default();
    add_game_render_plugins(&mut game);
    game.update_scene_scheldue();
    game.run();
}

fn main() {
    pollster::block_on(run());
}