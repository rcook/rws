mod cli;
mod config;
mod deps;
mod error;
mod os;
mod scripting;
mod workspace;

use crate::cli::make_rws_app;
use crate::cli::{arg, arg_value, command};
use crate::error::{user_error, user_error_result, AppError, Result};
use crate::os::path_to_str;
use crate::workspace::Workspace;

use clap::ArgMatches;
#[cfg(windows)]
use colored::control::set_virtual_terminal;
use colored::Colorize;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn reset_terminal() -> () {
    #[cfg(windows)]
    set_virtual_terminal(true).expect("set_virtual_terminal failed");
}

fn get_workspace(config_path: Option<PathBuf>) -> Result<Workspace> {
    match config_path {
        Some(c) => Workspace::new(
            c.parent()
                .ok_or_else(|| user_error("Invalid config path"))?,
            &c,
        ),
        None => Workspace::find(&env::current_dir()?),
    }
}

fn main() {
    exit(match main_inner() {
        Ok(()) => 0,
        Err(AppError::User(message)) => {
            println!("{}", format!("Error: {}", message).bright_red());
            1
        }
        Err(AppError::System(facility, message)) => {
            println!(
                "{}",
                format!("System ({}): {}", facility, message).red().bold()
            );
            2
        }
    })
}

fn main_inner() -> Result<()> {
    reset_terminal();

    let matches = make_rws_app().get_matches();

    let config_path = match matches.value_of("config") {
        Some(c) => Some(Path::new(c).canonicalize()?),
        None => None,
    };

    match matches.subcommand() {
        (command::GIT, Some(s)) => run_helper(config_path, s, |cmd| {
            let mut command = Command::new("git");
            for i in 0..(cmd.len()) {
                command.arg(&cmd[i]);
            }
            command
        }),

        (command::INFO, Some(_)) => do_info(config_path),
        (command::RUN, Some(s)) => run_helper(config_path, s, |cmd| {
            let mut command = Command::new(&cmd[0]);
            for i in 1..(cmd.len()) {
                command.arg(&cmd[i]);
            }
            command
        }),
        _ => do_info(config_path),
    }
}

fn do_info(config_path: Option<PathBuf>) -> Result<()> {
    let workspace = get_workspace(config_path)?;
    println!(
        "Workspace directory: {}",
        path_to_str(&workspace.root_dir).cyan()
    );
    println!(
        "Workspace configuration file: {}",
        workspace
            .config_path
            .as_ref()
            .map(|x| path_to_str(x))
            .unwrap_or("(none)")
            .cyan()
    );
    show_project_dirs("alpha", &workspace.project_dirs_alpha);
    match workspace.project_dirs_topo {
        Some(ds) => show_project_dirs(arg_value::TOPO, &ds),
        None => {}
    }
    Ok(())
}

fn show_project_dirs(order: &str, project_dirs: &Vec<PathBuf>) {
    if project_dirs.len() > 0 {
        println!("Project directories ({} order):", order);
        for project_dir in project_dirs {
            println!("  {}", path_to_str(project_dir).cyan())
        }
    } else {
        println!("Project directories ({} order): {}", order, "(none)".cyan());
    }
}

fn run_helper<F>(config_path: Option<PathBuf>, submatches: &ArgMatches, f: F) -> Result<()>
where
    F: Fn(&Vec<&str>) -> Command,
{
    let cmd = &submatches
        .values_of(arg::CMD)
        .map(|x| x.collect())
        .unwrap_or(Vec::new());
    if cmd.len() < 1 {
        return user_error_result("Command requires at least one command argument");
    }

    let fail_fast = !submatches.is_present(arg::NO_FAIL_FAST);
    let topo_order = submatches
        .value_of(arg::ORDER)
        .expect("--order is required")
        == arg_value::TOPO;

    let workspace = get_workspace(config_path)?;
    let mut failure_count = 0;
    let project_dirs = match (topo_order, &workspace.project_dirs_topo) {
        (true, Some(ds)) => ds,
        _ => &workspace.project_dirs_alpha,
    };
    for project_dir in project_dirs {
        let d = path_to_str(project_dir);
        println!("{}", d.cyan());
        let exit_status = f(cmd).current_dir(&project_dir).spawn()?.wait()?;
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
