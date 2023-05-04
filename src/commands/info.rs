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
use super::helpers::show_project_dirs;
use crate::git::GitInfo;
use crate::workspace::{Plan, Workspace};
use anyhow::Result;
use colored::Colorize;
use joatmon::path_to_str;

pub fn do_info(workspace: &Workspace, show_env: bool) -> Result<()> {
    println!(
        "Workspace directory: {}",
        path_to_str(&workspace.workspace_dir).cyan()
    );
    println!(
        "Workspace configuration file: {}",
        workspace
            .config_path
            .as_ref()
            .map(|x| path_to_str(x).cyan())
            .unwrap_or_else(|| "(none)".red().italic())
    );

    let plan = Plan::new(workspace)?;
    show_project_dirs("alpha", &plan.project_dirs_alpha);
    match &plan.project_dirs_topo {
        Some(ds) => show_project_dirs("topo", ds),
        None => {}
    }

    if show_env {
        println!();
        match GitInfo::from_environment() {
            Ok(git_info) => {
                println!(
                    "Path to Git: {}",
                    path_to_str(&git_info.executable_path).cyan()
                );
                println!("Git version: {}", git_info.version.cyan())
            }
            _ => println!("Path to Git: {}", "(not found)".red().italic()),
        }
    }

    Ok(())
}