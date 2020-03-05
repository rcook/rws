mod config;
mod deps;
mod os;
mod scripting;
mod workspace;

use crate::workspace::Workspace;

use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};
#[cfg(windows)]
use colored::control::set_virtual_terminal;
use colored::Colorize;
use std::env;
use std::path::PathBuf;
use std::process::Command;

const GIT_SUBCOMMAND: &str = "git";
const INFO_SUBCOMMAND: &str = "info";
const RUN_SUBCOMMAND: &str = "run";

struct BoolSwitch<'a> {
    name: &'a str,
    help: &'a str,
    no_name: &'a str,
    no_help: &'a str,
}

impl<'a> BoolSwitch<'a> {
    fn new(name: &'a str, help: &'a str, no_name: &'a str, no_help: &'a str) -> BoolSwitch<'a> {
        BoolSwitch {
            name: name,
            help: help,
            no_name: no_name,
            no_help: no_help,
        }
    }
}

trait BoolSwitchExt<'a> {
    fn bool_switch(self, bs: BoolSwitch<'a>) -> Self;
}

impl<'a, 'b> BoolSwitchExt<'a> for App<'a, 'b> {
    fn bool_switch(self, bs: BoolSwitch<'a>) -> Self {
        self.arg(
            Arg::with_name(bs.name)
                .long(bs.name)
                .help(bs.help)
                .takes_value(false),
        )
        .arg(
            Arg::with_name(bs.no_name)
                .conflicts_with(bs.name)
                .long(bs.no_name)
                .help(bs.no_help)
                .takes_value(false),
        )
    }
}

fn main() -> std::io::Result<()> {
    #[cfg(windows)]
    set_virtual_terminal(true).unwrap();

    let app = App::new("Richard's Workspace Tool")
        .author(crate_authors!())
        .about("Manages Git-based workspaces")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::TrailingVarArg)
        .version(crate_version!())
        .subcommand(
            SubCommand::with_name(GIT_SUBCOMMAND)
                .about("Runs Git command in each project directory")
                .bool_switch(BoolSwitch::new(
                    "fail-fast",
                    "Aborts command on first error (default)",
                    "no-fail-fast",
                    "Runs command in all project directories",
                ))
                .arg(
                    Arg::with_name("order")
                        .help("Order of project traversal")
                        .long("order")
                        .possible_values(&["alpha", "topo"])
                        .takes_value(true)
                        .default_value("topo")
                        .required(true),
                )
                .arg(
                    Arg::with_name("cmd")
                        .help("Command to pass to Git")
                        .multiple(true),
                ),
        )
        .subcommand(SubCommand::with_name(INFO_SUBCOMMAND).about("Prints workspace information"))
        .subcommand(
            SubCommand::with_name(RUN_SUBCOMMAND)
                .about("Runs command in each project directory")
                .bool_switch(BoolSwitch::new(
                    "fail-fast",
                    "Aborts command on first error (default)",
                    "no-fail-fast",
                    "Runs command in all project directories",
                ))
                .arg(
                    Arg::with_name("order")
                        .help("Order of project traversal")
                        .long("order")
                        .possible_values(&["alpha", "topo"])
                        .takes_value(true)
                        .default_value("topo")
                        .required(true),
                )
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
