use space_game::Game;
use space_render::add_game_render_plugins;
use SpaceSandbox::init_logger;

async fn run() {
    init_logger();
    rayon::ThreadPoolBuilder::default()
        .num_threads(3)
        .build_global().unwrap();
    let mut game = Game::default();
    add_game_render_plugins(&mut game).await;
    game.update_scene_scheldue();
    game.run();
}

fn main() {
    pollster::block_on(run());
}