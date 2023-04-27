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
mod commands;
mod config;
mod config_key;
mod error;
mod git;
mod os;
mod run_info;
mod scripting;
mod util;
mod workspace;

use crate::cli::make_rws_app;
use crate::cli::{arg, command};
use crate::commands::{do_git, do_info, do_init, do_run};
use crate::error::{Error, Result};
use crate::run_info::RunInfo;
use crate::util::reset_terminal;
use crate::workspace::{Plan, Workspace};
use clap::ArgMatches;
use colored::Colorize;
use joat_path::absolute_path;
use std::env::current_dir;
use std::path::Path;
use std::process::exit;

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
        Some((command::GIT, s)) => do_git(workspace, &RunInfo::new(s)?),
        Some((command::INFO, submatches)) => {
            do_info(&Plan::resolve(workspace)?, submatches.is_present(arg::ENV))
        }
        Some((command::INIT, _)) => do_init(&workspace),
        Some((command::RUN, s)) => do_run(workspace, &RunInfo::new(s)?),
        _ => do_info(&Plan::resolve(workspace)?, false),
    }
}
