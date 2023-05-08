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
use super::internal::Workspace;
use super::topo_order::compute_topo_order;
use crate::config::{Command, DependencySource, StaticDependencies};
use crate::scripting::ScriptCommand;
use anyhow::{anyhow, Result};
use joatmon::{get_base_name, WorkingDirectory};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

/// A build plan for a workspace
pub struct Plan {
    /// Project directories in alphabetical order
    pub project_dirs_alpha: Vec<PathBuf>,
    /// Project directories in topological order
    pub project_dirs_topo: Option<Vec<PathBuf>>,
}

impl Plan {
    /// Create a plan from a workspace
    pub fn new(workspace: &Workspace) -> Result<Self> {
        let exclude_project_dirs = HashSet::from_iter(
            workspace
                .definition
                .as_ref()
                .and_then(|d| d.excluded_projects.as_ref())
                .unwrap_or(&Vec::new())
                .iter()
                .map(|s| workspace.workspace_dir.join(s)),
        );

        let project_dirs_alpha =
            Self::get_project_dirs_alpha(&workspace.workspace_dir, &exclude_project_dirs)?;

        let project_dirs_topo = match &workspace.definition {
            Some(d) => match &d.dependency_source {
                Some(DependencySource::Static(static_dependencies)) => {
                    Some(compute_topo_order(&project_dirs_alpha, |project_dir| {
                        Self::get_precs_from_config_hash(
                            static_dependencies,
                            workspace,
                            project_dir,
                        )
                    })?)
                }
                Some(DependencySource::Command(command)) => {
                    Some(compute_topo_order(&project_dirs_alpha, |project_dir| {
                        Self::get_precs_from_script_command(command, workspace, project_dir)
                    })?)
                }
                None => None,
            },
            None => None,
        };
        Ok(Self {
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
        static_dependencies: &StaticDependencies,
        workspace: &Workspace,
        project_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let project_name = get_base_name(project_dir)
            .ok_or_else(|| anyhow!("Invalid project directory {}", project_dir.display()))?;

        Ok(static_dependencies
            .get(project_name)
            .map(|ps| {
                ps.iter()
                    .map(|p| workspace.workspace_dir.join(p))
                    .collect::<Vec<_>>()
            })
            .unwrap_or(Vec::new()))
    }

    fn get_precs_from_script_command(
        command: &Command,
        workspace: &Workspace,
        project_dir: &Path,
    ) -> Result<Vec<PathBuf>> {
        let working_dir = WorkingDirectory::change(project_dir)?;
        let cmd = ScriptCommand::new(workspace.definition.as_ref(), command)?;
        let deps: Vec<String> = cmd.eval(workspace)?;
        drop(working_dir);
        Ok(deps
            .into_iter()
            .map(|x| workspace.workspace_dir.join(x))
            .collect::<Vec<_>>())
    }
}
