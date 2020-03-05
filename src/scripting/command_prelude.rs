use rlua::prelude::{LuaError, LuaResult};
use std::fs::{read_to_string, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub fn current_dir() -> LuaResult<String> {
    Ok(std::env::current_dir()
        .unwrap()
        .to_str()
        .unwrap()
        .to_string())
}

pub fn is_file(path: String) -> LuaResult<bool> {
    Ok(Path::new(&path).is_file())
}

pub fn read_file(path: String) -> LuaResult<String> {
    read_to_string(&path).map_err(|_| LuaError::RuntimeError(format!("Failed to read {}", path)))
}

pub fn read_file_lines(path: String) -> LuaResult<Vec<String>> {
    let f = File::open(&path)
        .map_err(|_| LuaError::RuntimeError(format!("Failed to open {}", path)))?;
    BufReader::new(f)
        .lines()
        .collect::<std::io::Result<_>>()
        .map_err(|_| LuaError::RuntimeError(format!("Failed while reading from {}", path)))
}
