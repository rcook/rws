// The MIT License (MIT)
//
// Copyright (c) 2020-3 Richard Cook
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of
// this software and associated documentation files (the "Software"), to deal in
// the Software without restriction, including without limitation the rights to
// use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
// the Software, and to permit persons to whom the Software is furnished to do so,
// subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
// FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
// COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
// IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
// CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
//
use anyhow::{anyhow, bail, Result};
use std::path::PathBuf;
use which::{which, Error};

pub struct GitInfo {
    pub executable_path: PathBuf,
    pub version: String,
}

impl GitInfo {
    pub fn from_environment() -> Result<GitInfo> {
        let executable_path = which("git").map_err(|e| match e {
            Error::CannotFindBinaryPath => anyhow!("Cannot locate Git executable"),
            _ => anyhow!("which failed: {}", e),
        })?;

        let output = std::process::Command::new(&executable_path)
            .arg("--version")
            .output()?;
        let parts = std::str::from_utf8(&output.stdout)?
            .split_whitespace()
            .collect::<Vec<_>>();
        if parts.len() != 3 || parts[0] != "git" || parts[1] != "version" {
            bail!("Git version output was invalid");
        }

        let version = parts[2].to_string();

        Ok(GitInfo {
            executable_path,
            version,
        })
    }
}
