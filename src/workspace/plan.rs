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
use super::internal::{DependencySource, Workspace};
use super::topo_order::compute_topo_order;
use crate::config::ConfigHash;
use crate::scripting::ScriptCommand;
use anyhow::{anyhow, Result};
use joatmon::{get_base_name, WorkingDirectory};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// A build plan for a workspace
pub struct Plan {
    /// Workspace
    pub workspace: Workspace,
    /// Project directories in alphabetical order
    pub project_dirs_alpha: Vec<PathBuf>,
    /// Project directories in topological order
    pub project_dirs_topo: Option<Vec<PathBuf>>,
}

impl Plan {
    /// Create a plan from a workspace
    pub fn resolve(workspace: Workspace) -> Result<Self> {
        let project_dirs_alpha = Self::get_project_dirs_alpha(
            &workspace.workspace_dir,
            &workspace.excluded_project_dirs,
        )?;

        let project_dirs_topo = match &workspace.dependency_source {
            DependencySource::Hash(hash) => {
                Some(compute_topo_order(&project_dirs_alpha, |project_dir| {
                    Self::get_precs_from_config_hash(hash, &workspace, project_dir)
                })?)
            }
            DependencySource::ScriptCommand(command) => {
                Some(compute_topo_order(&project_dirs_alpha, |project_dir| {
                    Self::get_precs_from_script_command(command, &workspace, project_dir)
                })?)
            }
            DependencySource::None => None,
        };

        Ok(Self {
            workspace,
            project_dirs_alpha,
            project_dirs_topo,
        })
    }

    fn get_project_dirs_alpha(
        workspace_dir: &Path,
        excluded_project_dirs: &HashSet<PathBuf>,
    ) -> std::io::Result<Vec<PathBuf>> {
        let mut project_dirs_alpha = Vec::new();
        for entry in fs::read_dir(workspace_dir)? {
            let e = entry?;
            let project_dir = e.path();
            if !excluded_project_dirs.contains(&project_dir) && project_dir.is_dir() {
                let git_dir = project_dir.join(".git");
                if git_dir.exists() {
                    project_dirs_alpha.push(project_dir)
                }
            }
        }

        project_dirs_alpha.sort();
        Ok(project_dirs_alpha)
    }

    fn get_precs_from_config_hash(
        hash: &ConfigHash,
        workspace: &Workspace,
        project_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let project_name = get_base_name(project_dir)
            .ok_or_else(|| anyhow!("Invalid project directory {}", project_dir.display()))?;
        match hash.get(project_name).and_then(|x| x.into_vec()) {
            Some(v) => (0..v.len())
                .map(|i| {
                    v.get(i)
                        .into_string()
                        .ok_or_else(|| anyhow!("Invalid dependency for project {}", project_name))
                        .map(|s| workspace.workspace_dir.join(s))
                })
                .collect::<Result<Vec<_>>>(),
            None => Ok(Vec::new()),
        }
    }

    fn get_precs_from_script_command(
        command: &ScriptCommand,
        workspace: &Workspace,
        project_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let working_dir = WorkingDirectory::change(project_dir)?;
        let deps: Vec<String> = command.eval(workspace)?;
        drop(working_dir);
        Ok(deps
            .into_iter()
            .map(|x| workspace.workspace_dir.join(x))
            .collect::<Vec<_>>())
    }
}
