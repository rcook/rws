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
use crate::git::GitInfo;
use crate::order::DirectoryOrder;
use crate::run_info::RunInfo;
use crate::util::reset_terminal;
use crate::workspace::{Plan, Workspace};
use anyhow::Result;
use colored::Colorize;
use joatmon::{path_to_str, WorkingDirectory};
use std::path::PathBuf;
use std::process::Command;

pub fn do_git(workspace: Workspace, run_info: &RunInfo) -> Result<()> {
    let git_info = GitInfo::from_environment()?;
    run_helper(&Plan::resolve(workspace)?, run_info, |cmd| {
        let mut command = Command::new(&git_info.executable_path);
        for c in cmd.iter() {
            command.arg(c);
        }
        command
    })
}

pub fn do_info(plan: &Plan, show_env: bool) -> Result<()> {
    println!(
        "Workspace directory: {}",
        path_to_str(&plan.workspace.workspace_dir).cyan()
    );
    println!(
        "Workspace configuration file: {}",
        plan.workspace
            .config_path
            .as_ref()
            .map(|x| path_to_str(x).cyan())
            .unwrap_or_else(|| "(none)".red().italic())
    );

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

pub fn do_run(workspace: Workspace, run_info: &RunInfo) -> Result<()> {
    run_helper(&Plan::resolve(workspace)?, run_info, |cmd| {
        let mut command = Command::new(&cmd[0]);
        for c in cmd.iter().skip(1) {
            command.arg(c);
        }
        command
    })
}

fn show_project_dirs(order: &str, project_dirs: &[PathBuf]) {
    if project_dirs.is_empty() {
        println!(
            "Project directories ({} order): {}",
            order,
            "(none)".red().italic()
        );
    } else {
        println!("Project directories ({} order):", order);
        for project_dir in project_dirs {
            println!("  {}", path_to_str(project_dir).cyan())
        }
    }
}

fn run_helper<F>(plan: &Plan, run_info: &RunInfo, f: F) -> Result<()>
where
    F: Fn(&Vec<String>) -> Command,
{
    let mut failure_count = 0;
    let project_dirs = match (&run_info.order, &plan.project_dirs_topo) {
        (DirectoryOrder::Topological, Some(ds)) => ds,
        _ => &plan.project_dirs_alpha,
    };
    for project_dir in project_dirs {
        let d = path_to_str(project_dir);
        println!("{}", d.cyan());
        let exit_status = f(&run_info.cmd).current_dir(project_dir).spawn()?.wait()?;
        reset_terminal();
        if exit_status.success() {
            println!("{}", format!("Command succeeded in {}", d).green())
        } else {
            failure_count += 1;
            match exit_status.code() {
                Some(code) => {
                    let m = format!("Command exited with status {} in {}", code, d);
                    println!(
                        "{}",
                        if run_info.fail_fast {
                            m.red()
                        } else {
                            m.yellow()
                        }
                    );
                    if run_info.fail_fast {
                        break;
                    }
                }
                None => println!("{}", format!("Command terminated by signal in {}", d).red()),
            }
        }
    }

    if !run_info.fail_fast && failure_count > 0 {
        println!(
            "{}",
            format!("Command failed in {} project directories", failure_count).red()
        )
    }

    Ok(())
}

pub fn do_init(workspace: &Workspace) -> Result<()> {
    match &workspace.init_command {
        Some(c) => {
            let working_dir = WorkingDirectory::change(&workspace.workspace_dir)?;
            c.eval()?;
            drop(working_dir);
        }
        None => {}
    }
    Ok(())
}
