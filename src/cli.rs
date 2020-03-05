use clap::{crate_authors, crate_version, App, AppSettings, Arg, SubCommand};

pub const GIT_SUBCOMMAND: &str = "git";
pub const INFO_SUBCOMMAND: &str = "info";
pub const RUN_SUBCOMMAND: &str = "run";

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
        )
}
