use crate::error::{user_error_result, Result};

use std::path::PathBuf;
use which::which;

pub struct GitInfo {
    pub executable_path: PathBuf,
    pub version: String,
}

impl GitInfo {
    pub fn from_environment() -> Result<GitInfo> {
        let executable_path = which("git")?;

        let output = std::process::Command::new(&executable_path)
            .arg("--version")
            .output()?;
        let parts = std::str::from_utf8(&output.stdout)?
            .trim()
            .split_whitespace()
            .collect::<Vec<_>>();
        if parts.len() != 3 || parts[0] != "git" || parts[1] != "version" {
            return user_error_result("Git version output was invalid");
        }

        let version = parts[2].to_string();

        Ok(GitInfo {
            executable_path: executable_path,
            version: version,
        })
    }
}
