use std::env::{current_dir, set_current_dir};
use std::path::{Component, Path};

pub fn with_working_dir<F, R>(dir: &Path, f: F) -> std::io::Result<R>
where
    F: FnOnce() -> R,
{
    let saved_dir = current_dir()?;
    set_current_dir(dir)?;
    let result = f();
    set_current_dir(saved_dir)?;
    Ok(result)
}

pub fn path_to_str(path: &Path) -> &str {
    path.to_str()
        .expect("Path contains invalid Unicode characters")
}

pub fn get_base_name(path: &Path) -> Option<&str> {
    path.components().last().and_then(|x| {
        if let Component::Normal(s) = x {
            s.to_str()
        } else {
            None
        }
    })
}
