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
use crate::config::Definition;
use anyhow::{anyhow, bail, Result};
use joatmon::{find_sentinel_file, read_yaml_file};
use std::env;
use std::path::{Path, PathBuf};

const WORKSPACE_CONFIG_FILE_NAME: &str = "rws-workspace.yaml";

/// Workspace information derived from file system and configuration file
#[derive(Debug)]
pub struct Workspace {
    /// Workspace directory
    pub workspace_dir: PathBuf,
    /// Configuration path
    pub config_path: Option<PathBuf>,
    /// Definition
    pub definition: Option<Definition>,
}

impl Workspace {
    /// Constructor
    pub fn new(workspace_dir: Option<&Path>, config_path: Option<&Path>) -> Result<Self> {
        match (workspace_dir, config_path) {
            (Some(d), Some(c)) => Self::known(d, Some(c)),
            (Some(d), None) => {
                let p = d.join(WORKSPACE_CONFIG_FILE_NAME);
                Self::known(d, if p.exists() { Some(&p) } else { None })
            }
            (None, Some(c)) => Self::known(
                c.to_path_buf()
                    .parent()
                    .ok_or_else(|| anyhow!("Invalid config path"))?,
                Some(c),
            ),
            (None, None) => Self::find(&env::current_dir()?),
        }
    }

    fn known(workspace_dir: &Path, config_path_opt: Option<&Path>) -> Result<Self> {
        match config_path_opt {
            Some(config_path) => Ok(Self {
                workspace_dir: workspace_dir.to_path_buf(),
                config_path: Some(config_path.to_path_buf()),
                definition: Some(read_yaml_file(config_path)?),
            }),
            None => Ok(Self {
                workspace_dir: workspace_dir.to_path_buf(),
                config_path: None,
                definition: None,
            }),
        }
    }

    fn find(search_dir: &Path) -> Result<Self> {
        Ok(
            match find_sentinel_file(WORKSPACE_CONFIG_FILE_NAME, search_dir, Some(5)) {
                Some(config_path) => {
                    let mut workspace_dir = config_path.clone();
                    if !workspace_dir.pop() {
                        bail!("Failed to obtain workspace directory")
                    }
                    let definition = read_yaml_file(&config_path)?;
                    Self {
                        workspace_dir,
                        config_path: Some(config_path),
                        definition: Some(definition),
                    }
                }
                None => Self {
                    workspace_dir: search_dir.to_path_buf(),
                    config_path: None,
                    definition: None,
                },
            },
        )
    }
}
