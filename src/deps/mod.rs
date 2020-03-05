use crate::os::with_working_dir;
use crate::scripting::command::Command;

use std::path::Path;

pub fn get_deps(project_dir: &Path, command: &Command) -> Option<Vec<String>> {
    with_working_dir(project_dir, || command.eval().as_str_vec())
}
