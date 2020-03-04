use std::env::{current_dir, set_current_dir};
use std::path::Path;

pub fn with_working_dir<F, R>(dir: &Path, f: F) -> R
where
    F: FnOnce() -> R,
{
    let saved_dir = current_dir().unwrap();
    set_current_dir(dir).unwrap();
    let result = f();
    set_current_dir(saved_dir).unwrap();
    result
}
