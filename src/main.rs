use std::str::FromStr;

use engine;
use engine::resource::*;

fn main() {
    let mut res_system = engine::resource::FileResourceEngine::default();
    let path = String::from_str("./res").unwrap();
    res_system.init(&path);
}