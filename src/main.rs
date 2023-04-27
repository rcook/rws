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
mod cli;
mod config;
mod config_key;
mod error;
mod git;
mod os;
mod scripting;
mod util;
mod workspace;

use crate::cli::make_rws_app;
use crate::cli::{arg, arg_value, command};
use crate::error::{user_error_result, Error, Result};
use crate::git::GitInfo;
use crate::os::{path_to_str, with_working_dir};
use crate::workspace::{Plan, Workspace};

use clap::ArgMatches;
#[cfg(windows)]
use colored::control::set_virtual_terminal;
use colored::Colorize;
use joat_path::absolute_path;
use std::env::current_dir;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn reset_terminal() {
    #[cfg(windows)]
    set_virtual_terminal(true).expect("set_virtual_terminal failed");
}

fn main() {
    exit(match main_inner() {
        Ok(()) => 0,
        Err(Error::User(message)) => {
            println!("{}", format!("Error: {}", message).bright_red());
            1
        }
        Err(Error::Internal(facility, message)) => {
            println!(
                "{}",
                format!("Internal ({}): {}", facility, message).red().bold()
            );
            2
        }
    })
}

fn get_workspace(matches: &ArgMatches) -> Result<Workspace> {
    let base_dir = current_dir()?;
    match matches.value_of(arg::CONFIG) {
        Some(c) => {
            let config_path = absolute_path(&base_dir, Path::new(c))?;
            match matches.value_of(arg::DIR) {
                Some(d) => {
                    let workspace_dir = absolute_path(&base_dir, Path::new(d))?;
                    Workspace::new(Some(workspace_dir), Some(config_path))
                }
                None => Workspace::new(None, Some(config_path)),
            }
        }
        None => match matches.value_of(arg::DIR) {
            Some(d) => {
                let workspace_dir = absolute_path(&base_dir, Path::new(d))?;
                Workspace::new(Some(workspace_dir), None)
            }
            None => Workspace::new(None, None),
        },
    }
}

fn main_inner() -> Result<()> {
    reset_terminal();

    let matches = make_rws_app().get_matches();

    let workspace = get_workspace(&matches)?;

    match matches.subcommand() {
        Some((command::GIT, s)) => {
            let git_info = GitInfo::from_environment()?;
            run_helper(&Plan::resolve(workspace)?, s, |cmd| {
                let mut command = Command::new(&git_info.executable_path);
                for c in cmd.iter().skip(1) {
                    command.arg(c);
                }
                command
            })
        }

        Some((command::INFO, submatches)) => do_info(&Plan::resolve(workspace)?, Some(submatches)),

        Some((command::INIT, _)) => do_init(&workspace),

        Some((command::RUN, s)) => run_helper(&Plan::resolve(workspace)?, s, |cmd| {
            let mut command = Command::new(cmd[0]);
            for c in cmd.iter().skip(1) {
                command.arg(c);
            }
            command
        }),

        _ => do_info(&Plan::resolve(workspace)?, None),
    }
}

fn do_info(plan: &Plan, submatches: Option<&ArgMatches>) -> Result<()> {
    let show_env = submatches.map(|x| x.is_present(arg::ENV)).unwrap_or(false);

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
        Some(ds) => show_project_dirs(arg_value::TOPO, ds),
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

fn run_helper<F>(plan: &Plan, submatches: &ArgMatches, f: F) -> Result<()>
where
    F: Fn(&Vec<&str>) -> Command,
{
    let cmd = &submatches
        .values_of(arg::CMD)
        .map(|x| x.collect())
        .unwrap_or_else(Vec::new);
    if cmd.is_empty() {
        return user_error_result("Command requires at least one command argument");
    }

    let fail_fast = !submatches.is_present(arg::NO_FAIL_FAST);
    let topo_order = submatches
        .value_of(arg::ORDER)
        .expect("--order is required")
        == arg_value::TOPO;

    let mut failure_count = 0;
    let project_dirs = match (topo_order, &plan.project_dirs_topo) {
        (true, Some(ds)) => ds,
        _ => &plan.project_dirs_alpha,
    };
    for project_dir in project_dirs {
        let d = path_to_str(project_dir);
        println!("{}", d.cyan());
        let exit_status = f(cmd).current_dir(project_dir).spawn()?.wait()?;
        reset_terminal();
        if exit_status.success() {
            println!("{}", format!("Command succeeded in {}", d).green())
        } else {
            failure_count += 1;
            match exit_status.code() {
                Some(code) => {
                    let m = format!("Command exited with status {} in {}", code, d);
                    println!("{}", if fail_fast { m.red() } else { m.yellow() });
                    if fail_fast {
                        break;
                    }
                }
                None => println!("{}", format!("Command terminated by signal in {}", d).red()),
            }
        }
    }

    if !fail_fast && failure_count > 0 {
        println!(
            "{}",
            format!("Command failed in {} project directories", failure_count).red()
        )
    }

    Ok(())
}

fn do_init(workspace: &Workspace) -> Result<()> {
    match &workspace.init_command {
        Some(c) => with_working_dir(&workspace.workspace_dir, || c.eval())??,
        None => {}
    }
    Ok(())
}
