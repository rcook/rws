mod cli;
mod config;
mod deps;
mod os;
mod scripting;
mod workspace;

use crate::cli::make_rws_app;
use crate::cli::{arg, arg_value, command};
use crate::workspace::Workspace;

use clap::ArgMatches;
#[cfg(windows)]
use colored::control::set_virtual_terminal;
use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    set_virtual_terminal(true).unwrap();

    match make_rws_app().get_matches().subcommand() {
        (command::GIT, Some(s)) => run_helper(s, |cmd| {
            let mut command = Command::new("git");
            for i in 0..(cmd.len()) {
                command.arg(&cmd[i]);
            }
            command
        }),
        (command::INFO, Some(_)) => do_info(),
        (command::RUN, Some(s)) => run_helper(s, |cmd| {
            let mut command = Command::new(&cmd[0]);
            for i in 1..(cmd.len()) {
                command.arg(&cmd[i]);
            }
            command
        }),
        _ => panic!("Unimplemented"),
    }
}

fn do_info() -> std::io::Result<()> {
    let current_dir = env::current_dir()?;
    let workspace = Workspace::find(&current_dir).unwrap();
    println!(
        "Workspace directory: {}",
        workspace.root_dir.to_str().unwrap().cyan()
    );
    println!(
        "Workspace configuration: file {}",
        workspace.config_path.to_str().unwrap().cyan()
    );
    show_project_dirs("alpha", &workspace.project_dirs_alpha);
    show_project_dirs(arg_value::TOPO, &workspace.project_dirs_topo);
    Ok(())
}

fn show_project_dirs(order: &str, project_dirs: &Vec<PathBuf>) {
    if project_dirs.len() > 0 {
        println!("Project directories ({} order)", order);
        for project_dir in project_dirs {
            println!("  {}", project_dir.to_str().unwrap().cyan())
        }
    } else {
        println!("Project directories ({} order): {}", order, "(none)".cyan());
    }
}

fn run_helper<F>(submatches: &ArgMatches, f: F) -> std::io::Result<()>
where
    F: Fn(&Vec<&str>) -> Command,
{
    let cmd = &submatches
        .values_of(arg::CMD)
        .map(|x| x.collect())
        .unwrap_or(Vec::new());
    if cmd.len() < 1 {
        panic!("Unimplemented");
    }

    let fail_fast = !submatches.is_present(arg::NO_FAIL_FAST);
    let topo_order = submatches.value_of(arg::ORDER).unwrap() == arg_value::TOPO;

    let current_dir = env::current_dir()?;
    let workspace = Workspace::find(&current_dir).unwrap();
    let mut failure_count = 0;
    let project_dirs = if topo_order {
        &workspace.project_dirs_topo
    } else {
        &workspace.project_dirs_alpha
    };
    for project_dir in project_dirs {
        let d = project_dir.to_str().unwrap();
        println!("{}", d.cyan());
        let exit_status = f(cmd)
            .current_dir(&project_dir)
            .spawn()
            .unwrap()
            .wait()
            .unwrap();
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
