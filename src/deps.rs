use crate::error::{user_error, Result};
use crate::os::with_working_dir;
use crate::scripting::command::Command;

use std::path::Path;

pub fn get_deps(project_dir: &Path, command: &Command) -> Result<Vec<String>> {
    with_working_dir(project_dir, || command.eval())?
        .as_str_vec()
        .ok_or_else(|| user_error("Invalid string vector"))
}
