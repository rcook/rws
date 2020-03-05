use rlua::prelude::LuaResult;
use std::path::Path;

pub fn is_file(path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).is_file())
}

pub fn current_dir() -> LuaResult<String> {
    Ok(std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string())
}

pub fn greet(name: String) -> LuaResult<()> {
    println!("Hello {}!", name);
    Ok(())
}
