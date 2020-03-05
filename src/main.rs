mod cli;
mod config;
mod deps;
mod os;
mod scripting;
mod workspace;

use crate::cli::{make_rws_app, GIT_SUBCOMMAND, INFO_SUBCOMMAND, RUN_SUBCOMMAND};
use crate::workspace::Workspace;

#[cfg(windows)]
use colored::control::set_virtual_terminal;
use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    set_virtual_terminal(true).unwrap();

    let app = make_rws_app();
    let matches = app.get_matches();

    match matches.subcommand() {
        (GIT_SUBCOMMAND, Some(_)) => {
            let submatches = matches.subcommand_matches(GIT_SUBCOMMAND).unwrap();
            do_git(
                !submatches.is_present("no-fail-fast"),
                submatches.value_of("order").unwrap() == "topo",
                &submatches
                    .values_of("cmd")
                    .map(|x| x.collect())
                    .unwrap_or(Vec::new()),
            )
        }
        (INFO_SUBCOMMAND, Some(_)) => do_info(),
        (RUN_SUBCOMMAND, Some(_)) => {
            let submatches = matches.subcommand_matches(RUN_SUBCOMMAND).unwrap();
            do_run(
                !submatches.is_present("no-fail-fast"),
                submatches.value_of("order").unwrap() == "topo",
                &submatches
                    .values_of("cmd")
                    .map(|x| x.collect())
                    .unwrap_or(Vec::new()),
            )
        }
        _ => panic!("Unimplemented"),
    }
}

fn do_git(fail_fast: bool, topo_order: bool, cmd: &Vec<&str>) -> std::io::Result<()> {
    if cmd.len() < 1 {
        panic!("Unimplemented");
    }

    run_helper(fail_fast, topo_order, || {
        let mut command = Command::new("git");
        for i in 0..(cmd.len()) {
            command.arg(&cmd[i]);
        }
        command
    })
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
    show_project_dirs("topo", &workspace.project_dirs_topo);
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

fn do_run(fail_fast: bool, topo_order: bool, cmd: &Vec<&str>) -> std::io::Result<()> {
    if cmd.len() < 1 {
        panic!("Unimplemented");
    }

    run_helper(fail_fast, topo_order, || {
        let mut command = Command::new(&cmd[0]);
        for i in 1..(cmd.len()) {
            command.arg(&cmd[i]);
        }
        command
    })
}

fn run_helper<F>(fail_fast: bool, topo_order: bool, f: F) -> std::io::Result<()>
where
    F: Fn() -> Command,
{
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
        let exit_status = f()
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
