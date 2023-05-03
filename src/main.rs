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
mod args;
mod commands;
mod config;
mod config_key;
mod git;
mod order;
mod run_info;
mod scripting;
mod util;
mod workspace;

use crate::args::{Args, Command};
use crate::commands::{do_git, do_info, do_init, do_run};
use crate::run_info::RunInfo;
use crate::util::reset_terminal;
use crate::workspace::{Plan, Workspace};
use anyhow::Result;
use clap::Parser;
use colored::Colorize;
use std::process::exit;

fn main() {
    exit(match run() {
        Ok(_) => 0,
        Err(e) => {
            println!("{}", format!("{}", e).bright_red());
            1
        }
    })
}

fn run() -> Result<()> {
    reset_terminal();
    let args = Args::parse();
    let workspace = Workspace::new(args.workspace_dir.as_deref(), args.config_path.as_deref())?;
    match args.command {
        Command::Git {
            fail_fast,
            order,
            command,
            args,
        } => do_git(&workspace, &RunInfo::new(command, args, fail_fast, order))?,
        Command::Info => do_info(&workspace, &Plan::new(&workspace)?, true)?,
        Command::Init => do_init(&workspace)?,
        Command::Run {
            fail_fast,
            order,
            command,
            args,
        } => do_run(&workspace, &RunInfo::new(command, args, fail_fast, order))?,
    }
    Ok(())
}
