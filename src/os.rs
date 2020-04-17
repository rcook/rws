use std::env::{current_dir, set_current_dir};
use std::io::{Error, ErrorKind};
use std::path::{Component, Path, PathBuf, Prefix};

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

#[cfg(target_os = "windows")]
pub fn get_absolute_path(path: &Path) -> std::io::Result<PathBuf> {
    let p = path.canonicalize()?;

    let is_verbatim_disc = match p.components().next() {
        Some(Component::Prefix(prefix_component)) => match prefix_component.kind() {
            Prefix::VerbatimDisk(_) => true,
            _ => false,
        },
        _ => false,
    };
    if !is_verbatim_disc {
        return Err(Error::new(
            ErrorKind::Other,
            format!("Could not canonicalize path {}", path.display()),
        ));
    }

    const VERBATIM_PREFIX: &str = r#"\\?\"#;
    let temp = p.display().to_string();
    assert!(temp.starts_with(VERBATIM_PREFIX));
    Ok(Path::new(&temp[VERBATIM_PREFIX.len()..]).to_path_buf())
}

#[cfg(not(target_os = "windows"))]
pub fn get_absolute_path(path: &Path) -> std::io::Result<PathBuf> {
    path.canonicalize()
}
