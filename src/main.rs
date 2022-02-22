use rlua::{Lua};

extern "C" {
    pub fn square(value : i32) -> i32;
}

fn main() {

    //let output = Command::new("cmd").spawn().unwrap();
    // println!("{:?}", output.);

unsafe {
    println!("Square: {}", square(11));
}
    

    let lua = Lua::new();

    lua.context(|lua_ctx| {
        let text = std::fs::read_to_string("test_script.lua").unwrap();
        lua_ctx.load(text.as_str()).exec().unwrap();
    });
}