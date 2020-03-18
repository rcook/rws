mod cli;
mod config;
mod error;
mod os;
mod scripting;
mod workspace;

use crate::cli::make_rws_app;
use crate::cli::{arg, arg_value, command};
use crate::error::{user_error_result, AppError, Result};
use crate::os::{path_to_str, with_working_dir};
use crate::workspace::{Plan, Workspace};

use clap::ArgMatches;
#[cfg(windows)]
use colored::control::set_virtual_terminal;
use colored::Colorize;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};

fn reset_terminal() -> () {
    #[cfg(windows)]
    set_virtual_terminal(true).expect("set_virtual_terminal failed");
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

fn get_workspace(matches: &ArgMatches) -> Result<Workspace> {
    match matches.value_of("config") {
        Some(c) => {
            let config_path = Path::new(c).canonicalize()?;
            match matches.value_of("dir") {
                Some(d) => {
                    let workspace_dir = Path::new(d).canonicalize()?;
                    Workspace::new(Some(workspace_dir), Some(config_path))
                }
                None => Workspace::new(None, Some(config_path)),
            }
        }
        None => match matches.value_of("dir") {
            Some(d) => {
                let workspace_dir = Path::new(d).canonicalize()?;
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
        (command::GIT, Some(s)) => run_helper(&Plan::resolve(workspace)?, s, |cmd| {
            let mut command = Command::new("git");
            for i in 0..(cmd.len()) {
                command.arg(&cmd[i]);
            }
            command
        }),

        (command::INFO, Some(_)) => do_info(&Plan::resolve(workspace)?),

        (command::INIT, _) => do_init(&workspace),

        (command::RUN, Some(s)) => run_helper(&Plan::resolve(workspace)?, s, |cmd| {
            let mut command = Command::new(&cmd[0]);
            for i in 1..(cmd.len()) {
                command.arg(&cmd[i]);
            }
            command
        }),

        _ => do_info(&Plan::resolve(workspace)?),
    }
}

fn do_info(plan: &Plan) -> Result<()> {
    println!(
        "Workspace directory: {}",
        path_to_str(&plan.workspace.workspace_dir).cyan()
    );
    println!(
        "Workspace configuration file: {}",
        plan.workspace
            .config_path
            .as_ref()
            .map(|x| path_to_str(x))
            .unwrap_or("(none)")
            .cyan()
    );
    show_project_dirs("alpha", &plan.project_dirs_alpha);
    match &plan.project_dirs_topo {
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

fn run_helper<F>(plan: &Plan, submatches: &ArgMatches, f: F) -> Result<()>
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

    let mut failure_count = 0;
    let project_dirs = match (topo_order, &plan.project_dirs_topo) {
        (true, Some(ds)) => ds,
        _ => &plan.project_dirs_alpha,
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

fn do_init(workspace: &Workspace) -> Result<()> {
    match &workspace.init_command {
        Some(c) => with_working_dir(&workspace.workspace_dir, || c.eval())??,
        None => {}
    }
    Ok(())
}
