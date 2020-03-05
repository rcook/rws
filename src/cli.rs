use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

pub mod command {
    pub const GIT: &str = "git";
    pub const INFO: &str = "info";
    pub const RUN: &str = "run";
}

pub mod arg {
    pub const CMD: &str = "cmd";
    pub const FAIL_FAST: &str = "fail-fast";
    pub const NO_FAIL_FAST: &str = "no-fail-fast";
    pub const ORDER: &str = "order";
}

pub mod arg_value {
    pub const ALPHA: &str = "alpha";
    pub const TOPO: &str = "topo";
}

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

pub fn make_rws_app<'a, 'b>() -> App<'a, 'b> {
    App::new("Richard's Workspace Tool")
        .author(crate_authors!())
        .about("Manages Git-based workspaces")
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::TrailingVarArg)
        .version(crate_version!())
        .subcommand(run_command(
            command::GIT,
            "Runs Git command in each project directory",
            "Command to pass to Git",
        ))
        .subcommand(SubCommand::with_name(command::INFO).about("Prints workspace information"))
        .subcommand(run_command(
            command::RUN,
            "Runs command in each project directory",
            "Command to pass to shell",
        ))
}

fn run_command<'a, 'b>(name: &'a str, about: &'a str, cmd_help: &'a str) -> App<'a, 'b> {
    SubCommand::with_name(name)
        .about(about)
        .bool_switch(BoolSwitch::new(
            arg::FAIL_FAST,
            "Aborts command on first error (default)",
            arg::NO_FAIL_FAST,
            "Runs command in all project directories",
        ))
        .arg(
            Arg::with_name(arg::ORDER)
                .help("Order of project traversal")
                .long("order")
                .possible_values(&[arg_value::ALPHA, arg_value::TOPO])
                .takes_value(true)
                .default_value(arg_value::TOPO)
                .required(true),
        )
        .arg(Arg::with_name(arg::CMD).help(cmd_help).multiple(true))
}
