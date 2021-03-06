use clap::{crate_authors, App, AppSettings, Arg, SubCommand};

pub mod command {
    pub const GIT: &str = "git";
    pub const INFO: &str = "info";
    pub const INIT: &str = "init";
    pub const RUN: &str = "run";
}

pub mod arg {
    pub const CMD: &str = "cmd";
    pub const CONFIG: &str = "config";
    pub const DIR: &str = "dir";
    pub const ENV: &str = "env";
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
            name,
            help,
            no_name,
            no_help,
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
                .help(bs.help)
                .long(bs.name)
                .takes_value(false),
        )
        .arg(
            Arg::with_name(bs.no_name)
                .conflicts_with(bs.name)
                .help(bs.no_help)
                .long(bs.no_name)
                .takes_value(false),
        )
    }
}

pub fn make_rws_app<'a, 'b>() -> App<'a, 'b> {
    use arg::*;
    use command::*;

    App::new("Richard's Workspace Tool")
        .author(crate_authors!())
        .about("Manages Git-based workspaces")
        .setting(AppSettings::TrailingVarArg)
        .version(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name(CONFIG)
                .help("Path to configuration file")
                .long(CONFIG)
                .value_name("CONFIG-PATH")
                .takes_value(true),
        )
        .arg(
            Arg::with_name(DIR)
                .help("Path to workspace directory")
                .long(DIR)
                .value_name("WORKSPACE-DIR")
                .takes_value(true),
        )
        .subcommand(run_command(
            GIT,
            "Runs Git command in each project directory using system Git command",
            "Command to pass to Git",
        ))
        .subcommand(
            SubCommand::with_name(INFO)
                .about("Prints workspace and environment information")
                .arg(
                    Arg::with_name(ENV)
                        .help("Shows additional environment information")
                        .long(ENV)
                        .takes_value(false),
                ),
        )
        .subcommand(SubCommand::with_name(INIT).about("Initializes workspace"))
        .subcommand(run_command(
            RUN,
            "Runs command in each project directory",
            "Command to pass to shell",
        ))
}

fn run_command<'a, 'b>(name: &'a str, about: &'a str, cmd_help: &'a str) -> App<'a, 'b> {
    use arg::*;
    use arg_value::*;

    SubCommand::with_name(name)
        .about(about)
        .bool_switch(BoolSwitch::new(
            FAIL_FAST,
            "Aborts command on first error (default)",
            NO_FAIL_FAST,
            "Runs command in all project directories",
        ))
        .arg(
            Arg::with_name(ORDER)
                .help("Order of project traversal")
                .long(ORDER)
                .possible_values(&[ALPHA, TOPO])
                .takes_value(true)
                .default_value(TOPO)
                .required(true),
        )
        .arg(Arg::with_name(CMD).help(cmd_help).multiple(true))
}
