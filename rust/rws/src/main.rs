mod config;
mod deps;
mod os;
mod scripting;
mod workspace;

use crate::workspace::Workspace;

use clap::{App, AppSettings, Arg, SubCommand};
#[cfg(windows)]
use colored::control::set_virtual_terminal;
use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const CARGO_PKG_AUTHORS: &'static str = env!("CARGO_PKG_AUTHORS");
const CARGO_PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const GIT_SUBCOMMAND: &str = "git";
const INFO_SUBCOMMAND: &str = "info";
const RUN_SUBCOMMAND: &str = "run";

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    set_virtual_terminal(true).unwrap();

    let app = App::new("Richard's Workspace Tool")
        .author(CARGO_PKG_AUTHORS)
        .about("Manages Git-based workspaces")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::TrailingVarArg)
        .version(CARGO_PKG_VERSION)
        .subcommand(
            SubCommand::with_name(GIT_SUBCOMMAND)
                .about("Run Git command in each project directory")
                .arg(
                    Arg::with_name("cmd")
                        .help("Command to pass to Git")
                        .multiple(true),
                ),
        )
        .subcommand(SubCommand::with_name(INFO_SUBCOMMAND).about("Prints workspace information"))
        .subcommand(
            SubCommand::with_name(RUN_SUBCOMMAND)
                .about("Run command in each project directory")
                .arg(
                    Arg::with_name("cmd")
                        .help("Command to pass to shell")
                        .multiple(true),
                ),
        );
    let matches = app.get_matches();

    match matches.subcommand() {
        (GIT_SUBCOMMAND, Some(_)) => {
            let submatches = matches.subcommand_matches(GIT_SUBCOMMAND).unwrap();
            do_git(
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
                &submatches
                    .values_of("cmd")
                    .map(|x| x.collect())
                    .unwrap_or(Vec::new()),
            )
        }
        _ => panic!("Unimplemented"),
    }
}

fn do_git(cmd: &Vec<&str>) -> std::io::Result<()> {
    if cmd.len() < 1 {
        panic!("Unimplemented");
    }

    run_helper(|| {
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

fn do_run(cmd: &Vec<&str>) -> std::io::Result<()> {
    if cmd.len() < 1 {
        panic!("Unimplemented");
    }

    run_helper(|| {
        let mut command = Command::new(&cmd[0]);
        for i in 1..(cmd.len()) {
            command.arg(&cmd[i]);
        }
        command
    })
}

fn run_helper<F>(f: F) -> std::io::Result<()>
where
    F: Fn() -> Command,
{
    let current_dir = env::current_dir()?;
    let workspace = Workspace::find(&current_dir).unwrap();
    for project_dir in &workspace.project_dirs_topo {
        println!("  {}", project_dir.to_str().unwrap().cyan());
        let exit_status = f().current_dir(&project_dir).spawn().unwrap().wait();
        println!("exit_status={:?}", exit_status)
    }

    Ok(())
}
